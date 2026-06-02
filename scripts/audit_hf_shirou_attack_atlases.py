#!/usr/bin/env python3
"""Generate visual QA contact sheets and alpha diagnostics for HF Shirou atlases."""

from __future__ import annotations

import argparse
import json
from collections import deque
from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SPEC = ROOT / "assets/images/characters/reference/hf_shirou_attack_rows.json"
DEFAULT_OUT = ROOT / "tmp/hf_shirou_attack_audit"


def load_spec(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def checkerboard(size: tuple[int, int], tile: int = 16) -> Image.Image:
    width, height = size
    image = Image.new("RGBA", size, (0, 0, 0, 255))
    draw = ImageDraw.Draw(image)
    for y in range(0, height, tile):
        for x in range(0, width, tile):
            shade = 52 if ((x // tile) + (y // tile)) % 2 == 0 else 34
            draw.rectangle((x, y, x + tile - 1, y + tile - 1), fill=(shade, shade, shade, 255))
    return image


def image_data(image: Image.Image):
    if hasattr(image, "get_flattened_data"):
        return image.get_flattened_data()
    return image.getdata()


def alpha_border_pixels(crop: Image.Image, threshold: int = 16) -> int:
    alpha = crop.getchannel("A")
    width, height = crop.size
    count = 0
    for x in range(width):
        for y in (0, height - 1):
            if alpha.getpixel((x, y)) > threshold:
                count += 1
    for y in range(height):
        for x in (0, width - 1):
            if alpha.getpixel((x, y)) > threshold:
                count += 1
    return count


def component_holes(crop: Image.Image, threshold: int = 16, min_component_pixels: int = 80) -> int:
    alpha = crop.getchannel("A")
    width, height = crop.size
    solid = alpha.point(lambda value: 255 if value > threshold else 0)
    solid_pixels = solid.load()
    visited: set[tuple[int, int]] = set()
    total_holes = 0

    for start_y in range(height):
        for start_x in range(width):
            if solid_pixels[start_x, start_y] == 0 or (start_x, start_y) in visited:
                continue

            queue = deque([(start_x, start_y)])
            visited.add((start_x, start_y))
            component: list[tuple[int, int]] = []
            min_x = max_x = start_x
            min_y = max_y = start_y

            while queue:
                x, y = queue.popleft()
                component.append((x, y))
                min_x = min(min_x, x)
                max_x = max(max_x, x)
                min_y = min(min_y, y)
                max_y = max(max_y, y)

                for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                    if (
                        0 <= nx < width
                        and 0 <= ny < height
                        and solid_pixels[nx, ny] != 0
                        and (nx, ny) not in visited
                    ):
                        visited.add((nx, ny))
                        queue.append((nx, ny))

            if len(component) < min_component_pixels:
                continue

            box_width = max_x - min_x + 1
            box_height = max_y - min_y + 1
            local_solid = [[False] * box_width for _ in range(box_height)]
            for x, y in component:
                local_solid[y - min_y][x - min_x] = True

            outside: set[tuple[int, int]] = set()
            flood = deque()
            for x in range(box_width):
                for y in (0, box_height - 1):
                    if not local_solid[y][x] and (x, y) not in outside:
                        outside.add((x, y))
                        flood.append((x, y))
            for y in range(box_height):
                for x in (0, box_width - 1):
                    if not local_solid[y][x] and (x, y) not in outside:
                        outside.add((x, y))
                        flood.append((x, y))

            while flood:
                x, y = flood.popleft()
                for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                    if (
                        0 <= nx < box_width
                        and 0 <= ny < box_height
                        and not local_solid[ny][nx]
                        and (nx, ny) not in outside
                    ):
                        outside.add((nx, ny))
                        flood.append((nx, ny))

            for y in range(box_height):
                for x in range(box_width):
                    if not local_solid[y][x] and (x, y) not in outside:
                        total_holes += 1

    return total_holes


def chroma_residue_pixels(crop: Image.Image) -> int:
    rgba = crop.convert("RGBA")
    count = 0
    for red, green, blue, alpha in image_data(rgba):
        if alpha <= 16:
            continue
        if green > 210 and red < 90 and blue < 90:
            count += 1
    return count


def paste_on_checker(crop: Image.Image, scale: int = 1) -> Image.Image:
    background = checkerboard(crop.size)
    background.alpha_composite(crop)
    if scale != 1:
        background = background.resize((crop.width * scale, crop.height * scale), Image.Resampling.NEAREST)
    return background


def audit_sheet(
    root: Path,
    output_root: Path,
    sheet: dict[str, Any],
    cell_size: tuple[int, int],
    hole_warning_threshold: int,
    green_warning_threshold: int,
) -> dict[str, Any]:
    source = root / sheet["source"]
    columns = int(sheet["columns"])
    rows = sheet["rows"]
    expected_size = (columns * cell_size[0], len(rows) * cell_size[1])

    with Image.open(source) as image:
        image = image.convert("RGBA")
        if image.size != expected_size:
            raise ValueError(f"{source} has size {image.size}, expected {expected_size}")

        contact = Image.new("RGBA", image.size, (0, 0, 0, 255))
        frame_entries = []
        for row_index, row_spec in enumerate(rows):
            row_number = int(row_spec["row"])
            for column in range(columns):
                left = column * cell_size[0]
                top = row_index * cell_size[1]
                crop = image.crop((left, top, left + cell_size[0], top + cell_size[1]))
                contact.alpha_composite(paste_on_checker(crop), (left, top))
                alpha = crop.getchannel("A")
                bbox = alpha.getbbox()
                frame_entries.append(
                    {
                        "row": row_number,
                        "frame": column + 1,
                        "bbox": list(bbox) if bbox else None,
                        "opaque_pixels": sum(1 for value in image_data(alpha) if value > 16),
                        "border_alpha_pixels": alpha_border_pixels(crop),
                        "enclosed_alpha_hole_pixels": component_holes(crop),
                        "green_chroma_residue_pixels": chroma_residue_pixels(crop),
                    }
                )

        output_root.mkdir(parents=True, exist_ok=True)
        contact_path = output_root / f"{sheet['id']}_checker_contact.png"
        contact.save(contact_path)

    edge_contact_frames = [
        frame
        for frame in frame_entries
        if frame["border_alpha_pixels"] > 0
    ]
    alpha_hole_warning_frames = [
        frame
        for frame in frame_entries
        if frame["enclosed_alpha_hole_pixels"] > hole_warning_threshold
    ]
    green_residue_warning_frames = [
        frame
        for frame in frame_entries
        if frame["green_chroma_residue_pixels"] > green_warning_threshold
    ]

    return {
        "id": sheet["id"],
        "source": sheet["source"],
        "contact_sheet": str(contact_path.relative_to(root)),
        "frames": frame_entries,
        "edge_contact_frames": edge_contact_frames,
        "alpha_hole_warning_frames": alpha_hole_warning_frames,
        "green_residue_warning_frames": green_residue_warning_frames,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, default=DEFAULT_SPEC)
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument(
        "--fail-on-issues",
        action="store_true",
        help="Exit non-zero if diagnostics find frame-edge alpha contact or material green residue.",
    )
    parser.add_argument("--hole-warning-threshold", type=int, default=20)
    parser.add_argument("--green-warning-threshold", type=int, default=20)
    args = parser.parse_args()

    spec_path = args.spec if args.spec.is_absolute() else ROOT / args.spec
    output_root = args.out if args.out.is_absolute() else ROOT / args.out
    spec = load_spec(spec_path)
    cell_size = tuple(spec["cell_size"])
    if len(cell_size) != 2:
        raise ValueError("cell_size must contain width and height")

    report = {
        "character": spec["character"],
        "source_spec": str(spec_path.relative_to(ROOT)),
        "cell_size": list(cell_size),
        "sheets": [],
    }
    total_frames = 0
    total_edge_contact_frames = 0
    total_alpha_hole_warning_frames = 0
    total_green_residue_warning_frames = 0
    for sheet in spec["sheets"]:
        entry = audit_sheet(
            ROOT,
            output_root,
            sheet,
            cell_size,
            args.hole_warning_threshold,
            args.green_warning_threshold,
        )
        report["sheets"].append(entry)
        total_frames += len(entry["frames"])
        total_edge_contact_frames += len(entry["edge_contact_frames"])
        total_alpha_hole_warning_frames += len(entry["alpha_hole_warning_frames"])
        total_green_residue_warning_frames += len(entry["green_residue_warning_frames"])

    output_root.mkdir(parents=True, exist_ok=True)
    report_path = output_root / "report.json"
    with report_path.open("w", encoding="utf-8") as handle:
        json.dump(report, handle, ensure_ascii=False, indent=2)

    print(f"audited {len(report['sheets'])} sheets, {total_frames} frames")
    print(f"edge-contact frames: {total_edge_contact_frames}")
    print(f"alpha-hole warning frames: {total_alpha_hole_warning_frames}")
    print(f"green-residue warning frames: {total_green_residue_warning_frames}")
    print(f"wrote {report_path.relative_to(ROOT)}")

    if args.fail_on_issues and (total_edge_contact_frames or total_green_residue_warning_frames):
        raise SystemExit("attack atlas QA found diagnostic issue frames")


if __name__ == "__main__":
    main()
