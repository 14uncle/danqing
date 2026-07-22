#!/usr/bin/env python3
"""Export the danqing logo into PNG sizes and a multi-resolution ICO.

This script re-implements the same geometry as assets/logo/logo.svg using Pillow
so that it does not depend on system libraries such as Cairo.

Dependency:
    pip install Pillow

Usage:
    python tools/export-logo.py

Outputs:
    assets/logo/logo_16.png
    assets/logo/logo_24.png
    assets/logo/logo_32.png
    assets/logo/logo_48.png
    assets/logo/logo_256.png
    assets/logo/logo.ico
"""

from io import BytesIO
from pathlib import Path

from PIL import Image, ImageDraw

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = REPO_ROOT / "assets" / "logo"

PNG_SIZES = [16, 24, 32, 48, 256]
ICO_SIZES = [16, 24, 32, 48, 256]

# Same colors as assets/logo/logo.svg: jade-teal frame + brand cinnabar drop.
# Cinnabar is a brand-only color; it is not part of the LightTheme token set.
ACCENT = (15, 118, 110, 255)
CINNABAR = (227, 66, 52, 255)
FILL = (255, 255, 255, int(255 * 0.85))

# Geometry expressed in the 256x256 design coordinate space.
FRAME_X = 42
FRAME_Y = 42
FRAME_SIZE = 172
FRAME_RADIUS = 46
STROKE_WIDTH = 26
DOT_CX = 200
DOT_CY = 200
DOT_R = 33


def rounded_rectangle(
    draw: ImageDraw.ImageDraw,
    xy: tuple[float, float, float, float],
    radius: float,
    fill: tuple[int, int, int, int],
) -> None:
    """Draw a filled rounded rectangle using Pillow primitives.

    Fallback for older Pillow versions that lack ImageDraw.rounded_rectangle.
    """
    if hasattr(draw, "rounded_rectangle"):
        draw.rounded_rectangle(xy, radius=radius, fill=fill)
        return

    x0, y0, x1, y1 = xy
    d = radius * 2
    # Body
    draw.rectangle([x0 + radius, y0, x1 - radius, y1], fill=fill)
    draw.rectangle([x0, y0 + radius, x1, y1 - radius], fill=fill)
    # Corners
    draw.ellipse([x0, y0, x0 + d, y0 + d], fill=fill)
    draw.ellipse([x1 - d, y0, x1, y0 + d], fill=fill)
    draw.ellipse([x0, y1 - d, x0 + d, y1], fill=fill)
    draw.ellipse([x1 - d, y1 - d, x1, y1], fill=fill)


def render_logo(size: int) -> Image.Image:
    """Render the logo at the requested size with antialiasing via supersampling."""
    scale = 4
    canvas_size = size * scale

    img = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Scale geometry from 256x256 design space to canvas_size.
    s = canvas_size / 256.0

    frame_x = FRAME_X * s
    frame_y = FRAME_Y * s
    frame_size = FRAME_SIZE * s
    frame_radius = FRAME_RADIUS * s
    stroke = STROKE_WIDTH * s
    dot_cx = DOT_CX * s
    dot_cy = DOT_CY * s
    dot_r = DOT_R * s

    # Outer accent frame.
    rounded_rectangle(
        draw,
        (frame_x, frame_y, frame_x + frame_size, frame_y + frame_size),
        frame_radius,
        fill=ACCENT,
    )

    # Inner white fill (creates the stroke effect).
    inner_inset = stroke
    inner_x = frame_x + inner_inset
    inner_y = frame_y + inner_inset
    inner_size = frame_size - 2 * inner_inset
    inner_radius = max(0.0, frame_radius - inner_inset)
    rounded_rectangle(
        draw,
        (inner_x, inner_y, inner_x + inner_size, inner_y + inner_size),
        inner_radius,
        fill=FILL,
    )

    # Cinnabar pigment drop straddling the lower-right frame edge.
    draw.ellipse(
        [
            dot_cx - dot_r,
            dot_cy - dot_r,
            dot_cx + dot_r,
            dot_cy + dot_r,
        ],
        fill=CINNABAR,
    )

    # Downsample with high-quality antialiasing.
    return img.resize((size, size), Image.Resampling.LANCZOS)


def save_png(image: Image.Image, size: int) -> Path:
    path = OUT_DIR / f"logo_{size}.png"
    image.save(path, "PNG")
    return path


def save_ico(images: dict[int, Image.Image]) -> Path:
    """Save a multi-resolution ICO file.

    Windows icons commonly include 16/32/48/256; we also keep 24 for Linux trays.
    Pillow stores 256x256 as PNG-compressed inside the ICO container.
    """
    path = OUT_DIR / "logo.ico"
    ordered = [images[size] for size in ICO_SIZES if size in images]
    ordered[0].save(path, format="ICO", append_images=ordered[1:])
    return path


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    rendered: dict[int, Image.Image] = {}
    for size in PNG_SIZES:
        img = render_logo(size)
        rendered[size] = img
        path = save_png(img, size)
        print(f"Exported {path}")

    ico_path = save_ico(rendered)
    print(f"Exported {ico_path}")


if __name__ == "__main__":
    main()
