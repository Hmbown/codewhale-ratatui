#!/usr/bin/env python3
"""Compose README boards from real gallery SVG buffers, using only the stdlib.

    cargo run --example gallery -- --svg target/readme-buffers
    python3 tools/render-gallery.py target/readme-buffers assets/readme
    python3 tools/render-gallery.py target/readme-buffers assets/readme --check

Every input entry appears once in a board for each requested profile. The
default profiles are dark-truecolor and light-truecolor; --profiles accepts
any exported profiles. Whale states have their own boards, and the complete
action sheet stands alone. SVGs embed the buffer drawing directly and need no
JavaScript, foreignObject, remote image or font resource to display on GitHub.
"""

import argparse
import copy
import json
import os
from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET

SVG = "http://www.w3.org/2000/svg"
ET.register_namespace("", SVG)
WIDTH = 1040
MARGIN = 48
GAP = 24
CARD_PAD = 20
CARD_HEADER = 48
TOP = 154
MAX_HEIGHT = 2200
MONO = "'DejaVu Sans Mono','Cascadia Mono','SFMono-Regular',Consolas,'Liberation Mono',monospace"
SANS = "-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif"
GROUPS = {
    "components": ("Sessions and fleets", "Messages, tools, workspaces and parallel agents"),
    "foundation": ("The Codewhale language", "Depth, rules, marks, hints and terminal chrome"),
    "input": ("Input and selection", "Editable fields, forms, lists and focused choices"),
    "chrome": ("Navigation and controls", "Headings, tabs, toggles and keyboard maps"),
    "display": ("Work and receipts", "Diffs, trees, progress, approvals and durable outcomes"),
    "motion": ("Motion and feedback", "Spinners, notifications and calm transitions"),
    "whales": ("A whale with a job", "Session state, attention, completion and the pod"),
    "whale-actions": ("Every whale action", "The complete v2 state vocabulary, in terminal cells"),
}
PROFILES = ["dark-truecolor", "dark-graphite", "light-truecolor", "dark-256", "light-256",
            "ansi-16", "unknown-ground", "no-color", "ascii"]


def element(parent, tag, attrs=None, text=None):
    node = ET.SubElement(parent, f"{{{SVG}}}{tag}", attrs or {})
    if text is not None:
        node.text = text
    return node


def classify(name):
    if name == "whale-actions":
        return "whale-actions"
    if name.startswith("whale-"):
        return "whales"
    if name in {"key-hints", "status-marks", "depth", "horizon", "icons"}:
        return "foundation"
    if re.search(r"dialog|sheet", name):
        return "foundation"
    if re.search(r"message|composer|tool-card|fleet", name):
        return "components"
    if re.search(r"input|form|fuzzy|picker|list|empty", name):
        return "input"
    if re.search(r"heading|tabs|toggle|segmented|keymap|setting", name):
        return "chrome"
    if re.search(r"receipt|diff|tree|progress|approval|review|count-bar", name):
        return "display"
    if re.search(r"motion|spinner|toast", name):
        return "motion"
    return "components"


def palette(profile):
    if profile.startswith("light"):
        return {"bg": "#f2f5f9", "card": "#ffffff", "edge": "#d6dfe9",
                "fg": "#142235", "muted": "#52647a", "accent": "#0b48bb"}
    return {"bg": "#070e1a", "card": "#0c1625", "edge": "#24374d",
            "fg": "#e9f0f8", "muted": "#8b9eb5", "accent": "#38a5e9"}


def read_inputs(source, profiles):
    exported = {}
    for profile in profiles:
        suffix = f".{profile}.svg"
        exported[profile] = {
            path.name[:-len(suffix)]: path
            for path in sorted(source.glob(f"*{suffix}"))
        }
        if not exported[profile]:
            raise ValueError(f"no {profile} SVG buffers found in {source}")
    manifest = source / "manifest.json"
    if manifest.exists():
        data = json.loads(manifest.read_text(encoding="utf-8"))
        entries = data.get("entries", data) if isinstance(data, dict) else data
        names = [entry["name"] if isinstance(entry, dict) else entry for entry in entries]
    else:
        names = sorted(exported[profiles[0]])
    if len(names) != len(set(names)):
        raise ValueError("duplicate gallery names in manifest")
    expected = set(names)
    for profile, paths in exported.items():
        missing = expected - paths.keys()
        extra = paths.keys() - expected
        if missing or extra:
            raise ValueError(f"{profile}: missing {sorted(missing)}, extra {sorted(extra)}")
    return names, exported


def load_buffer(path):
    root = ET.fromstring(path.read_text(encoding="utf-8"))
    if root.tag != f"{{{SVG}}}svg":
        raise ValueError(f"{path} is not an SVG buffer")
    # Exported buffers are intentionally self-contained. Refuse an unrelated
    # image or active document accidentally placed in the input directory.
    allowed = {"svg", "title", "desc", "rect", "g", "text", "path"}
    for node in root.iter():
        if node.tag.removeprefix(f"{{{SVG}}}") not in allowed:
            raise ValueError(f"unexpected element {node.tag} in {path}")
        if any(key.lower().startswith("on") or "href" in key.lower() for key in node.attrib):
            raise ValueError(f"active or external SVG content in {path}")
    _, _, width, height = (float(v) for v in root.attrib["viewBox"].split())
    if width <= 0 or height <= 0:
        raise ValueError(f"invalid buffer dimensions in {path}")
    return root, width, height


def layout(items):
    available = WIDTH - 2 * MARGIN
    column = (available - GAP) / 2
    pages = []
    page = []
    x, y, row_height = MARGIN, TOP, 0
    for name, buffer, width, height in items:
        card_width = column if width + 2 * CARD_PAD <= column else available
        scale = min(1.0, (card_width - 2 * CARD_PAD) / width)
        card_height = CARD_HEADER + height * scale + CARD_PAD
        if x + card_width > WIDTH - MARGIN + 0.1:
            x, y, row_height = MARGIN, y + row_height + GAP, 0
        if page and y + card_height + MARGIN > MAX_HEIGHT:
            pages.append(page)
            page = []
            x, y, row_height = MARGIN, TOP, 0
        page.append((name, buffer, width, height, x, y, card_width, card_height, scale))
        x += card_width + GAP
        row_height = max(row_height, card_height)
    if page:
        pages.append(page)
    return pages


def board(group, profile, page, number, total):
    colors = palette(profile)
    title, subtitle = GROUPS[group]
    height = int(max(item[5] + item[7] for item in page) + MARGIN)
    root = ET.Element(f"{{{SVG}}}svg", {
        "width": str(WIDTH), "height": str(height), "viewBox": f"0 0 {WIDTH} {height}",
        "role": "img", "aria-labelledby": "title desc",
    })
    suffix = f" · {number}/{total}" if total > 1 else ""
    element(root, "title", {"id": "title"}, f"{title} — {profile}{suffix}")
    element(root, "desc", {"id": "desc"},
            "Actual ratatui buffer renders: " + ", ".join(item[0] for item in page))
    element(root, "rect", {"width": "100%", "height": "100%", "fill": colors["bg"]})
    element(root, "rect", {"x": str(MARGIN), "y": "32", "width": "36", "height": "4",
                          "rx": "2", "fill": colors["accent"]})
    element(root, "text", {"x": str(MARGIN), "y": "72", "fill": colors["fg"],
                          "font-family": SANS, "font-size": "30", "font-weight": "650"}, title + suffix)
    element(root, "text", {"x": str(MARGIN), "y": "102", "fill": colors["muted"],
                          "font-family": SANS, "font-size": "17"}, subtitle)
    element(root, "text", {"x": str(WIDTH - MARGIN), "y": "72", "fill": colors["accent"],
                          "font-family": MONO, "font-size": "14", "text-anchor": "end"}, profile)
    element(root, "text", {"x": str(WIDTH - MARGIN), "y": "102", "fill": colors["muted"],
                          "font-family": SANS, "font-size": "13", "text-anchor": "end"},
            "Real terminal buffers · 10 × 20 px cells")
    for name, buffer, width, height, x, y, card_width, card_height, scale in page:
        card = element(root, "g")
        element(card, "rect", {
            "x": f"{x:g}", "y": f"{y:g}", "width": f"{card_width:g}", "height": f"{card_height:g}",
            "rx": "10", "fill": colors["card"], "stroke": colors["edge"],
        })
        element(card, "text", {"x": f"{x + CARD_PAD:g}", "y": f"{y + 28:g}",
                               "fill": colors["fg"], "font-family": MONO, "font-size": "14"}, name)
        element(card, "text", {"x": f"{x + card_width - CARD_PAD:g}", "y": f"{y + 28:g}",
                               "fill": colors["muted"], "font-family": MONO, "font-size": "12",
                               "text-anchor": "end"}, f"{int(width / 10)} × {int(height / 20)}")
        drawing = copy.deepcopy(buffer)
        drawing_x = x + (card_width - width * scale) / 2 if group in {"whales", "whale-actions"} else x + CARD_PAD
        drawing.attrib.update({"x": f"{drawing_x:g}", "y": f"{y + CARD_HEADER:g}",
                               "width": f"{width * scale:g}", "height": f"{height * scale:g}"})
        card.append(drawing)
    return ET.tostring(root, encoding="unicode", xml_declaration=False) + "\n"


def profile_comparison(source):
    """The same actual state-mark buffer in all nine terminal profiles."""
    colors = palette("dark-truecolor")
    buffers = [(profile, *load_buffer(source / f"status-marks.{profile}.svg")) for profile in PROFILES]
    column = (WIDTH - 2 * MARGIN - 2 * GAP) / 3
    scale = min(1.0, min((column - 2 * CARD_PAD) / width for _, _, width, _ in buffers))
    card_height = CARD_HEADER + max(height * scale for _, _, _, height in buffers) + CARD_PAD
    height = int(TOP + 3 * card_height + 2 * GAP + MARGIN)
    root = ET.Element(f"{{{SVG}}}svg", {"width": str(WIDTH), "height": str(height),
        "viewBox": f"0 0 {WIDTH} {height}", "role": "img", "aria-labelledby": "title desc"})
    element(root, "title", {"id": "title"}, "One state vocabulary across nine terminal profiles")
    element(root, "desc", {"id": "desc"}, "Actual status-mark buffers: " + ", ".join(PROFILES))
    element(root, "rect", {"width": "100%", "height": "100%", "fill": colors["bg"]})
    element(root, "rect", {"x": str(MARGIN), "y": "32", "width": "36", "height": "4",
                          "rx": "2", "fill": colors["accent"]})
    element(root, "text", {"x": str(MARGIN), "y": "72", "fill": colors["fg"],
        "font-family": SANS, "font-size": "30", "font-weight": "650"}, "Every terminal has a language")
    element(root, "text", {"x": str(MARGIN), "y": "102", "fill": colors["muted"],
        "font-family": SANS, "font-size": "17"}, "One state vocabulary across nine terminal capability profiles")
    for i, (profile, buffer, width, body_height) in enumerate(buffers):
        x = MARGIN + (i % 3) * (column + GAP)
        y = TOP + (i // 3) * (card_height + GAP)
        profile_colors = palette(profile)
        card = element(root, "g")
        element(card, "rect", {"x": f"{x:g}", "y": f"{y:g}", "width": f"{column:g}",
            "height": f"{card_height:g}", "rx": "10", "fill": profile_colors["card"],
            "stroke": profile_colors["edge"]})
        element(card, "text", {"x": f"{x + CARD_PAD:g}", "y": f"{y + 28:g}",
            "fill": profile_colors["fg"], "font-family": MONO, "font-size": "14"}, profile)
        drawing = copy.deepcopy(buffer)
        drawing.attrib.update({"x": f"{x + CARD_PAD:g}", "y": f"{y + CARD_HEADER:g}",
            "width": f"{width * scale:g}", "height": f"{body_height * scale:g}"})
        card.append(drawing)
    return ET.tostring(root, encoding="unicode", xml_declaration=False) + "\n"


def generate(source, profiles):
    names, exported = read_inputs(source, profiles)
    outputs, index = {}, []
    for profile in profiles:
        grouped = {group: [] for group in GROUPS}
        for name in names:
            buffer, width, height = load_buffer(exported[profile][name])
            grouped[classify(name)].append((name, buffer, width, height))
        represented = []
        for group, items in grouped.items():
            pages = layout(items)
            for number, page in enumerate(pages, 1):
                suffix = f"-{number}" if len(pages) > 1 else ""
                filename = f"{group}.{profile}{suffix}.svg"
                outputs[filename] = board(group, profile, page, number, len(pages))
                page_names = [item[0] for item in page]
                represented.extend(page_names)
                index.append({"file": filename, "profile": profile, "group": group, "entries": page_names})
        if sorted(represented) != sorted(names):
            raise ValueError(f"{profile}: gallery coverage mismatch")
    outputs["profile-comparison.svg"] = profile_comparison(source)
    index.append({"file": "profile-comparison.svg", "profile": "all", "group": "profiles",
                  "entries": ["status-marks"], "profiles": PROFILES})
    outputs["index.json"] = json.dumps({"entries": names, "boards": index}, indent=2) + "\n"
    return outputs, len(names), index


def readme_gallery(readme, destination, index):
    """Replace the bounded generated slot while preserving all authored text."""
    original = readme.read_text(encoding="utf-8")
    start, end = "<!-- gallery:start -->", "<!-- gallery:end -->"
    if original.count(start) != 1 or original.count(end) != 1:
        raise ValueError(f"{readme}: expected exactly one gallery:start / gallery:end marker pair")
    begin, finish = original.index(start), original.index(end)
    if finish < begin:
        raise ValueError(f"{readme}: gallery markers are reversed")

    def embed(item):
        title = GROUPS.get(item["group"], ("Terminal profile comparison", ""))[0]
        profile = item["profile"].replace("-", " ")
        alt = f"{title} — {profile}" if item["profile"] != "all" else title
        path = Path(os.path.relpath(destination / item["file"], readme.parent)).as_posix()
        return f"![{alt}](<{path}>)"

    primary = [item for item in index
               if item["profile"] != "all" and not item["profile"].startswith("light")]
    families = list(dict.fromkeys(item["group"] for item in primary))
    comparison = [item for item in index if item["file"] == "profile-comparison.svg"]
    titles = [GROUPS[family][0] for family in families]
    if comparison:
        titles.append("Terminal profiles")
    navigation = " · ".join(f"[{title}](#{re.sub(r'[^a-z0-9]+', '-', title.lower()).strip('-')})"
                            for title in titles)
    lines = [start, "", "Generated from the real ratatui buffers. Every catalogue entry is shown below.", ""]
    if navigation:
        lines.extend(["Jump to: " + navigation, ""])
    for family in families:
        lines.extend(["### " + GROUPS[family][0], ""])
        lines.extend(embed(item) + "\n" for item in primary if item["group"] == family)
    light = [item for item in index if item["profile"].startswith("light")]
    if light:
        lines.extend(["<details>", "<summary>Light theme</summary>", ""])
        lines.extend(embed(item) + "\n" for item in light)
        lines.extend(["</details>", ""])
    if comparison:
        lines.extend(["### Terminal profiles", ""])
        lines.extend(embed(item) + "\n" for item in comparison)
    lines.append(end)
    generated = "\n".join(lines)
    return original[:begin] + generated + original[finish + len(end):]


def previous_owned_files(destination):
    """Only the previous index can authorize removing an obsolete board."""
    path = destination / "index.json"
    if not path.exists():
        return set()
    previous = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(previous, dict) or not isinstance(previous.get("boards"), list):
        raise ValueError(f"{path}: expected a generated board index")
    names = set()
    for board in previous["boards"]:
        name = board.get("file") if isinstance(board, dict) else None
        # Strict basenames also reject Windows separators/drives, hidden files,
        # parent traversal and non-SVG artifacts. Never recurse or glob-delete.
        if not isinstance(name, str) or not re.fullmatch(r"[a-z0-9][a-z0-9.-]*\.svg", name) or ".." in name:
            raise ValueError(f"{path}: unsafe generated board basename {name!r}")
        names.add(name)
    return names


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("source", type=Path, help="directory written by gallery --svg")
    parser.add_argument("destination", type=Path, help="generated README asset directory")
    parser.add_argument("--profiles", nargs="+", default=["dark-truecolor", "light-truecolor"])
    parser.add_argument("--check", action="store_true", help="compare generated bytes without writing")
    parser.add_argument("--readme", type=Path, help="update the gallery:start / gallery:end slot in this README")
    args = parser.parse_args()
    if args.source.resolve() == args.destination.resolve():
        parser.error("source and destination must be separate directories; use target/readme-buffers and assets/readme")
    if len(set(args.profiles)) != len(args.profiles):
        parser.error("duplicate profiles")
    try:
        outputs, count, index = generate(args.source, args.profiles)
        readme = readme_gallery(args.readme, args.destination, index) if args.readme else None
        obsolete = previous_owned_files(args.destination) - outputs.keys()
        obsolete = sorted(name for name in obsolete
                          if (args.destination / name).exists() or (args.destination / name).is_symlink())
        if args.check:
            stale = [name for name, content in outputs.items()
                     if not (args.destination / name).exists()
                     or (args.destination / name).read_text(encoding="utf-8") != content]
            if readme is not None and args.readme.read_text(encoding="utf-8") != readme:
                stale.append(f"{args.readme} gallery slot")
            stale.extend(f"obsolete indexed board {name}" for name in obsolete)
            if stale:
                raise ValueError("stale or missing assets: " + ", ".join(stale))
            print(f"Verified {len(index)} boards covering all {count} entries × {len(args.profiles)} profiles")
        else:
            args.destination.mkdir(parents=True, exist_ok=True)
            for name in obsolete:
                (args.destination / name).unlink()
            for name, content in outputs.items():
                (args.destination / name).write_bytes(content.encode("utf-8"))
            if readme is not None:
                args.readme.write_bytes(readme.encode("utf-8"))
            print(f"Wrote {len(index)} boards covering all {count} entries × {len(args.profiles)} profiles")
    except (OSError, ValueError, KeyError, ET.ParseError) as error:
        print(f"render-gallery: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
