import struct
import zlib


PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
MAX_PIXELS = 50_000_000


class QaError(Exception):
    pass


def png_chunk(kind, data):
    checksum = zlib.crc32(kind)
    checksum = zlib.crc32(data, checksum) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)


def write_png(path, width, height, pixels):
    if len(pixels) != width * height:
        raise QaError("pixel count does not match PNG dimensions")
    rows = []
    for y in range(height):
        row = bytearray([0])
        for pixel in pixels[y * width : (y + 1) * width]:
            row.extend(pixel)
        rows.append(bytes(row))
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    payload = PNG_SIGNATURE
    payload += png_chunk(b"IHDR", header)
    payload += png_chunk(b"IDAT", zlib.compress(b"".join(rows), 9))
    payload += png_chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(payload)


def paeth(left, up, upper_left):
    value = left + up - upper_left
    left_distance = abs(value - left)
    up_distance = abs(value - up)
    upper_left_distance = abs(value - upper_left)
    if left_distance <= up_distance and left_distance <= upper_left_distance:
        return left
    if up_distance <= upper_left_distance:
        return up
    return upper_left


def unfilter_row(filter_type, row, previous, bytes_per_pixel):
    result = bytearray(len(row))
    for index, value in enumerate(row):
        left = result[index - bytes_per_pixel] if index >= bytes_per_pixel else 0
        up = previous[index] if previous else 0
        upper_left = (
            previous[index - bytes_per_pixel]
            if previous and index >= bytes_per_pixel
            else 0
        )
        if filter_type == 0:
            predictor = 0
        elif filter_type == 1:
            predictor = left
        elif filter_type == 2:
            predictor = up
        elif filter_type == 3:
            predictor = (left + up) // 2
        elif filter_type == 4:
            predictor = paeth(left, up, upper_left)
        else:
            raise QaError(f"unsupported PNG filter {filter_type}")
        result[index] = (value + predictor) & 0xFF
    return result


def read_chunks(path):
    content = path.read_bytes()
    if not content.startswith(PNG_SIGNATURE):
        raise QaError(f"{path} is not a PNG file")
    offset = len(PNG_SIGNATURE)
    chunks = []
    while offset < len(content):
        if offset + 12 > len(content):
            raise QaError(f"{path} has a truncated PNG chunk")
        length = struct.unpack(">I", content[offset : offset + 4])[0]
        kind = content[offset + 4 : offset + 8]
        start = offset + 8
        end = start + length
        if end + 4 > len(content):
            raise QaError(f"{path} has a truncated PNG payload")
        data = content[start:end]
        expected = struct.unpack(">I", content[end : end + 4])[0]
        checksum = zlib.crc32(kind)
        checksum = zlib.crc32(data, checksum) & 0xFFFFFFFF
        if expected != checksum:
            raise QaError(f"{path} has an invalid PNG checksum")
        chunks.append((kind, data))
        offset = end + 4
        if kind == b"IEND":
            break
    return chunks


def decode_rows(raw, width, height, channels):
    stride = width * channels
    if len(raw) != (stride + 1) * height:
        raise QaError("PNG has unexpected decoded size")
    rows = []
    previous = None
    cursor = 0
    for _ in range(height):
        filter_type = raw[cursor]
        cursor += 1
        encoded = raw[cursor : cursor + stride]
        cursor += stride
        row = unfilter_row(filter_type, encoded, previous, channels)
        rows.append(row)
        previous = row
    return rows


def decode_pixels(rows, width, color_type, palette, transparency, path):
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color_type]
    transparent_gray = None
    transparent_rgb = None
    if color_type == 0 and len(transparency) >= 2:
        transparent_gray = struct.unpack(">H", transparency[:2])[0]
    if color_type == 2 and len(transparency) >= 6:
        transparent_rgb = struct.unpack(">HHH", transparency[:6])
    pixels = []
    for row in rows:
        for x in range(width):
            values = row[x * channels : (x + 1) * channels]
            if color_type == 0:
                gray = values[0]
                alpha = 0 if transparent_gray == gray else 255
                pixels.append((gray, gray, gray, alpha))
            elif color_type == 2:
                red, green, blue = values
                alpha = 0 if transparent_rgb == (red, green, blue) else 255
                pixels.append((red, green, blue, alpha))
            elif color_type == 3:
                if palette is None or values[0] >= len(palette):
                    raise QaError(f"{path} has an invalid PNG palette index")
                red, green, blue = palette[values[0]]
                alpha = transparency[values[0]] if values[0] < len(transparency) else 255
                pixels.append((red, green, blue, alpha))
            elif color_type == 4:
                gray, alpha = values
                pixels.append((gray, gray, gray, alpha))
            else:
                pixels.append(tuple(values))
    return pixels


def read_png(path):
    width = height = bit_depth = color_type = interlace = None
    palette = None
    transparency = b""
    compressed = bytearray()
    for kind, data in read_chunks(path):
        if kind == b"IHDR":
            width, height, bit_depth, color_type, _, _, interlace = struct.unpack(
                ">IIBBBBB", data
            )
        elif kind == b"PLTE":
            palette = [
                tuple(data[index : index + 3]) for index in range(0, len(data), 3)
            ]
        elif kind == b"tRNS":
            transparency = data
        elif kind == b"IDAT":
            compressed.extend(data)
    if width is None or height is None:
        raise QaError(f"{path} has no PNG header")
    if width == 0 or height == 0 or width * height > MAX_PIXELS:
        raise QaError(f"{path} has unsupported PNG dimensions")
    if bit_depth != 8 or interlace != 0:
        raise QaError("visual QA supports only 8-bit non-interlaced PNG files")
    channel_counts = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}
    if color_type not in channel_counts:
        raise QaError(f"unsupported PNG color type {color_type}")
    channels = channel_counts[color_type]
    try:
        raw = zlib.decompress(bytes(compressed))
    except zlib.error as error:
        raise QaError(f"{path} has invalid compressed PNG data") from error
    rows = decode_rows(raw, width, height, channels)
    pixels = decode_pixels(rows, width, color_type, palette, transparency, path)
    return width, height, pixels
