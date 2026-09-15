#!/usr/bin/env python3
"""Check the exact runtime rectangles and render review sheets for repaired 1P assets."""

import json
from pathlib import Path

from PIL import Image, ImageDraw

from audit_hf_shirou_attack_atlases import alpha_border_pixels, checkerboard


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/images/characters/shirou_repaired"
OUT = ROOT / "tmp/shirou_repaired_review"


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    report = {}
    for name in ("locomotion", "ground_attacks"):
        spec = json.loads((SOURCE / f"{name}.json").read_text())
        image = Image.open(SOURCE / f"{name}.png").convert("RGBA")
        assert image.size == tuple(spec["size"]), f"{name}: image size differs from metadata"
        assert len(spec["frames"]) == 16
        review = checkerboard((1024, 1024))
        frames = []
        for index, frame in enumerate(spec["frames"]):
            left, top, right, bottom = frame["rect"]
            assert 0 <= left < right <= image.width
            assert 0 <= top < bottom <= image.height
            crop = image.crop((left, top, right, bottom))
            border = alpha_border_pixels(crop)
            assert border == 0, f"{name} frame {index}: opaque pixels cut by frame edge"
            assert crop.getchannel("A").getbbox(), f"{name} frame {index}: empty frame"
            # Reproduce the runtime scale and body origin on a 256px canvas.
            scale = spec["logical_scale"]
            display = crop.resize((round(crop.width * scale), round(crop.height * scale)))
            ox, oy = frame["origin"]
            x = (index % 4) * 256 + round(128 - (ox - left) * scale)
            y = (index // 4) * 256 + round(226 - (oy - top) * scale)
            review.alpha_composite(display, (x, y))
            draw = ImageDraw.Draw(review)
            draw.line(((index % 4) * 256, (index // 4) * 256 + 226,
                       (index % 4 + 1) * 256 - 1, (index // 4) * 256 + 226),
                      fill=(80, 160, 180, 255))
            draw.text(((index % 4) * 256 + 8, (index // 4) * 256 + 8), str(index),
                      fill=(240, 240, 240, 255))
            frames.append({"index": index, "border_pixels": border})
        review.save(OUT / f"{name}.png")
        report[name] = frames
    (OUT / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"32 authored frames checked: no empty frames or cut frame edges. Review: {OUT}")


if __name__ == "__main__":
    main()
