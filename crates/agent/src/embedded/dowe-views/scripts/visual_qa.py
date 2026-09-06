#!/usr/bin/env python3

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

from visual_qa_blueprint import (
    compare_images,
    initialize_blueprint,
    load_blueprint,
    require_generated_path,
    write_json,
)
from visual_qa_png import QaError, read_png, write_png


def find_browser(requested):
    if requested:
        candidate = Path(requested)
        if candidate.is_file():
            return str(candidate)
        resolved = shutil.which(requested)
        if resolved:
            return resolved
        raise QaError(f"cannot find browser {requested}")
    for name in (
        "chromium",
        "chromium-browser",
        "google-chrome",
        "google-chrome-stable",
        "microsoft-edge",
    ):
        resolved = shutil.which(name)
        if resolved:
            return resolved
    for candidate in (
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    ):
        if Path(candidate).is_file():
            return candidate
    raise QaError("no supported Chrome or Chromium executable was found")


def validate_local_url(value):
    parsed = urllib.parse.urlparse(value)
    if parsed.scheme != "http" or parsed.hostname not in {
        "127.0.0.1",
        "localhost",
        "::1",
    }:
        raise QaError("visual QA run accepts only a local HTTP URL")
    return value


def wait_for_url(url, process, timeout):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise QaError("dowe dev exited before the local view URL became ready")
        try:
            with urllib.request.urlopen(url, timeout=1) as response:
                if response.status < 500:
                    return
        except (urllib.error.URLError, TimeoutError):
            time.sleep(0.25)
    raise QaError(f"timed out waiting for {url}")


def capture_browser(browser, url, width, height, output, timeout):
    common = [
        browser,
        "--disable-gpu",
        "--hide-scrollbars",
        "--force-device-scale-factor=1",
        f"--window-size={width},{height}",
        f"--screenshot={output}",
        url,
    ]
    failures = []
    for headless in ("--headless=new", "--headless"):
        result = subprocess.run(
            [browser, headless, *common[1:]],
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
        if result.returncode == 0 and output.is_file():
            return
        failures.append(result.stderr.strip() or result.stdout.strip())
    raise QaError("browser capture failed: " + " | ".join(failures))


def run_visual_qa(args):
    width, height, _ = read_png(args.reference)
    load_blueprint(args.blueprint, width, height)
    output = require_generated_path(args.output, args.project)
    output.mkdir(parents=True, exist_ok=True)
    rendered = output / "rendered.png"
    browser = find_browser(args.browser)
    dowe = shutil.which(args.dowe) or (
        args.dowe if Path(args.dowe).is_file() else None
    )
    if dowe is None:
        raise QaError(f"cannot find Dowe executable {args.dowe}")
    url = validate_local_url(args.url)
    log_path = output / "dowe-dev.log"
    process = None
    with log_path.open("w", encoding="utf-8") as log:
        try:
            process = subprocess.Popen(
                [dowe, "dev", "--target", "web"],
                cwd=args.project,
                stdout=log,
                stderr=subprocess.STDOUT,
                text=True,
            )
            wait_for_url(url, process, args.timeout)
            capture_browser(browser, url, width, height, rendered, args.timeout)
        finally:
            if process is not None and process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
    return compare_images(
        args.reference,
        rendered,
        args.blueprint,
        output,
        args.channel_delta,
        args.maximum_mismatch,
        args.project,
    )


def complete_test_blueprint(path):
    blueprint = json.loads(path.read_text(encoding="utf-8"))
    blueprint["regions"] = [
        {
            "id": "sample",
            "band": "full",
            "bounds": {"x": 0, "y": 0, "width": 2, "height": 2},
            "owner": "page",
            "component": "Section",
            "container": "Grid",
            "dataOwner": "const",
            "responsive": {
                "evidence": "observed",
                "rules": [
                    "reference: sample remains visible",
                    "xs: sample remains visible",
                    "md: sample remains visible",
                ],
            },
            "states": ["default"],
            "accessibility": ["visible title"],
        }
    ]
    write_json(path, blueprint)


def self_test():
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        generated = root / ".dowe" / "visual-qa" / "self-test"
        previous = Path.cwd()
        try:
            os.chdir(root)
            reference = generated / "reference.png"
            rendered = generated / "rendered.png"
            blueprint = generated / "blueprint.json"
            pixels = [
                (0, 0, 0, 255),
                (255, 255, 255, 255),
                (0, 0, 0, 255),
                (255, 255, 255, 255),
            ]
            changed = list(pixels)
            changed[0] = (255, 0, 0, 255)
            write_png(reference, 2, 2, pixels)
            write_png(rendered, 2, 2, changed)
            initialize_blueprint(reference, blueprint)
            complete_test_blueprint(blueprint)
            passed = compare_images(reference, rendered, blueprint, generated, 16, 0.30)
            if not passed:
                raise QaError("visual QA self-test expected the comparison to pass")
            strict_passed = compare_images(
                reference, rendered, blueprint, generated, 16, 0.20
            )
            if strict_passed:
                raise QaError("visual QA self-test expected the strict comparison to fail")
            report = json.loads(
                (generated / "report.json").read_text(encoding="utf-8")
            )
            if report["overall"]["mismatchRatio"] != 0.25:
                raise QaError("visual QA self-test produced an unexpected mismatch ratio")
            width, height, _ = read_png(generated / "diff.png")
            if (width, height) != (2, 2):
                raise QaError("visual QA self-test produced an invalid diff")
            try:
                initialize_blueprint(reference, root / "outside.json")
            except QaError:
                pass
            else:
                raise QaError("visual QA self-test allowed output outside .dowe")
        finally:
            os.chdir(previous)
    print("visual QA self-test passed")


def add_threshold_arguments(parser):
    parser.add_argument("--channel-delta", type=int, default=16)
    parser.add_argument("--maximum-mismatch", type=float, default=0.08)


def parse_args():
    parser = argparse.ArgumentParser(prog="visual_qa.py")
    commands = parser.add_subparsers(dest="command", required=True)
    initialize = commands.add_parser("init")
    initialize.add_argument("--reference", type=Path, required=True)
    initialize.add_argument("--output", type=Path, required=True)
    compare = commands.add_parser("compare")
    compare.add_argument("--reference", type=Path, required=True)
    compare.add_argument("--rendered", type=Path, required=True)
    compare.add_argument("--blueprint", type=Path, required=True)
    compare.add_argument("--output", type=Path, required=True)
    add_threshold_arguments(compare)
    run = commands.add_parser("run")
    run.add_argument("--project", type=Path, required=True)
    run.add_argument("--reference", type=Path, required=True)
    run.add_argument("--blueprint", type=Path, required=True)
    run.add_argument("--output", type=Path, required=True)
    run.add_argument("--url", default="http://127.0.0.1:7655/")
    run.add_argument("--browser")
    run.add_argument("--dowe", default="dowe")
    run.add_argument("--timeout", type=float, default=30.0)
    add_threshold_arguments(run)
    commands.add_parser("self-test")
    return parser.parse_args()


def validate_thresholds(channel_delta, maximum_mismatch):
    if not 0 <= channel_delta <= 255:
        raise QaError("channel delta must be between 0 and 255")
    if not 0 <= maximum_mismatch <= 1:
        raise QaError("maximum mismatch must be between 0 and 1")


def main():
    args = parse_args()
    try:
        if args.command == "init":
            initialize_blueprint(args.reference, args.output)
            return 0
        if args.command == "self-test":
            self_test()
            return 0
        validate_thresholds(args.channel_delta, args.maximum_mismatch)
        if args.command == "compare":
            passed = compare_images(
                args.reference,
                args.rendered,
                args.blueprint,
                args.output,
                args.channel_delta,
                args.maximum_mismatch,
            )
        else:
            passed = run_visual_qa(args)
        return 0 if passed else 1
    except (QaError, OSError, subprocess.SubprocessError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
