#!/usr/bin/env python3
"""Render the deterministic, original parallax panorama used by the LDtk level."""

from __future__ import annotations

import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "assets" / "images" / "levels" / "sky_city_panorama.png"
SCALE = 2
W = H = 1024


def s(value: float) -> int:
    return round(value * SCALE)


def cloud(layer: Image.Image, x: float, y: float, width: float, alpha: int) -> None:
    draw = ImageDraw.Draw(layer, "RGBA")
    color = (250, 248, 232, alpha)
    lobes = [
        (0.04, 0.37, 0.34, 0.34),
        (0.22, 0.16, 0.38, 0.46),
        (0.47, 0.05, 0.34, 0.52),
        (0.65, 0.24, 0.31, 0.40),
        (0.14, 0.43, 0.72, 0.31),
    ]
    for lx, ly, lw, lh in lobes:
        draw.ellipse(
            (s(x + width * lx), s(y + width * ly), s(x + width * (lx + lw)), s(y + width * (ly + lh))),
            fill=color,
        )


def tower(draw: ImageDraw.ImageDraw, x: float, base_y: float, height: float, warm: bool = False) -> None:
    stone = (142, 151, 144, 220) if not warm else (164, 151, 120, 230)
    shadow = (87, 117, 124, 210)
    roof = (105, 104, 91, 235)
    draw.rounded_rectangle(
        (s(x - 25), s(base_y - height), s(x + 25), s(base_y)),
        radius=s(7),
        fill=stone,
    )
    draw.polygon(
        [(s(x - 37), s(base_y - height + 3)), (s(x), s(base_y - height - 37)), (s(x + 37), s(base_y - height + 3))],
        fill=roof,
    )
    for wy in range(round(base_y - height + 26), round(base_y - 18), 34):
        draw.rounded_rectangle(
            (s(x - 7), s(wy), s(x + 7), s(wy + 17)),
            radius=s(6),
            fill=shadow,
        )
    draw.rectangle((s(x - 31), s(base_y - 8), s(x + 31), s(base_y + 4)), fill=(106, 126, 120, 235))


def island(layer: Image.Image, x: float, y: float, width: float, height: float, towers: int) -> None:
    draw = ImageDraw.Draw(layer, "RGBA")
    rock = (80, 94, 91, 225)
    rock_light = (116, 119, 103, 205)
    grass = (91, 132, 79, 235)
    draw.polygon(
        [
            (s(x - width * 0.49), s(y)),
            (s(x + width * 0.49), s(y)),
            (s(x + width * 0.32), s(y + height * 0.52)),
            (s(x + width * 0.08), s(y + height)),
            (s(x - width * 0.18), s(y + height * 0.72)),
            (s(x - width * 0.38), s(y + height * 0.42)),
        ],
        fill=rock,
    )
    draw.polygon(
        [
            (s(x - width * 0.12), s(y + height * 0.08)),
            (s(x + width * 0.34), s(y + height * 0.08)),
            (s(x + width * 0.12), s(y + height * 0.82)),
        ],
        fill=rock_light,
    )
    draw.ellipse(
        (s(x - width * 0.52), s(y - 17), s(x + width * 0.52), s(y + 25)),
        fill=grass,
    )
    for index in range(towers):
        offset = (index - (towers - 1) / 2) * min(82, width / max(1, towers))
        tower(draw, x + offset, y - 2, 86 + (index % 2) * 42, warm=index % 2 == 1)


def main() -> None:
    canvas = Image.new("RGBA", (s(W), s(H)), (0, 0, 0, 0))
    pixels = canvas.load()
    top = (69, 154, 218)
    middle = (152, 207, 229)
    bottom = (250, 224, 174)
    for y in range(s(H)):
        t = y / (s(H) - 1)
        if t < 0.68:
            q = t / 0.68
            c = tuple(round(top[i] * (1 - q) + middle[i] * q) for i in range(3))
        else:
            q = (t - 0.68) / 0.32
            c = tuple(round(middle[i] * (1 - q) + bottom[i] * q) for i in range(3))
        for x in range(s(W)):
            pixels[x, y] = (*c, 255)

    glow = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow, "RGBA")
    for radius in range(240, 10, -12):
        alpha = max(1, round((240 - radius) / 240 * 8 + 2))
        glow_draw.ellipse(
            (s(700 - radius), s(185 - radius), s(700 + radius), s(185 + radius)),
            fill=(255, 244, 186, alpha),
        )
    glow = glow.filter(ImageFilter.GaussianBlur(s(18)))
    canvas.alpha_composite(glow)

    far = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    far_draw = ImageDraw.Draw(far, "RGBA")
    # Distant city silhouettes keep both horizontal edges visually compatible.
    for index, x in enumerate(range(-30, 1080, 72)):
        h = 70 + (index * 37) % 120
        far_draw.rectangle((s(x), s(650 - h), s(x + 38), s(650)), fill=(85, 141, 158, 72))
        far_draw.polygon(
            [(s(x - 8), s(650 - h)), (s(x + 19), s(620 - h)), (s(x + 46), s(650 - h))],
            fill=(75, 125, 145, 72),
        )
    for cx, cy, cw, ca in [(-90, 145, 330, 110), (190, 270, 270, 95), (570, 105, 360, 105), (870, 300, 310, 100)]:
        cloud(far, cx, cy, cw, ca)
    far = far.filter(ImageFilter.GaussianBlur(s(3.2)))
    canvas.alpha_composite(far)

    mid = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    island(mid, 176, 620, 320, 240, 3)
    island(mid, 710, 535, 390, 300, 4)
    island(mid, 985, 690, 245, 190, 2)
    mid_draw = ImageDraw.Draw(mid, "RGBA")
    # Waterfalls and suspended bridges.
    for wx, wy, wh in [(112, 614, 315), (657, 530, 390), (760, 535, 330), (970, 688, 260)]:
        mid_draw.rounded_rectangle((s(wx - 8), s(wy), s(wx + 8), s(wy + wh)), radius=s(8), fill=(160, 227, 245, 120))
        mid_draw.rectangle((s(wx - 2), s(wy), s(wx + 2), s(wy + wh)), fill=(240, 252, 250, 105))
    mid_draw.line((s(310), s(600), s(520), s(550)), fill=(111, 113, 101, 190), width=s(9))
    mid_draw.line((s(310), s(588), s(520), s(538)), fill=(170, 161, 129, 170), width=s(3))
    canvas.alpha_composite(mid)

    near = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    for cx, cy, cw, ca in [(-120, 760, 460, 165), (210, 805, 390, 145), (610, 770, 440, 155), (900, 825, 390, 150)]:
        cloud(near, cx, cy, cw, ca)
    near = near.filter(ImageFilter.GaussianBlur(s(1.5)))
    canvas.alpha_composite(near)

    detail = ImageDraw.Draw(canvas, "RGBA")
    for index in range(58):
        x = (index * 173 + 47) % W
        y = 70 + (index * 89) % 540
        radius = 1 + index % 3
        detail.ellipse((s(x - radius), s(y - radius), s(x + radius), s(y + radius)), fill=(255, 248, 194, 90 + index % 4 * 25))

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    canvas.resize((W, H), Image.Resampling.LANCZOS).convert("RGBA").save(OUTPUT, optimize=True)
    print(OUTPUT)


if __name__ == "__main__":
    main()
