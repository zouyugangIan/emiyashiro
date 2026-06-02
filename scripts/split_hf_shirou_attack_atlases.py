#!/usr/bin/env python3
"""Split HF Shirou attack atlases into deterministic 256px frame PNGs.

The game consumes the atlases directly, but this tool makes the first design
step reproducible: every row and frame can be inspected as a standalone file.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SPEC = ROOT / "assets/images/characters/reference/hf_shirou_attack_rows.json"
DEFAULT_OUT = ROOT / "tmp/hf_shirou_attack_frames"


def load_spec(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def alpha_bbox(crop: Image.Image) -> list[int] | None:
    alpha = crop.getchannel("A")
    bbox = alpha.getbbox()
    if bbox is None:
        return None

    left, top, right, bottom = bbox
    return [left, top, right - left, bottom - top]


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


def split_sheet(
    root: Path,
    output_root: Path,
    sheet: dict[str, Any],
    cell_size: tuple[int, int],
    dry_run: bool,
) -> dict[str, Any]:
    source = root / sheet["source"]
    columns = int(sheet["columns"])
    rows = sheet["rows"]
    expected_size = (columns * cell_size[0], len(rows) * cell_size[1])

    with Image.open(source) as image:
        image = image.convert("RGBA")
        if image.size != expected_size:
            raise ValueError(
                f"{source} has size {image.size}, expected {expected_size} "
                f"from {columns} columns and {len(rows)} rows"
            )

        sheet_dir = output_root / sheet["id"]
        if not dry_run:
            sheet_dir.mkdir(parents=True, exist_ok=True)

        frame_entries: list[dict[str, Any]] = []
        for row_index, row_spec in enumerate(rows):
            row_number = int(row_spec["row"])
            if row_number != row_index + 1:
                raise ValueError(f"{sheet['id']} row order mismatch at row {row_number}")

            for column in range(columns):
                left = column * cell_size[0]
                top = row_index * cell_size[1]
                box = (left, top, left + cell_size[0], top + cell_size[1])
                crop = image.crop(box)
                frame_name = f"r{row_number:02d}_f{column + 1:02d}.png"
                frame_path = sheet_dir / frame_name

                if not dry_run:
                    crop.save(frame_path)

                frame_entries.append(
                    {
                        "row": row_number,
                        "frame": column + 1,
                        "file": str(frame_path.relative_to(root)),
                        "bbox": alpha_bbox(crop),
                        "border_alpha_pixels": alpha_border_pixels(crop),
                    }
                )

    return {
        "id": sheet["id"],
        "source": sheet["source"],
        "columns": columns,
        "rows": len(rows),
        "cell_size": list(cell_size),
        "frames": frame_entries,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, default=DEFAULT_SPEC)
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate atlas dimensions and row order without writing frames.",
    )
    parser.add_argument(
        "--fail-on-edge-contact",
        action="store_true",
        help="Exit non-zero if any opaque pixels touch a 256px frame border.",
    )
    args = parser.parse_args()

    spec_path = args.spec if args.spec.is_absolute() else ROOT / args.spec
    output_root = args.out if args.out.is_absolute() else ROOT / args.out
    spec = load_spec(spec_path)
    cell_size = tuple(spec["cell_size"])
    if len(cell_size) != 2:
        raise ValueError("cell_size must contain width and height")

    split_manifest = {
        "character": spec["character"],
        "source_spec": str(spec_path.relative_to(ROOT)),
        "cell_size": list(cell_size),
        "sheets": [],
    }

    total_frames = 0
    edge_contacts: list[tuple[str, int, int, int]] = []
    for sheet in spec["sheets"]:
        entry = split_sheet(ROOT, output_root, sheet, cell_size, args.dry_run)
        split_manifest["sheets"].append(entry)
        total_frames += len(entry["frames"])
        for frame in entry["frames"]:
            border_pixels = int(frame["border_alpha_pixels"])
            if border_pixels > 0:
                edge_contacts.append(
                    (entry["id"], int(frame["row"]), int(frame["frame"]), border_pixels)
                )

    if not args.dry_run:
        output_root.mkdir(parents=True, exist_ok=True)
        manifest_path = output_root / "manifest.json"
        with manifest_path.open("w", encoding="utf-8") as handle:
            json.dump(split_manifest, handle, ensure_ascii=False, indent=2)

    mode = "validated" if args.dry_run else "split"
    print(f"{mode} {len(split_manifest['sheets'])} sheets, {total_frames} frames")
    print(f"edge-contact frames: {len(edge_contacts)}")
    if not args.dry_run:
        print(f"wrote {output_root.relative_to(ROOT)}")
    if edge_contacts and args.fail_on_edge_contact:
        details = ", ".join(
            f"{sheet} r{row:02d} f{frame:02d} ({pixels}px)"
            for sheet, row, frame, pixels in edge_contacts[:10]
        )
        raise SystemExit(f"opaque pixels touched frame borders: {details}")


if __name__ == "__main__":
    main()
