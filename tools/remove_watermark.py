#!/usr/bin/env python3
"""Remove AI watermarks from forest images — aggressive method."""
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
        # For top-left: sample from (x0, y1+50) area
        # For bottom-right: sample from (x0-100, y0-100) area
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
        # Inner rectangle fully opaque
        inner_pad = 20
        draw.rectangle([inner_pad, inner_pad, pw-inner_pad, ph-inner_pad], fill=255)

        # Feather edges with gradient
        for i in range(inner_pad):
            alpha = int(255 * (i / inner_pad))
            draw.rectangle([i, i, pw-i-1, ph-i-1], outline=alpha)

        mask = mask.filter(ImageFilter.GaussianBlur(radius=inner_pad))

        # Paste with mask
        result.paste(patch, (x0, y0), mask)

    return result


def main():
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    # 即梦: top-left + bottom-right (wider regions)
    jimeng = Image.open(ASSETS_DIR / "forest_jimeng.png")
    w, h = jimeng.size
    jimeng_clean = inpaint(jimeng, [
        (0, 0, 250, 100),          # top-left "AI生成"
        (w-420, h-150, w, h),      # bottom-right "即梦AI"
    ])
    jimeng_clean = jimeng_clean.resize((1536, 1024), Image.LANCZOS)
    jimeng_clean.save(ASSETS_DIR / "forest_jimeng_clean.png", "PNG", optimize=True)
    print(f"即梦: {w}x{h} -> 1536x1024, inpainted")

    # 元宝: bottom-right only
    yuanbao = Image.open(ASSETS_DIR / "forest_yanbao.png")
    w2, h2 = yuanbao.size
    yuanbao_clean = inpaint(yuanbao, [
        (w2-320, h2-120, w2, h2),  # bottom-right "元宝 AI生成"
    ])
    yuanbao_clean = yuanbao_clean.resize((1536, 1024), Image.LANCZOS)
    yuanbao_clean.save(ASSETS_DIR / "forest_yuanbao_clean.png", "PNG", optimize=True)
    print(f"元宝: {w2}x{h2} -> 1536x1024, inpainted")


if __name__ == "__main__":
    main()
