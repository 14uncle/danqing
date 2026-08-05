#!/usr/bin/env python3
"""Remove AI watermarks from scene images — aggressive paint-over method."""
import sys
from pathlib import Path
from PIL import Image, ImageFilter, ImageDraw

REPO_ROOT = Path(__file__).resolve().parent.parent
ASSETS_DIR = REPO_ROOT / "assets" / "scenes"


def inpaint(img: Image.Image, regions: list[tuple[int, int, int, int]]) -> Image.Image:
    """Inpaint watermark regions using nearby texture synthesis."""
    result = img.copy()
    w, h = img.size

    for x0, y0, x1, y1 in regions:
        # Sample texture from a clean nearby area
        if y0 < h // 3:  # top region
            src_x, src_y = x0, min(h - (y1-y0), y1 + 80)
        elif x0 > w // 2:  # bottom-right
            src_x, src_y = max(0, x0 - 120), max(0, y0 - 120)
        else:  # bottom-left
            src_x, src_y = min(w - (x1-x0), x1 + 50), max(0, y0 - 80)

        # Clamp
        src_x = max(0, min(src_x, w - (x1-x0)))
        src_y = max(0, min(src_y, h - (y1-y0)))

        # Clone patch
        patch = img.crop((src_x, src_y, src_x + (x1-x0), src_y + (y1-y0)))
        patch = patch.filter(ImageFilter.GaussianBlur(radius=8))

        # Create feathered mask
        pw, ph = patch.size
        mask = Image.new("L", (pw, ph), 0)
        draw = ImageDraw.Draw(mask)
        inner_pad = 20
        draw.rectangle([inner_pad, inner_pad, pw-inner_pad, ph-inner_pad], fill=255)
        for i in range(inner_pad):
            alpha = int(255 * (i / inner_pad))
            draw.rectangle([i, i, pw-i-1, ph-i-1], outline=alpha)
        mask = mask.filter(ImageFilter.GaussianBlur(radius=inner_pad))

        result.paste(patch, (x0, y0), mask)

    return result


def main():
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    # 山: bottom-right "元宝 AI生成"
    mountain = Image.open(ASSETS_DIR / "moutain_ai_2.png")
    w, h = mountain.size
    mountain_clean = inpaint(mountain, [
        (w-300, h-100, w, h),  # bottom-right
    ])
    mountain_clean = mountain_clean.resize((1536, 1024), Image.LANCZOS)
    mountain_clean.save(ASSETS_DIR / "mountain_ai_clean.png", "PNG", optimize=True)
    print(f"山: {w}x{h} -> 1536x1024")

    # 火: bottom-right "元宝 AI生成"
    bonfire = Image.open(ASSETS_DIR / "bonfire_ai_3.png")
    w2, h2 = bonfire.size
    bonfire_clean = inpaint(bonfire, [
        (w2-450, h2-180, w2, h2),  # bottom-right (扩大区域)
    ])
    bonfire_clean = bonfire_clean.resize((1536, 1024), Image.LANCZOS)
    bonfire_clean.save(ASSETS_DIR / "bonfire_ai_clean.png", "PNG", optimize=True)
    print(f"火: {w2}x{h2} -> 1536x1024")


if __name__ == "__main__":
    main()
