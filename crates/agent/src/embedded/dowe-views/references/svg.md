# SVG to Dowe reference

Use this reference when a UI includes a local SVG logo, icon, mark, illustration, diagram, or
other vector asset. Dowe's shared `parse.svg` implementation is the authority for conversion; the
integrated native agent exposes the same behavior through the read-only `convert_svg` tool.

## Choose the output form

| Need | Tool output | Dowe usage |
| --- | --- | --- |
| A fixed asset that will be authored in a view | `format:"source"` | Paste the returned `Svg` and direct `Path` lines into the view |
| Geometry arriving from a request, signal, or repeated row | `format:"data"` | Bind the returned normalized JSON with `Svg data:<reference>` |
| Theme-aware source colors | `format:"source" colors:"tokens"` | Use semantic `primary`, `secondary`, `accent`, and related tokens |
| Asset-faithful source colors | `format:"source" colors:"original"` | Keep the returned hexadecimal fills |

`convert_svg` defaults to `format:"source" colors:"original"` because a local asset normally needs
to preserve its visual identity. `format:"data"` always uses original colors. It is not a second
SVG parser and it does not write a converted file.

## Native agent tool

Pass a project-relative `.svg` path. The tool is read-only and returns a bounded result whose
`content` field is either Dowe source or a normalized JSON string:

```text
convert_svg path:"assets/brand/mark.svg" format:"source" colors:"original"
```

The source result is ready to adapt as a static vector:

```text
Svg viewBox:"0 0 24 24" w:"full" h:"full"
  Path d:"M4 4L20 4L12 20Z" fill:"#2457D6"
```

The data result is intended for a runtime binding:

```text
signal markData value:"{\"viewBox\":\"0 0 24 24\",\"paths\":[{\"d\":\"M4 4L20 4L12 20Z\",\"paint\":\"fill\",\"color\":\"#2457D6\"}]}"
Svg data:markData w:24 h:24
```

For a runtime result, keep the `Svg` in data form and do not add static `viewBox` or `Path`
children. For a static result, keep `Path` as a direct child of `Svg`; it is not a standalone
component. Use `fillRule:"evenodd"` and the returned matrix transform when the converted source
contains them.

## Conversion rules

The portable importer understands `svg`, `g`, `path`, non-rounded `rect`, inline `fill` and
`fill-rule` styles, and affine `matrix(...)` transforms. A supported rectangle becomes one path.
Unsupported drawing elements, rounded rectangles, and non-portable decorations are skipped. The
SVG must still produce a viewBox and at least one portable path.

`colors:"tokens"` maps non-portable fills to semantic Dowe colors in encounter order. Integer RGB
fills reuse an existing token when the channels are effectively the same. `colors:"original"`
preserves hexadecimal fills and normalizes integer RGB fills to six-digit hexadecimal values.

The importer accepts at most 262144 UTF-8 bytes and emits at most 1024 paths. Invalid dimensions,
path data, fill rules, transforms, or markup are errors. Use the error to repair or replace the
asset; do not fall back to raw XML.

## Safety and UI boundaries

The tool accepts only project-relative regular files and does not traverse symlinks or hard links,
private or generated directories, or instruction files. It never executes scripts, entities,
events, CSS, URLs, or external resources. It never mutates the asset and needs no approval.

Use `Svg` for vector marks, logos, icons, and geometric illustrations. Use `Image` for independently
obtained raster photographs, textures, or illustrations. Use `Canvas` only when the visual is an
interactive or custom drawing that the semantic vector contract cannot express. Preserve one
converted vector asset rather than reproducing it as unrelated shapes.
