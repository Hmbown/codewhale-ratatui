#!/usr/bin/env python3
"""Verify the README animation against the current actual-buffer export.

No raster dependencies are needed for CI: a changed SVG frame invalidates
the recorded source hash, and an edited animation invalidates its file hash.
"""
import hashlib
import json
from pathlib import Path
import sys


def check(source, gif):
    manifest = (source / "manifest.json").read_bytes()
    frames = json.loads(manifest)
    if len(frames) != 80:
        raise ValueError("expected 80 exported habitat frames")
    digest = hashlib.sha256(manifest)
    for index, frame in enumerate(frames):
        expected = {"file": f"frame-{index:03}.svg", "elapsed_ms": 6000 + index * 100,
                    "profile": "dark-truecolor", "width": 112, "height": 30}
        if frame != expected:
            raise ValueError("unexpected frame manifest")
        digest.update(frame["file"].encode())
        digest.update(b"\0")
        digest.update((source / frame["file"]).read_bytes())
    expected = {"source_sha256": digest.hexdigest(), "gif_sha256": hashlib.sha256(gif.read_bytes()).hexdigest(),
                "frames": 80, "width": 1120, "height": 600, "frame_ms": 100}
    if json.loads(Path(str(gif) + ".json").read_text()) != expected:
        raise ValueError("habitat animation is stale; regenerate actual frames and render-habitat.cjs")
    print("Habitat animation matches all 80 current terminal-buffer frames")


if __name__ == "__main__":
    try:
        check(Path(sys.argv[1] if len(sys.argv) > 1 else "target/habitat-frames"),
              Path(sys.argv[2] if len(sys.argv) > 2 else "assets/readme/habitat-motion.gif"))
    except (OSError, ValueError, KeyError) as error:
        sys.exit(str(error))
