#!/usr/bin/env python3
"""Remove AI watermarks from scene images — paint-over with nearby texture synthesis.

Usage:
    python tools/remove_watermark.py <input> <output> [x0 y0 x1 y1 ...]

If no regions specified, defaults to bottom-right corner (typical watermark location).
Regions are pixel coordinates: x0 y0 x1 y1 (repeated for multiple regions).
"""
import sys
from pathlib import Path
from PIL import Image, ImageFilter, ImageDraw


def inpaint(img: Image.Image, regions: list[tuple[int, int, int, int]]) -> Image.Image:
    """Inpaint watermark regions using nearby texture synthesis."""
    result = img.copy()
    w, h = img.size

    for x0, y0, x1, y1 in regions:
        # Sample texture from a clean nearby area
        if y0 < h // 3:  # top region
            src_x, src_y = x0, min(h - (y1 - y0), y1 + 80)
        elif x0 > w // 2:  # bottom-right
            src_x, src_y = max(0, x0 - 150), max(0, y0 - 150)
        else:  # bottom-left
            src_x, src_y = min(w - (x1 - x0), x1 + 50), max(0, y0 - 80)

        # Clamp
        src_x = max(0, min(src_x, w - (x1 - x0)))
        src_y = max(0, min(src_y, h - (y1 - y0)))

        # Clone patch
        patch = img.crop((src_x, src_y, src_x + (x1 - x0), src_y + (y1 - y0)))
        patch = patch.filter(ImageFilter.GaussianBlur(radius=10))

        # Create feathered mask
        pw, ph = patch.size
        mask = Image.new("L", (pw, ph), 0)
        draw = ImageDraw.Draw(mask)
        inner_pad = 25
        draw.rectangle([inner_pad, inner_pad, pw - inner_pad, ph - inner_pad], fill=255)
        for i in range(inner_pad):
            alpha = int(255 * (i / inner_pad))
            draw.rectangle([i, i, pw - i - 1, ph - i - 1], outline=alpha)
        mask = mask.filter(ImageFilter.GaussianBlur(radius=inner_pad))

        result.paste(patch, (x0, y0), mask)

    return result


def main():
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(1)

    inp = Path(sys.argv[1])
    out = Path(sys.argv[2])
    nums = [int(x) for x in sys.argv[3:]]
    if len(nums) >= 4 and len(nums) % 4 == 0:
        regions = [(nums[i], nums[i + 1], nums[i + 2], nums[i + 3]) for i in range(0, len(nums), 4)]
    else:
        # Default: bottom-right corner
        img = Image.open(inp)
        w, h = img.size
        regions = [(w - 350, h - 120, w, h)]

    img = Image.open(inp)
    w, h = img.size
    result = inpaint(img, regions)
    if (w, h) != (1536, 1024):
        result = result.resize((1536, 1024), Image.LANCZOS)
    result.save(out, "PNG", optimize=True)
    print(f"{inp.name}: {w}x{h} -> {out.name} 1536x1024")


if __name__ == "__main__":
    main()
