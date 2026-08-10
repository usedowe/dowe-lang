import json
from pathlib import Path

from visual_qa_png import QaError, read_png, write_png


THEME_FIELDS = ("colors", "typography", "spacing", "radii", "shadows")
REGION_FIELDS = (
    "id",
    "band",
    "bounds",
    "owner",
    "component",
    "container",
    "dataOwner",
    "responsive",
    "states",
    "accessibility",
)


def require_generated_path(path, project=None):
    resolved = path.resolve()
    generated_root = (Path(project).resolve() if project else Path.cwd().resolve()) / ".dowe"
    try:
        resolved.relative_to(generated_root)
    except ValueError as error:
        raise QaError("visual QA output must stay under the project .dowe directory")
    return resolved


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def initialize_blueprint(reference, output):
    width, height, _ = read_png(reference)
    target = require_generated_path(output)
    blueprint = {
        "viewport": {"width": width, "height": height},
        "bands": [{"id": "full", "top": 0, "bottom": height}],
        "regions": [],
        "theme": {field: [] for field in THEME_FIELDS},
        "assets": [],
        "candidateComponents": [],
    }
    write_json(target, blueprint)
    print(target)


def require_string_list(value, label, allow_empty=False):
    if not isinstance(value, list) or (not allow_empty and not value):
        raise QaError(f"{label} must be a non-empty string array")
    if any(not isinstance(item, str) or not item.strip() for item in value):
        raise QaError(f"{label} must contain non-empty strings")


def validate_bands(bands, height):
    if not isinstance(bands, list) or not bands:
        raise QaError("blueprint bands must be a non-empty array")
    cursor = 0
    identifiers = set()
    for band in bands:
        if not isinstance(band, dict):
            raise QaError("every blueprint band must be an object")
        identifier = band.get("id")
        top = band.get("top")
        bottom = band.get("bottom")
        if not isinstance(identifier, str) or not identifier.strip():
            raise QaError("every blueprint band requires a non-empty id")
        if identifier in identifiers:
            raise QaError(f"duplicate blueprint band id {identifier}")
        if not isinstance(top, int) or not isinstance(bottom, int):
            raise QaError(f"blueprint band {identifier} bounds must be integers")
        if top != cursor or bottom <= top or bottom > height:
            raise QaError("blueprint bands must be contiguous and cover the viewport")
        identifiers.add(identifier)
        cursor = bottom
    if cursor != height:
        raise QaError("blueprint bands must cover the complete viewport height")
    return identifiers


def validate_regions(regions, bands, width, height):
    if not isinstance(regions, list) or not regions:
        raise QaError("complete the blueprint regions before visual comparison")
    identifiers = set()
    for region in regions:
        if not isinstance(region, dict):
            raise QaError("every blueprint region must be an object")
        for field in REGION_FIELDS:
            if field not in region:
                raise QaError(f"blueprint region is missing {field}")
        identifier = region["id"]
        if (
            not isinstance(identifier, str)
            or not identifier.strip()
            or identifier in identifiers
        ):
            raise QaError("blueprint region ids must be unique non-empty strings")
        identifiers.add(identifier)
        if region["band"] not in bands:
            raise QaError(f"blueprint region {identifier} references an unknown band")
        bounds = region["bounds"]
        if not isinstance(bounds, dict) or set(bounds) != {
            "x",
            "y",
            "width",
            "height",
        }:
            raise QaError(f"blueprint region {identifier} requires exact bounds")
        if any(not isinstance(value, int) for value in bounds.values()):
            raise QaError(f"blueprint region {identifier} bounds must be integers")
        if (
            bounds["x"] < 0
            or bounds["y"] < 0
            or bounds["width"] <= 0
            or bounds["height"] <= 0
            or bounds["x"] + bounds["width"] > width
            or bounds["y"] + bounds["height"] > height
        ):
            raise QaError(f"blueprint region {identifier} bounds exceed the viewport")
        if region["owner"] not in {"layout", "page", "component"}:
            raise QaError(f"blueprint region {identifier} has an invalid owner")
        for field in ("component", "container"):
            if not isinstance(region[field], str) or not region[field].strip():
                raise QaError(f"blueprint region {identifier} requires {field}")
        if region["dataOwner"] not in {"none", "const", "signal", "store"}:
            raise QaError(f"blueprint region {identifier} has an invalid dataOwner")
        responsive = region["responsive"]
        if not isinstance(responsive, dict) or responsive.get("evidence") not in {
            "observed",
            "inferred",
        }:
            raise QaError(f"blueprint region {identifier} has invalid responsive evidence")
        require_string_list(
            responsive.get("rules"), f"region {identifier} responsive rules"
        )
        require_string_list(region["states"], f"region {identifier} states")
        require_string_list(
            region["accessibility"], f"region {identifier} accessibility"
        )


def validate_theme(theme):
    if not isinstance(theme, dict):
        raise QaError("blueprint theme must be an object")
    for field in THEME_FIELDS:
        require_string_list(theme.get(field), f"theme {field}", allow_empty=True)


def validate_assets(assets):
    if not isinstance(assets, list):
        raise QaError("blueprint assets must be an array")
    for asset in assets:
        if not isinstance(asset, dict):
            raise QaError("every blueprint asset must be an object")
        if asset.get("status") not in {"existing", "supplied", "missing"}:
            raise QaError("asset status must be existing, supplied, or missing")
        for field in ("path", "source"):
            if not isinstance(asset.get(field), str) or not asset[field].strip():
                raise QaError(f"blueprint asset requires {field}")


def validate_candidates(candidates):
    if not isinstance(candidates, list):
        raise QaError("blueprint candidateComponents must be an array")
    for candidate in candidates:
        if not isinstance(candidate, dict):
            raise QaError("every component candidate must be an object")
        if candidate.get("kind") not in {"static", "dynamic"}:
            raise QaError("component candidate kind must be static or dynamic")
        if candidate.get("action") not in {
            "extract-static",
            "keep-inline",
            "future-feature",
        }:
            raise QaError("component candidate action is invalid")
        for field in ("name", "reason"):
            if not isinstance(candidate.get(field), str) or not candidate[field].strip():
                raise QaError(f"component candidate requires {field}")


def load_blueprint(path, width, height):
    try:
        blueprint = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise QaError(f"cannot read blueprint {path}: {error}") from error
    if blueprint.get("viewport") != {"width": width, "height": height}:
        raise QaError("blueprint viewport must match the reference PNG exactly")
    bands = validate_bands(blueprint.get("bands"), height)
    validate_regions(blueprint.get("regions"), bands, width, height)
    validate_theme(blueprint.get("theme"))
    validate_assets(blueprint.get("assets"))
    validate_candidates(blueprint.get("candidateComponents"))
    return blueprint


def image_metrics(reference, rendered, width, top, bottom, channel_delta):
    start = top * width
    end = bottom * width
    count = end - start
    mismatch_count = 0
    total_delta = 0
    maximum_delta = 0
    for reference_pixel, rendered_pixel in zip(reference[start:end], rendered[start:end]):
        deltas = [
            abs(reference_pixel[index] - rendered_pixel[index]) for index in range(3)
        ]
        peak = max(deltas)
        maximum_delta = max(maximum_delta, peak)
        total_delta += sum(deltas)
        if peak > channel_delta:
            mismatch_count += 1
    return {
        "pixelCount": count,
        "mismatchCount": mismatch_count,
        "mismatchRatio": mismatch_count / count,
        "averageChannelDelta": total_delta / (count * 3),
        "maximumChannelDelta": maximum_delta,
    }


def compare_images(
    reference_path,
    rendered_path,
    blueprint_path,
    output,
    channel_delta,
    maximum,
    project=None,
):
    width, height, reference = read_png(reference_path)
    rendered_width, rendered_height, rendered = read_png(rendered_path)
    if (rendered_width, rendered_height) != (width, height):
        raise QaError("rendered PNG dimensions must match the reference viewport")
    blueprint = load_blueprint(blueprint_path, width, height)
    output = require_generated_path(output, project)
    output.mkdir(parents=True, exist_ok=True)
    bands = []
    for band in blueprint["bands"]:
        metrics = image_metrics(
            reference,
            rendered,
            width,
            band["top"],
            band["bottom"],
            channel_delta,
        )
        metrics.update(
            {
                "id": band["id"],
                "top": band["top"],
                "bottom": band["bottom"],
                "passed": metrics["mismatchRatio"] <= maximum,
            }
        )
        bands.append(metrics)
    overall = image_metrics(reference, rendered, width, 0, height, channel_delta)
    passed = overall["mismatchRatio"] <= maximum and all(
        band["passed"] for band in bands
    )
    diff_pixels = []
    for reference_pixel, rendered_pixel in zip(reference, rendered):
        peak = max(
            abs(reference_pixel[index] - rendered_pixel[index]) for index in range(3)
        )
        intensity = peak if peak > channel_delta else 0
        diff_pixels.append((intensity, 0, 0, 255))
    diff_path = output / "diff.png"
    report_path = output / "report.json"
    write_png(diff_path, width, height, diff_pixels)
    report = {
        "passed": passed,
        "viewport": {"width": width, "height": height},
        "thresholds": {
            "channelDelta": channel_delta,
            "maximumMismatchRatio": maximum,
        },
        "overall": overall,
        "bands": bands,
        "artifacts": {
            "diff": str(diff_path),
            "rendered": str(rendered_path.resolve()),
        },
    }
    write_json(report_path, report)
    print(report_path)
    return passed
