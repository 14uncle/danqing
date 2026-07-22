#!/usr/bin/env python3
"""Export danqing Phase 1 background assets.

Dependency:
    pip install Pillow

Usage:
    python tools/export-background.py

Outputs:
    assets/background/gradient.png   # main background with baked soft gradient
    assets/background/glow.png       # additive radial glow overlay
    assets/background/noise.png      # subtle fine-grain noise texture
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = REPO_ROOT / "assets" / "background"

# Brand accent #3B82F6, same as LightTheme::accent().
ACCENT = (59, 130, 246)

# Gradient endpoints: light blue-white to a clearly blue-tinted base.
# The bottom needs enough saturation/darkness for translucent glass cards
# (white at ~0.72 alpha) to read as glass rather than flat white.
TOP = (240, 245, 253)
BOTTOM = (186, 208, 244)

# Canvas size for the gradient / glow. 1024 is large enough to look smooth
# when scaled with Cover on typical window sizes.
SIZE = 1024

# Noise texture size. Smaller is fine because it is stretched with low opacity.
NOISE_SIZE = 512


def lerp(a: int, b: int, t: float) -> int:
    return int(a + (b - a) * t)


def blend_pixel(base: tuple[int, int, int], overlay: tuple[int, int, int], alpha: float) -> tuple[int, int, int]:
    """Alpha-composite an opaque RGB overlay onto an opaque RGB base."""
    return (
        lerp(base[0], overlay[0], alpha),
        lerp(base[1], overlay[1], alpha),
        lerp(base[2], overlay[2], alpha),
    )


def generate_gradient() -> Image.Image:
    """Create a soft vertical gradient with a baked radial glow."""
    img = Image.new("RGB", (SIZE, SIZE), TOP)
    draw = ImageDraw.Draw(img)

    for y in range(SIZE):
        t = y / (SIZE - 1)
        r = lerp(TOP[0], BOTTOM[0], t)
        g = lerp(TOP[1], BOTTOM[1], t)
        b = lerp(TOP[2], BOTTOM[2], t)
        draw.line([(0, y), (SIZE, y)], fill=(r, g, b))

    return img


def generate_glow() -> Image.Image:
    """Create a radial glow overlay with transparency.

    The glow is centered slightly above the middle so it feels like a light
    source behind the UI rather than a flat center spot.
    """
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    center_x = SIZE * 0.5
    center_y = SIZE * 0.42
    max_radius = SIZE * 0.75

    # Draw many concentric circles with decreasing opacity for a smooth radial
    # falloff. Pillow's radial gradient support is limited, so we approximate.
    steps = 240
    for i in range(steps, 0, -1):
        t = i / steps
        radius = max_radius * t
        # Ease-out falloff: opacity peaks at center and drops quickly.
        # Peak alpha 64: strong enough for the brand glow to survive the
        # 0.25 overlay opacity and remain visible behind glass cards.
        alpha = int(64 * (1.0 - t**1.5))
        if alpha <= 1:
            continue
        color = (*ACCENT, alpha)
        bbox = [
            center_x - radius,
            center_y - radius,
            center_x + radius,
            center_y + radius,
        ]
        draw.ellipse(bbox, fill=color)

    # Slight blur to remove banding from the concentric approximation.
    return img.filter(ImageFilter.GaussianBlur(radius=SIZE * 0.02))


def generate_noise() -> Image.Image:
    """Create a subtle fine-grain noise texture."""
    img = Image.new("L", (NOISE_SIZE, NOISE_SIZE), 128)
    draw = ImageDraw.Draw(img)

    # Deterministic simple PRNG; not cryptographic, just reproducible visuals.
    state = 0x1234_5678
    for y in range(NOISE_SIZE):
        for x in range(NOISE_SIZE):
            state = state * 1_103_515_245 + 12_345
            v = (state >> 24) & 0xFF
            # Keep noise very subtle: map 0..255 to 220..235.
            mapped = 220 + (v % 16)
            draw.point((x, y), fill=mapped)

    # Blur slightly so the noise feels like film grain rather than sharp dots.
    return img.filter(ImageFilter.GaussianBlur(radius=0.5))


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    gradient = generate_gradient()
    gradient_path = OUT_DIR / "gradient.png"
    gradient.save(gradient_path, "PNG", optimize=True)
    print(f"Exported {gradient_path} ({gradient.width}x{gradient.height})")

    glow = generate_glow()
    glow_path = OUT_DIR / "glow.png"
    glow.save(glow_path, "PNG", optimize=True)
    print(f"Exported {glow_path} ({glow.width}x{glow.height})")

    noise = generate_noise()
    noise_path = OUT_DIR / "noise.png"
    noise.save(noise_path, "PNG", optimize=True)
    print(f"Exported {noise_path} ({noise.width}x{noise.height})")


if __name__ == "__main__":
    main()
