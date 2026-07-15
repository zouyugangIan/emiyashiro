#!/usr/bin/env python3
"""Build Sakura's standalone runtime frames from generated JK module sheets."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
MODULE_ROOT = ROOT / "assets/images/characters/sakura_jk/modules"
BASE_SOURCE = ROOT / "assets/images/characters/sakura_jk/base_movement_v3.png"
BASE_FRAME_ROOT = ROOT / "assets/images/characters/sakura_jk/base_frames"
FRAME_ROOT = ROOT / "assets/images/characters/sakura_attack/frames"
OUTPUT_CELL = 256
INNER_CELL = 248
BASELINE_BOTTOM = 232


@dataclass(frozen=True)
class Module:
    name: str
    columns: int
    source_rows: int
    runtime_rows: int


MODULES = (
    Module("ground_light", 4, 2, 5),
    Module("heavy", 4, 2, 5),
    Module("air_combo", 4, 2, 5),
    Module("mobility", 3, 2, 4),
    Module("ninjutsu_projectiles", 4, 2, 4),
    Module("ultimate", 4, 2, 3),
    Module("weapon_projection", 3, 2, 4),
)


def alpha_border_pixels(image: Image.Image, threshold: int = 16) -> int:
    alpha = image.getchannel("A")
    width, height = image.size
    return sum(
        alpha.getpixel((x, y)) > threshold
        for x in range(width)
        for y in (0, height - 1)
    ) + sum(
        alpha.getpixel((x, y)) > threshold
        for y in range(height)
        for x in (0, width - 1)
    )


def chroma_green_pixels(image: Image.Image, threshold: int = 16) -> int:
    pixels = image.load()
    return sum(
        pixels[x, y][3] > threshold
        and pixels[x, y][1] > 180
        and pixels[x, y][0] < 80
        and pixels[x, y][2] < 120
        for y in range(image.height)
        for x in range(image.width)
    )


def remove_neighbor_cell_bleed(
    frame: Image.Image, bleed_side: str, threshold: int = 1
) -> Image.Image:
    """Drop disconnected fragments entering from the neighboring source cell."""
    alpha = frame.getchannel("A")
    width, height = frame.size
    pixels = alpha.load()
    visited = bytearray(width * height)
    components: list[tuple[list[int], bool]] = []
    edge_band = max(2, width // 12)

    for y in range(height):
        for x in range(width):
            start = y * width + x
            if visited[start] or pixels[x, y] <= threshold:
                continue

            visited[start] = 1
            stack = [start]
            component: list[int] = []
            reaches_bleed_edge = False
            while stack:
                index = stack.pop()
                component.append(index)
                current_x = index % width
                current_y = index // width
                if bleed_side == "right":
                    reaches_bleed_edge |= current_x >= width - edge_band
                else:
                    reaches_bleed_edge |= current_x < edge_band
                for neighbor_x, neighbor_y in (
                    (current_x - 1, current_y),
                    (current_x + 1, current_y),
                    (current_x, current_y - 1),
                    (current_x, current_y + 1),
                ):
                    if not (0 <= neighbor_x < width and 0 <= neighbor_y < height):
                        continue
                    neighbor = neighbor_y * width + neighbor_x
                    if visited[neighbor] or pixels[neighbor_x, neighbor_y] <= threshold:
                        continue
                    visited[neighbor] = 1
                    stack.append(neighbor)
            components.append((component, reaches_bleed_edge))

    if not components:
        return frame

    largest = max(range(len(components)), key=lambda index: len(components[index][0]))
    for index, (component, reaches_bleed_edge) in enumerate(components):
        if not reaches_bleed_edge or index == largest:
            continue
        for pixel_index in component:
            pixels[pixel_index % width, pixel_index // width] = 0

    cleaned = frame.copy()
    cleaned.putalpha(alpha)
    return cleaned


def remove_chroma_residue(frame: Image.Image) -> Image.Image:
    """Remove isolated green-screen pixels without touching cyan/purple VFX."""
    cleaned = frame.copy()
    pixels = cleaned.load()
    for y in range(cleaned.height):
        for x in range(cleaned.width):
            red, green, blue, alpha = pixels[x, y]
            if alpha == 0 or (
                alpha > 16 and green > 180 and red < 80 and blue < 120
            ):
                pixels[x, y] = (0, 0, 0, 0)
    return cleaned


def fit_runtime_cell(frame: Image.Image) -> Image.Image:
    frame = remove_chroma_residue(frame)
    frame.thumbnail((INNER_CELL, INNER_CELL), Image.Resampling.LANCZOS)
    output = Image.new("RGBA", (OUTPUT_CELL, OUTPUT_CELL), (0, 0, 0, 0))
    inset_x = (OUTPUT_CELL - frame.width) // 2
    inset_y = (OUTPUT_CELL - frame.height) // 2
    output.alpha_composite(frame, (inset_x, inset_y))
    return remove_chroma_residue(output)


def align_alpha_bottom(frame: Image.Image, target_bottom: int) -> Image.Image:
    """Keep state changes stable; physics, rather than source padding, moves Sakura."""
    bbox = frame.getchannel("A").getbbox()
    if bbox is None:
        return frame

    offset_y = target_bottom - bbox[3]
    if offset_y == 0:
        return frame

    aligned = Image.new("RGBA", frame.size, (0, 0, 0, 0))
    aligned.alpha_composite(frame, (0, offset_y))
    return aligned


def split_source_cells(source: Path, columns: int, source_rows: int) -> list[Image.Image]:
    with Image.open(source) as opened:
        image = opened.convert("RGBA")

    if image.width % columns or image.height % source_rows:
        raise ValueError(
            f"{source} must be an exact {columns}x{source_rows} source grid, "
            f"got {image.size}"
        )
    cell_width = image.width // columns
    cell_height = image.height // source_rows

    frames = []
    for frame_index in range(columns * source_rows):
        column = frame_index % columns
        row = frame_index // columns
        frame = image.crop(
            (
                column * cell_width,
                row * cell_height,
                (column + 1) * cell_width,
                (row + 1) * cell_height,
            )
        )
        if alpha_border_pixels(frame) > 0:
            raise ValueError(
                f"{source} frame {frame_index + 1} touches a source-cell edge; "
                "the source sheet would produce a cut animation frame"
            )
        frames.append(fit_runtime_cell(frame))
    return frames


def split_module(module: Module) -> dict[str, object]:
    source = MODULE_ROOT / f"{module.name}.png"
    source_frames = split_source_cells(source, module.columns, module.source_rows)

    module_dir = FRAME_ROOT / module.name
    module_dir.mkdir(parents=True, exist_ok=True)
    entries = []
    for runtime_row in range(1, module.runtime_rows + 1):
        for frame_number, frame in enumerate(source_frames, start=1):
            filename = f"r{runtime_row:02d}_f{frame_number:02d}.png"
            output = module_dir / filename
            frame.save(output, optimize=True)
            alpha = frame.getchannel("A")
            entries.append(
                {
                    "row": runtime_row,
                    "frame": frame_number,
                    "file": str(output.relative_to(ROOT)),
                    "bbox": list(alpha.getbbox()) if alpha.getbbox() else None,
                    "border_alpha_pixels": alpha_border_pixels(frame),
                    "chroma_green_pixels": chroma_green_pixels(frame),
                }
            )

    return {
        "id": module.name,
        "source": str(source.relative_to(ROOT)),
        "source_facing": "right",
        "runtime_facing": "right",
        "columns": len(source_frames),
        "rows": module.runtime_rows,
        "cell_size": [OUTPUT_CELL, OUTPUT_CELL],
        "frames": entries,
    }


def build_base_frames() -> list[dict[str, object]]:
    frames = split_source_cells(BASE_SOURCE, columns=4, source_rows=2)
    BASE_FRAME_ROOT.mkdir(parents=True, exist_ok=True)
    entries = []
    for frame_number, frame in enumerate(frames, start=1):
        frame = align_alpha_bottom(frame, BASELINE_BOTTOM)
        output = BASE_FRAME_ROOT / f"f{frame_number:02d}.png"
        frame.save(output, optimize=True)
        alpha = frame.getchannel("A")
        entries.append(
            {
                "frame": frame_number,
                "file": str(output.relative_to(ROOT)),
                "bbox": list(alpha.getbbox()) if alpha.getbbox() else None,
                "border_alpha_pixels": alpha_border_pixels(frame),
                "chroma_green_pixels": chroma_green_pixels(frame),
            }
        )
    return entries


def main() -> None:
    manifest = {
        "character": "Sakura 2P JK redesign",
        "cell_size": [OUTPUT_CELL, OUTPUT_CELL],
        "generation": "standalone-image-sequence",
        "runtime_facing": "right",
        "base_source": str(BASE_SOURCE.relative_to(ROOT)),
        "base_source_facing": "right",
        "base_frames": build_base_frames(),
        "sheets": [split_module(module) for module in MODULES],
    }
    FRAME_ROOT.mkdir(parents=True, exist_ok=True)
    manifest_path = FRAME_ROOT / "manifest.json"
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )

    frames = sum(len(sheet["frames"]) for sheet in manifest["sheets"])
    edge_contacts = sum(
        frame["border_alpha_pixels"] > 0
        for sheet in manifest["sheets"]
        for frame in sheet["frames"]
    )
    chroma_residue_frames = sum(
        frame["chroma_green_pixels"] > 0
        for frame in manifest["base_frames"]
    ) + sum(
        frame["chroma_green_pixels"] > 0
        for sheet in manifest["sheets"]
        for frame in sheet["frames"]
    )
    print(f"built {len(MODULES)} Sakura JK modules, {frames} runtime frames")
    print(f"built {len(manifest['base_frames'])} dedicated base-movement frames")
    print(f"edge-contact frames: {edge_contacts}")
    print(f"chroma-residue frames: {chroma_residue_frames}")


if __name__ == "__main__":
    main()
