#!/usr/bin/env python3
"""Render the deterministic 8-tile atlas used by Windheart Sky City."""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "assets" / "images" / "levels" / "sky_city_tiles.png"
GRID = 32
SCALE = 4


def box(tile: int, left: int, top: int, right: int, bottom: int) -> tuple[int, int, int, int]:
    x = tile * GRID
    left, top = max(0, left), max(0, top)
    right, bottom = min(GRID - 1, right), min(GRID - 1, bottom)
    return tuple(value * SCALE for value in (x + left, top, x + right, bottom))


def point(tile: int, x: int, y: int) -> tuple[int, int]:
    return (
        (tile * GRID + min(GRID - 1, max(0, x))) * SCALE,
        min(GRID - 1, max(0, y)) * SCALE,
    )


def main() -> None:
    image = Image.new("RGBA", (8 * GRID * SCALE, GRID * SCALE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image, "RGBA")

    stone_palettes = [
        ((126, 120, 102, 255), (94, 92, 82, 255), (168, 157, 127, 210)),
        ((137, 126, 103, 255), (101, 93, 79, 255), (179, 164, 131, 205)),
        ((116, 119, 106, 255), (84, 91, 85, 255), (158, 158, 132, 205)),
        ((144, 134, 111, 255), (106, 99, 84, 255), (185, 170, 138, 205)),
    ]
    for tile, (base, shadow, light) in enumerate(stone_palettes):
        draw.rectangle(box(tile, 0, 0, 32, 32), fill=base)
        draw.line([point(tile, 0, 10), point(tile, 32, 10)], fill=shadow, width=SCALE)
        draw.line([point(tile, 0, 22), point(tile, 32, 22)], fill=shadow, width=SCALE)
        shift = (tile * 7) % 13
        draw.line([point(tile, 11 + shift, 0), point(tile, 8 + shift, 10)], fill=shadow, width=SCALE)
        draw.line([point(tile, 23 - shift // 2, 10), point(tile, 26 - shift // 2, 22)], fill=shadow, width=SCALE)
        draw.line([point(tile, 7 + shift // 2, 22), point(tile, 5 + shift // 2, 32)], fill=shadow, width=SCALE)
        draw.line([point(tile, 2, 2), point(tile, 28, 2)], fill=light, width=SCALE)

    for tile, tint in [(4, (235, 244, 248, 255)), (5, (224, 239, 246, 255))]:
        draw.rectangle(box(tile, 0, 0, 32, 32), fill=tint)
        for cx, cy, radius, alpha in [(7, 11, 8, 165), (18, 8, 10, 190), (28, 14, 8, 160), (14, 27, 15, 135)]:
            draw.ellipse(box(tile, cx - radius, cy - radius, cx + radius, cy + radius), fill=(255, 255, 255, alpha))
        draw.line([point(tile, 1, 30), point(tile, 31, 30)], fill=(169, 207, 220, 120), width=SCALE)

    # Hazard and wind tiles are translucent so the terrain underneath remains readable.
    draw.rectangle(box(6, 0, 0, 32, 32), fill=(83, 47, 35, 100))
    for offset in (1, 11, 21):
        draw.polygon(
            [point(6, offset, 31), point(6, offset + 5, 3), point(6, offset + 10, 31)],
            fill=(236, 154, 42, 235),
        )
        draw.line([point(6, offset + 5, 7), point(6, offset + 5, 27)], fill=(255, 229, 132, 230), width=SCALE)

    for y, alpha in [(5, 170), (14, 130), (23, 95)]:
        draw.arc(box(7, 4, y - 4, 28, y + 6), 195, 345, fill=(196, 247, 255, alpha), width=2 * SCALE)
    draw.polygon([point(7, 16, 2), point(7, 10, 11), point(7, 22, 11)], fill=(216, 252, 255, 180))

    image = image.resize((8 * GRID, GRID), Image.Resampling.LANCZOS)
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    image.save(OUTPUT, optimize=True)
    print(OUTPUT)


if __name__ == "__main__":
    main()
