#!/usr/bin/env python3
"""Export danqing Phase 2 scene assets (pomodoro POC).

Nine procedural scenes spanning dark/bright families:
    bonfire   篝火 (dark, warm fire glow)
    sea       海   (bright, cyan)
    rain      雨   (gray-blue)
    mountain  山   (neutral dusk, ridgelines)
    forest    森林 (misty conifer green, treelines + fog bands)
    starry    星夜 (deep indigo night, starfield + moon glow + dark hills)
    snowfield 雪原 (pale cold sky, snow plain drifts + faint far hills)
    desert    沙漠 (dusk sand, setting sun glow + rolling dunes)
    cloudsea  云海 (sunrise sky, rolling cloud sea above horizon)

Each scene PNG bakes: multi-stop vertical gradient + radial glow +
center readability veil + scene-specific details.
No external assets; fully deterministic.

Outputs:
    assets/scenes/{bonfire,sea,rain,mountain,forest}.png
    examples/pomodoro/scenes.rs   # SceneSpec consts incl. palettes (generated, do not edit)

The script also enforces contrast guards at generation time:
display text vs sampled center extremes >= 3:1,
control text vs glass-composited surface >= 4:1 (WCAG).

Dependency:
    pip install Pillow

Usage:
    python tools/export-scenes.py
"""

import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = REPO_ROOT / "assets" / "scenes"
SCENES_RS = REPO_ROOT / "examples" / "pomodoro" / "scenes.rs"

# 3:2 canvas matching the POC window aspect (960x640); Cover scales cleanly.
WIDTH, HEIGHT = 1536, 1024
SIZE = (WIDTH, HEIGHT)

# Hard-edged elements (ridges/embers) render at SS x and LANCZOS
# downsample back: cheap SSAA — Pillow ImageDraw has no anti-aliasing,
# and window Cover upscale (e.g. 1920x1080 maximized) makes baked-in
# staircase edges painfully visible. Smooth elements (gradient/glow/veil)
# stay at native resolution: nothing to alias there, and their big
# gaussian blurs would be orders of magnitude slower at SS x.
SS = 4

# Center sampling region for backdrop extremes (where the countdown sits).
CENTER_BOX = (0.30, 0.35, 0.70, 0.65)  # x0, y0, x1, y1 fractions

# WCAG guard thresholds.
DISPLAY_MIN = 3.0
CONTROL_MIN = 4.0


def lerp(a: float, b: float, t: float) -> float:
    return a + (b - a) * t


def lerp_rgb(a: tuple, b: tuple, t: float) -> tuple[int, int, int]:
    return (
        int(lerp(a[0], b[0], t)),
        int(lerp(a[1], b[1], t)),
        int(lerp(a[2], b[2], t)),
    )


def srgb_to_linear(c: float) -> float:
    c = c / 255.0
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def luminance(rgb: tuple) -> float:
    return (
        0.2126 * srgb_to_linear(rgb[0])
        + 0.7152 * srgb_to_linear(rgb[1])
        + 0.0722 * srgb_to_linear(rgb[2])
    )


def contrast(a: tuple, b: tuple) -> float:
    la, lb = luminance(a), luminance(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)


def composite(top_rgb: tuple, top_alpha: float, base_rgb: tuple) -> tuple[int, int, int]:
    return tuple(int(top_rgb[i] * top_alpha + base_rgb[i] * (1.0 - top_alpha)) for i in range(3))


def build_gradient(stops: list[tuple[float, tuple]]) -> Image.Image:
    """Vertical multi-stop gradient. stops: [(pos 0..1, (r,g,b)), ...] sorted by pos."""
    img = Image.new("RGB", SIZE)
    draw = ImageDraw.Draw(img)
    for y in range(HEIGHT):
        t = y / (HEIGHT - 1)
        # Find surrounding stops.
        lo = stops[0]
        hi = stops[-1]
        for i in range(len(stops) - 1):
            if stops[i][0] <= t <= stops[i + 1][0]:
                lo, hi = stops[i], stops[i + 1]
                break
        span = hi[0] - lo[0]
        local = 0.0 if span <= 0 else (t - lo[0]) / span
        draw.line([(0, y), (WIDTH, y)], fill=lerp_rgb(lo[1], hi[1], local))
    return img


def radial_overlay(
    color: tuple,
    center: tuple[float, float],
    radius_frac: float,
    peak_alpha: int,
    falloff: float = 1.5,
    blur_frac: float = 0.02,
) -> Image.Image:
    """Radial alpha overlay via concentric ellipses + blur (same technique as phase-1 glow)."""
    img = Image.new("RGBA", SIZE, (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    cx, cy = center[0] * WIDTH, center[1] * HEIGHT
    max_r = radius_frac * max(WIDTH, HEIGHT)
    steps = 240
    for i in range(steps, 0, -1):
        t = i / steps
        r = max_r * t
        alpha = int(peak_alpha * (1.0 - t**falloff))
        if alpha <= 1:
            continue
        draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(*color, alpha))
    return img.filter(ImageFilter.GaussianBlur(radius=WIDTH * blur_frac))


def build_ridges(layers: list[dict]) -> Image.Image:
    """Deterministic mountain ridgelines (midpoint-displacement silhouettes).

    Rendered at SS x and downsampled for anti-aliased edges.
    """
    w, h = WIDTH * SS, HEIGHT * SS
    overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    for layer in layers:
        state = layer["seed"]
        n = 9
        xs = [i * w // (n - 1) for i in range(n)]

        def rnd() -> float:
            nonlocal state
            state = (state * 1103515245 + 12345) & 0x7FFFFFFF
            return (state >> 16) / 32768.0

        base_y = layer["base_y"] * h
        amp = layer["amp"] * h
        pts = [(x, base_y - rnd() * amp) for x in xs]
        # Smooth-ish: interpolate midpoints between peaks.
        smooth = []
        for i in range(len(pts) - 1):
            (x0, y0), (x1, y1) = pts[i], pts[i + 1]
            smooth.append((x0, y0))
            smooth.append(((x0 + x1) / 2, (y0 + y1) / 2))
        smooth.append(pts[-1])
        polygon = smooth + [(w, h), (0, h)]
        ImageDraw.Draw(overlay).polygon(polygon, fill=(*layer["color"], layer["alpha"]))
    overlay = overlay.resize(SIZE, Image.LANCZOS)
    return overlay.filter(ImageFilter.GaussianBlur(radius=1.5))


def build_waves(layers: list[dict]) -> Image.Image:
    """Sea waves: sinusoidal silhouettes layered toward the bottom (SS x for AA).

    Each layer: base_y/amp (height fractions), freq (cycles across width),
    phase (radians), color, alpha. Filled below the curve like ridges,
    but the sinusoid reads as swell instead of a mountain line.
    """
    w, h = WIDTH * SS, HEIGHT * SS
    overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    for layer in layers:
        base_y = layer["base_y"] * h
        amp = layer["amp"] * h
        freq = layer["freq"]
        phase = layer.get("phase", 0.0)
        steps = 160
        pts = [
            (
                w * i / steps,
                base_y + amp * math.sin(2.0 * math.pi * freq * i / steps + phase),
            )
            for i in range(steps + 1)
        ]
        draw.polygon(pts + [(w, h), (0, h)], fill=(*layer["color"], layer["alpha"]))
    overlay = overlay.resize(SIZE, Image.LANCZOS)
    return overlay.filter(ImageFilter.GaussianBlur(radius=1.2))


def _hash_unit(v: int) -> float:
    """Deterministic 0..1 hash of an integer (per-ridge variation)."""
    x = (v * 1103515245 + 12345) & 0x7FFFFFFF
    return ((x >> 16) & 0x7FFF) / 32768.0


def build_dunes(layers: list[dict]) -> Image.Image:
    """Smooth asymmetric sand dune ridges (SS x for AA).

    Each layer: base_y/amp/freq/phase/skew/color/alpha/blur/shear/seed. The
    profile is a smooth skewed bump — a long gentle windward climb (fraction
    `skew` of the period) up to a rounded crest, then a shorter steeper slip
    face; both sides cosine-eased so there are no hard corners (顺滑).
    Each ridge gets a seeded height/phase jitter (so the field is not a
    uniform repeating pattern) and the whole layer shears diagonally
    (`shear`) — dunes, not ocean swell (build_waves' symmetric, parallel,
    horizontal sinusoid banks).
    """
    w, h = WIDTH * SS, HEIGHT * SS
    result = Image.new("RGBA", SIZE, (0, 0, 0, 0))
    for layer in layers:
        overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        draw = ImageDraw.Draw(overlay)
        base_y = layer["base_y"] * h
        amp = layer["amp"] * h
        freq = layer["freq"]
        phase = layer.get("phase", 0.0)
        skew = layer.get("skew", 0.72)
        shear = layer.get("shear", 0.0)
        seed = layer.get("seed", 0xD00)
        steps = 240
        pts = []
        # 扩展采样范围覆盖斜切偏移, 保证脊线剪切后仍铺满全宽 (不露画布边缘)。
        x0 = -0.20 * w
        x1 = 1.20 * w
        for i in range(steps + 1):
            x = x0 + (x1 - x0) * i / steps
            ridge = int(x / w * freq)
            j = _hash_unit(ridge + seed)
            amp_i = amp * (0.72 + 0.56 * j)  # 每道沙丘高度不同
            t = (x / w * freq + phase + 0.35 * j) % 1.0
            if t < skew:
                prof = 0.5 - 0.5 * math.cos(math.pi * t / skew)  # 平滑缓坡
            else:
                prof = 0.5 + 0.5 * math.cos(math.pi * (t - skew) / (1.0 - skew))  # 平滑陡面
            y = base_y - amp_i * prof
            xs = x + shear * (base_y - y)  # 斜向剪切: 脊线倾斜, 非水平平行
            pts.append((xs, y))
        draw.polygon(pts + [(w, h), (0, h)], fill=(*layer["color"], layer["alpha"]))
        overlay = overlay.resize(SIZE, Image.LANCZOS)
        overlay = overlay.filter(ImageFilter.GaussianBlur(radius=layer.get("blur", 1.0)))
        result = Image.alpha_composite(result, overlay)
    return result


def build_trees(layers: list[dict]) -> Image.Image:
    """Forest treelines: dense rows of conifer triangles (SS x for AA).

    Each layer: base_y (baseline fraction), h_min/h_max (tree height
    fractions), color, alpha, blur (native-res gaussian radius), seed;
    optional und (baseline undulation amplitude) / freq (undulation
    cycles). The baseline rolls like forested hills — a flat baseline
    reads as a shelf, not terrain. Trees overlap heavily and sit on a
    solid mass that follows the same curve, so each layer reads as
    continuous canopy with a jagged horizon. Layers composite far-to-near;
    far layers should be lighter, lower-alpha and blurrier (fog eats them).
    """
    w, h = WIDTH * SS, HEIGHT * SS
    result = Image.new("RGBA", SIZE, (0, 0, 0, 0))
    for layer in layers:
        state = layer["seed"]

        def rnd() -> float:
            nonlocal state
            state = (state * 1103515245 + 12345) & 0x7FFFFFFF
            return (state >> 16) / 32768.0

        overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        draw = ImageDraw.Draw(overlay)
        base_y = layer["base_y"] * h
        und = layer.get("und", 0.02) * h
        freq = layer.get("freq", 2.0)
        phase = (layer["seed"] % 628) / 100.0  # seed-derived, deterministic

        def baseline(x: float) -> float:
            return base_y + und * math.sin(2.0 * math.pi * freq * x / w + phase)

        # Solid forest mass below the undulating treeline.
        steps = 120
        mass = [(w * i / steps, baseline(w * i / steps)) for i in range(steps + 1)]
        draw.polygon(mass + [(w, h), (0, h)], fill=(*layer["color"], layer["alpha"]))
        # Dense overlapping conifers; gaps show the same-color mass beneath.
        x = -rnd() * 40 * SS
        while x < w:
            th = (layer["h_min"] + rnd() * (layer["h_max"] - layer["h_min"])) * h
            half = th * (0.20 + rnd() * 0.10)
            by = baseline(x)
            draw.polygon(
                [(x - half, by), (x, by - th), (x + half, by)],
                fill=(*layer["color"], layer["alpha"]),
            )
            x += half * 2 * (0.35 + rnd() * 0.4)
        overlay = overlay.resize(SIZE, Image.LANCZOS)
        overlay = overlay.filter(ImageFilter.GaussianBlur(radius=layer.get("blur", 1.2)))
        result = Image.alpha_composite(result, overlay)
    return result


def build_mist(bands: list[dict]) -> Image.Image:
    """Horizontal fog bands (native res): sin-ramped alpha rows + big blur.

    Each band: y/height (fractions), color, alpha (peak at band center).
    Smooth by construction, so no SS needed; the blur melts it into the scene.
    """
    overlay = Image.new("RGBA", SIZE, (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    for band in bands:
        y0 = int((band["y"] - band["height"] / 2) * HEIGHT)
        y1 = int((band["y"] + band["height"] / 2) * HEIGHT)
        for y in range(y0, y1):
            t = (y - y0) / max(1, y1 - y0)
            alpha = int(band["alpha"] * math.sin(math.pi * t))
            if alpha > 0:
                draw.line([(0, y), (WIDTH, y)], fill=(*band["color"], alpha))
    return overlay.filter(ImageFilter.GaussianBlur(radius=18))


def build_embers(count: int, color: tuple, seed: int) -> Image.Image:
    """Bonfire embers: tiny bright dots rising above the glow. SS x for AA."""
    w, h = WIDTH * SS, HEIGHT * SS
    overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    state = seed

    def rnd() -> float:
        nonlocal state
        state = (state * 1103515245 + 12345) & 0x7FFFFFFF
        return (state >> 16) / 32768.0

    for _ in range(count):
        x = w * (0.30 + rnd() * 0.40)
        y = h * (0.55 + rnd() * 0.33)
        r = (1 + rnd() * 2.2) * SS
        a = int(60 + rnd() * 140)
        draw.ellipse([x - r, y - r, x + r, y + r], fill=(*color, a))
    overlay = overlay.resize(SIZE, Image.LANCZOS)
    return overlay.filter(ImageFilter.GaussianBlur(radius=0.6))


def build_stars(cfg: dict) -> Image.Image:
    """Starry night: deterministic scattered star dots (SS x for AA).

    Small sparse stars (dim) + a few brighter ones. All dots are small and
    sparse enough that the center-region 64x64 downsample (used by the
    contrast guard) averages them away — the backdrop extremes stay on the
    dark sky, so near-white countdown text keeps its contrast.
    cfg: count, seed, band (y range), colors (dim/bright), big_frac.
    """
    w, h = WIDTH * SS, HEIGHT * SS
    overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    state = cfg["seed"]
    band = cfg["band"]
    colors = cfg["colors"]
    big_frac = cfg.get("big_frac", 0.10)

    def rnd() -> float:
        nonlocal state
        state = (state * 1103515245 + 12345) & 0x7FFFFFFF
        return (state >> 16) / 32768.0

    for _ in range(cfg["count"]):
        x = w * rnd()
        y = h * (band[0] + rnd() * (band[1] - band[0]))
        big = rnd() < big_frac
        r = (1.0 + rnd() * (1.6 if big else 0.9)) * SS
        a = int((140 + rnd() * 115) if big else (50 + rnd() * 100))
        col = colors[1] if rnd() < 0.22 else colors[0]
        draw.ellipse([x - r, y - r, x + r, y + r], fill=(*col, a))
    overlay = overlay.resize(SIZE, Image.LANCZOS)
    return overlay.filter(ImageFilter.GaussianBlur(radius=0.4))


def build_snow_forest(layers: list[dict]) -> Image.Image:
    """Dusk snowy conifer forest: dense snow-dusted spruces (SS x for AA).

    Each layer: base_y (baseline), h_min/h_max (tree height), body (spruce
    color), snow (snow-cap color), ground (forest-floor color), alpha, blur,
    seed; und/freq for baseline undulation. A solid mass fills below the
    baseline; dense overlapping tiered spruces rise above it, each with a
    white snow cap on the top tier — the canopy reads as a dark winter
    forest (light-text friendly; the bright caps stay above the center text
    region, so the sampled center stays dark).
    """
    w, h = WIDTH * SS, HEIGHT * SS
    result = Image.new("RGBA", SIZE, (0, 0, 0, 0))
    for layer in layers:
        state = layer["seed"]

        def rnd() -> float:
            nonlocal state
            state = (state * 1103515245 + 12345) & 0x7FFFFFFF
            return (state >> 16) / 32768.0

        overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        draw = ImageDraw.Draw(overlay)
        base_y = layer["base_y"] * h
        und = layer.get("und", 0.03) * h
        freq = layer.get("freq", 2.0)
        phase = (layer["seed"] % 628) / 100.0

        def baseline(x: float) -> float:
            return base_y + und * math.sin(2.0 * math.pi * freq * x / w + phase)

        # 树根雪地: 基线以下实心 (阴影蓝, 黄昏基调); 可选 (远层只画树)。
        if "ground" in layer:
            steps = 120
            mass = [(w * i / steps, baseline(w * i / steps)) for i in range(steps + 1)]
            draw.polygon(mass + [(w, h), (0, h)], fill=(*layer["ground"], layer["alpha"]))
        # 密集重叠云杉 + 顶层雪帽。
        x = -rnd() * 50 * SS
        while x < w:
            th = (layer["h_min"] + rnd() * (layer["h_max"] - layer["h_min"])) * h
            by = baseline(x)
            half = th * (0.17 + rnd() * 0.07)
            tiers = 3
            for t in range(tiers):
                t_bot = by - th * (t / tiers)
                t_top = by - th * ((t + 1) / tiers)
                ww = half * (1.0 - 0.20 * t)
                draw.polygon(
                    [(x - ww, t_bot), (x + ww, t_bot), (x, t_top)],
                    fill=(*layer["body"], layer["alpha"]),
                )
            # 顶层雪帽: 树尖叠白三角 (比树身明显, 雪压枝感)。
            t_top = by - th
            cap_h = th * 0.20
            cap_w = half * 0.6
            draw.polygon(
                [(x - cap_w, t_top + cap_h), (x + cap_w, t_top + cap_h), (x, t_top)],
                fill=(*layer["snow"], layer["alpha"]),
            )
            x += half * 2.0 * (0.35 + rnd() * 0.4)
        overlay = overlay.resize(SIZE, Image.LANCZOS)
        overlay = overlay.filter(ImageFilter.GaussianBlur(radius=layer.get("blur", 1.2)))
        result = Image.alpha_composite(result, overlay)
    return result


def build_waterfall(cfg: dict) -> Image.Image:
    """A green gorge with a central white cascade (SS x for AA).

    Jagged dark cliff walls frame the sides; a light trapezoid cascade runs
    down the center with vertical water streaks; a soft pool gathers at the
    base. The cascade spans the center text region (bright), so dark text
    keeps contrast; the dark cliffs stay outside it.
    """
    w, h = WIDTH * SS, HEIGHT * SS
    overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    wall = cfg["wall"]

    def cliff(edge_x: float, sign: int, seed: int) -> None:
        state = seed

        def rnd() -> float:
            nonlocal state
            state = (state * 1103515245 + 12345) & 0x7FFFFFFF
            return (state >> 16) / 32768.0

        pts = [(0 if sign < 0 else w, 0), (edge_x * w + sign * 0.01 * rnd() * w, 0)]
        n = 10
        for i in range(1, n + 1):
            y = h * i / n
            xx = edge_x * w + sign * (0.02 + rnd() * 0.05) * w
            pts.append((xx, y))
        pts.append((edge_x * w + sign * 0.02 * w, h))
        pts.append((0 if sign < 0 else w, h))
        draw.polygon(pts, fill=(*wall, cfg["wall_alpha"]))

    cliff(cfg["wall_edge_l"], -1, cfg["seed_left"])
    cliff(cfg["wall_edge_r"], 1, cfg["seed_right"])
    # 中央瀑布: 上窄下宽的梯形水幕。
    cascade = [
        (cfg["top_w0"] * w, cfg["top_y"] * h),
        (cfg["top_w1"] * w, cfg["top_y"] * h),
        (cfg["bot_w1"] * w, cfg["bot_y"] * h),
        (cfg["bot_w0"] * w, cfg["bot_y"] * h),
    ]
    draw.polygon(cascade, fill=(*cfg["water"], cfg["water_alpha"]))
    # 竖向水纹: 落在中央稳定带内。
    state = cfg["seed_streak"]

    def rnd2() -> float:
        nonlocal state
        state = (state * 1103515245 + 12345) & 0x7FFFFFFF
        return (state >> 16) / 32768.0

    for _ in range(cfg["streaks"]):
        sx = (cfg["top_w0"] + 0.02 + rnd2() * (cfg["top_w1"] - cfg["top_w0"] - 0.04)) * w
        a = int(60 + rnd2() * 130)
        draw.line(
            [(sx, cfg["top_y"] * h), (sx, cfg["bot_y"] * 0.98 * h)],
            fill=(*cfg["streak"], a),
            width=int(1.2 * SS),
        )
    # 底部水潭: 软椭圆。
    px, py = cfg["pool_x"] * w, cfg["pool_y"] * h
    prx, pry = cfg["pool_rx"] * w, cfg["pool_ry"] * h
    ImageDraw.Draw(overlay).ellipse(
        [px - prx, py - pry, px + prx, py + pry], fill=(*cfg["pool"], 230)
    )
    overlay = overlay.resize(SIZE, Image.LANCZOS).filter(ImageFilter.GaussianBlur(radius=1.2))
    pool = overlay  # pool blur 已含在整体模糊
    return pool


def sample_center_extremes(img: Image.Image) -> tuple[tuple, tuple]:
    """Brightest/darkest colors (by luminance) in the center region, after all baking."""
    x0, y0 = int(CENTER_BOX[0] * WIDTH), int(CENTER_BOX[1] * HEIGHT)
    x1, y1 = int(CENTER_BOX[2] * WIDTH), int(CENTER_BOX[3] * HEIGHT)
    region = img.crop((x0, y0, x1, y1)).resize((64, 64))  # downsample: extremes of areas, not pixels
    best_light, best_dark = None, None
    l_light, l_dark = -1.0, 2.0
    for px in region.getdata():
        l = luminance(px)
        if l > l_light:
            l_light, best_light = l, px
        if l < l_dark:
            l_dark, best_dark = l, px
    return best_light, best_dark


# ---------------------------------------------------------------------------
# Scene configs. Palettes are hand-authored; backdrop extremes are SAMPLED
# from the generated center region and injected into the palette at emit time.
# ---------------------------------------------------------------------------

SCENES = [
    {
        "key": "bonfire",
        "name": "篝火",
        "stops": [
            (0.00, (13, 8, 6)),
            (0.45, (26, 15, 10)),
            (0.75, (48, 24, 12)),
            (1.00, (24, 12, 8)),
        ],
        "glow": {"color": (255, 159, 67), "center": (0.5, 0.86), "radius": 0.48, "peak": 120},
        "veil": {"color": (0, 0, 0), "center": (0.5, 0.48), "radius": 0.55, "peak": 60},
        "embers": {"count": 42, "color": (255, 190, 110), "seed": 0xB0E1},
        "palette": {
            "base": (26, 15, 10),
            "accent": (255, 159, 67),
            "text_primary": (250, 244, 235),
            "text_secondary": (199, 184, 166),
            "surface": ((0, 0, 0), 0.25),
            "surface_input": ((0, 0, 0), 0.38),
        },
    },
    {
        "key": "sea",
        "name": "海",
        "stops": [
            (0.00, (234, 250, 251)),
            (0.45, (191, 232, 238)),
            (0.72, (137, 207, 220)),
            (1.00, (88, 166, 188)),
        ],
        "glow": {"color": (255, 255, 255), "center": (0.5, 0.30), "radius": 0.48, "peak": 90},
        "veil": {"color": (255, 255, 255), "center": (0.5, 0.48), "radius": 0.55, "peak": 55},
        "waves": [
            # 远涌: 低饱和亮带, 长波缓幅。
            {"base_y": 0.72, "amp": 0.012, "freq": 3.0, "phase": 0.0,
             "color": (214, 240, 245), "alpha": 60},
            # 中浪: 相位错开, 更亮。
            {"base_y": 0.83, "amp": 0.018, "freq": 2.5, "phase": 1.7,
             "color": (235, 248, 250), "alpha": 90},
            # 近岸碎浪: 幅最大, 最亮, 略有泡沫感。
            {"base_y": 0.93, "amp": 0.024, "freq": 2.0, "phase": 3.4,
             "color": (248, 253, 254), "alpha": 130},
        ],
        "palette": {
            "base": (168, 221, 232),
            "accent": (12, 74, 110),
            "text_primary": (8, 32, 48),
            "text_secondary": (60, 90, 105),
            "surface": ((255, 255, 255), 0.55),
            "surface_input": ((255, 255, 255), 0.85),
        },
    },
    {
        "key": "rain",
        "name": "雨",
        "stops": [
            (0.00, (122, 135, 147)),
            (0.45, (88, 101, 113)),
            (0.80, (63, 74, 86)),
            (1.00, (46, 55, 66)),
        ],
        "glow": {"color": (210, 224, 235), "center": (0.32, 0.22), "radius": 0.5, "peak": 50},
        "veil": {"color": (10, 14, 18), "center": (0.5, 0.48), "radius": 0.55, "peak": 45},
        # 雨丝不烘焙: 2026-07-29 用户裁定静态图去丝, 雨全部由运行时程序化
        # 雨幕渲染 (background.wgsl rain_overlay; 暂停雨钟冻结、雨丝定格可见)。
        "palette": {
            "base": (82, 95, 107),
            "accent": (127, 179, 217),
            "text_primary": (242, 246, 249),
            "text_secondary": (195, 205, 213),
            "surface": ((0, 0, 0), 0.25),
            "surface_input": ((0, 0, 0), 0.38),
        },
    },
    {
        "key": "mountain",
        "name": "山",
        "stops": [
            (0.00, (43, 36, 64)),
            (0.40, (86, 80, 115)),
            (0.62, (139, 125, 158)),
            (0.78, (199, 172, 178)),
            (1.00, (62, 52, 82)),
        ],
        "glow": {"color": (240, 200, 170), "center": (0.5, 0.66), "radius": 0.4, "peak": 60},
        "veil": {"color": (20, 16, 32), "center": (0.5, 0.46), "radius": 0.55, "peak": 40},
        "ridges": [
            {"base_y": 0.86, "amp": 0.10, "color": (52, 44, 74), "alpha": 235, "seed": 0xA01},
            {"base_y": 0.97, "amp": 0.08, "color": (34, 28, 50), "alpha": 255, "seed": 0xA02},
        ],
        "palette": {
            "base": (86, 80, 115),
            "accent": (232, 192, 122),
            "text_primary": (245, 241, 250),
            "text_secondary": (205, 198, 218),
            "surface": ((0, 0, 0), 0.25),
            "surface_input": ((0, 0, 0), 0.38),
        },
    },
    {
        "key": "forest",
        "name": "森林",
        # 雾顶亮、中部压暗保倒计时对比度、底部深绿的纵向结构。
        "stops": [
            (0.00, (168, 185, 171)),
            (0.30, (126, 146, 130)),
            (0.55, (82, 104, 88)),
            (0.80, (50, 72, 59)),
            (1.00, (34, 52, 43)),
        ],
        # 顶部天光 (穿雾), 克制峰值避免中央采样区过亮。
        "glow": {"color": (214, 228, 214), "center": (0.5, 0.10), "radius": 0.42, "peak": 45},
        "veil": {"color": (13, 21, 16), "center": (0.5, 0.48), "radius": 0.55, "peak": 60},
        "trees": [
            # 远林: 雾中淡影, 最虚。
            {"base_y": 0.52, "h_min": 0.05, "h_max": 0.10, "color": (118, 138, 122),
             "alpha": 110, "blur": 2.5, "seed": 0xF01},
            # 中林。
            {"base_y": 0.68, "h_min": 0.08, "h_max": 0.15, "color": (72, 94, 78),
             "alpha": 190, "blur": 1.5, "seed": 0xF02},
            # 近林: 最深最实, 收住底边。
            {"base_y": 0.88, "h_min": 0.12, "h_max": 0.22, "color": (36, 56, 45),
             "alpha": 255, "blur": 1.0, "seed": 0xF03},
        ],
        # 雾不烘焙: 2026-07-30 用户裁定森林静态图去底雾, 雾全部由运行时程序化
        # 渲染 (background.wgsl forest_mist, 3 层 2D 各向同性 + 风驱, 暂停 500ms
        # 沉降回裸静态图 — 雨场景改造范式)。 对比 export-scenes.py 雨配置: 雨
        # 丝不烘焙, 同样全运行时。
        "palette": {
            "base": (50, 72, 59),
            "accent": (172, 198, 158),
            "text_primary": (240, 246, 240),
            "text_secondary": (186, 201, 187),
            "surface": ((0, 0, 0), 0.25),
            "surface_input": ((0, 0, 0), 0.38),
        },
    },
    {
        "key": "starry",
        "name": "星夜",
        # 深靛蓝夜空 → 暗地; 中央保持暗, 保白字对比度。星点稀疏,
        # 中心极值落在夜空本身 (无月亮)。
        "stops": [
            (0.00, (10, 12, 30)),
            (0.45, (22, 26, 52)),
            (0.72, (38, 42, 74)),
            (1.00, (16, 18, 40)),
        ],
        "stars": {
            "count": 170, "seed": 0x51A1, "band": (0.02, 0.80),
            "colors": ((200, 212, 244), (255, 236, 190)), "big_frac": 0.10,
        },
        "ridges": [
            {"base_y": 0.88, "amp": 0.06, "color": (12, 14, 32), "alpha": 235, "seed": 0x501},
            {"base_y": 0.97, "amp": 0.05, "color": (8, 9, 22), "alpha": 255, "seed": 0x502},
        ],
        "veil": {"color": (0, 0, 0), "center": (0.5, 0.48), "radius": 0.55, "peak": 50},
        "palette": {
            "base": (22, 26, 52),
            "accent": (255, 224, 160),
            "text_primary": (246, 247, 255),
            "text_secondary": (185, 192, 220),
            "surface": ((0, 0, 0), 0.25),
            "surface_input": ((0, 0, 0), 0.38),
        },
    },
    {
        "key": "snowfield",
        "name": "针叶林雪原",
        # 黄昏蓝灰天空 → 深色云杉雪林 (远层淡、近层深); 中央为暗林冠 (白字对比度),
        # 雪帽在中央区上方, 地面为护栏极限内的阴影蓝雪。
        "stops": [
            (0.00, (46, 56, 78)),
            (0.28, (82, 94, 116)),
            (0.52, (112, 122, 136)),
            (0.80, (126, 136, 148)),
            (1.00, (112, 124, 138)),
        ],
        "glow": {"color": (205, 220, 240), "center": (0.5, 0.10), "radius": 0.40, "peak": 40},
        "snow_forest": [
            {"base_y": 0.36, "und": 0.02, "freq": 2.4, "h_min": 0.06, "h_max": 0.10,
             "body": (70, 88, 102), "snow": (192, 208, 226),
             "alpha": 200, "blur": 2.0, "seed": 0x811},
            {"base_y": 0.42, "und": 0.03, "freq": 2.0, "h_min": 0.14, "h_max": 0.20,
             "body": (32, 48, 58), "snow": (236, 246, 254), "ground": (104, 118, 130),
             "alpha": 255, "blur": 1.0, "seed": 0x812},
        ],
        "veil": {"color": (0, 0, 0), "center": (0.5, 0.48), "radius": 0.55, "peak": 30},
        "palette": {
            "base": (74, 84, 104),
            "accent": (180, 205, 225),
            "text_primary": (240, 246, 252),
            "text_secondary": (186, 198, 214),
            "surface": ((0, 0, 0), 0.25),
            "surface_input": ((0, 0, 0), 0.38),
        },
    },
    {
        "key": "desert",
        "name": "沙漠",
        # 暮色天空 → 暖沙地; 中央保持亮 (低垂落日暖光), 深字对比度随落日抬升。
        # 沙丘基线在中央采样区 (y 0.65) 之下, 避免最深沙影拉低 backdrop_dark。
        "stops": [
            (0.00, (98, 72, 98)),
            (0.35, (150, 104, 92)),
            (0.55, (196, 138, 92)),
            (0.75, (172, 110, 72)),
            (1.00, (150, 90, 60)),
        ],
        "glow": {"color": (255, 216, 168), "center": (0.5, 0.50), "radius": 0.34, "peak": 100},
        "dunes": [
            {"base_y": 0.70, "amp": 0.055, "freq": 2.2, "phase": 0.0, "skew": 0.70,
             "shear": 0.32, "color": (170, 108, 70), "alpha": 200, "blur": 1.0, "seed": 0xD11},
            {"base_y": 0.82, "amp": 0.065, "freq": 1.8, "phase": 0.5, "skew": 0.74,
             "shear": 0.26, "color": (142, 88, 58), "alpha": 230, "blur": 0.8, "seed": 0xD12},
            {"base_y": 0.94, "amp": 0.07, "freq": 1.4, "phase": 1.1, "skew": 0.78,
             "shear": 0.20, "color": (120, 74, 48), "alpha": 255, "blur": 0.6, "seed": 0xD13},
        ],
        "veil": {"color": (0, 0, 0), "center": (0.5, 0.48), "radius": 0.55, "peak": 0},
        "palette": {
            "base": (172, 110, 72),
            "accent": (255, 200, 130),
            "text_primary": (58, 34, 26),
            "text_secondary": (120, 84, 60),
            "surface": ((255, 255, 255), 0.50),
            "surface_input": ((255, 255, 255), 0.80),
        },
    },
    {
        "key": "waterfall",
        "name": "瀑布",
        # 青绿峡谷 → 中央白色瀑布; 中央采样区为亮水幕 (深字对比度), 暗峭壁在两侧。
        "stops": [
            (0.00, (150, 190, 186)),
            (0.30, (128, 170, 168)),
            (0.55, (110, 152, 152)),
            (0.80, (150, 180, 178)),
            (1.00, (170, 196, 192)),
        ],
        "glow": {"color": (214, 242, 238), "center": (0.5, 0.30), "radius": 0.30, "peak": 55},
        "waterfall": {
            "wall": (34, 56, 54), "wall_alpha": 255, "wall_edge_l": 0.32, "wall_edge_r": 0.68,
            "seed_left": 0xA11, "seed_right": 0xA12,
            "top_w0": 0.36, "top_w1": 0.64, "top_y": 0.02,
            "bot_w0": 0.24, "bot_w1": 0.76, "bot_y": 0.90,
            "water": (226, 240, 246), "water_alpha": 240,
            "streak": (196, 218, 228), "streaks": 12, "seed_streak": 0xA21,
            "pool_x": 0.5, "pool_y": 0.90, "pool_rx": 0.34, "pool_ry": 0.08,
            "pool": (198, 224, 232),
        },
        "mist": [
            {"y": 0.80, "height": 0.10, "color": (228, 242, 244), "alpha": 46},
            {"y": 0.90, "height": 0.12, "color": (234, 246, 248), "alpha": 50},
        ],
        "veil": {"color": (0, 0, 0), "center": (0.5, 0.48), "radius": 0.55, "peak": 0},
        "palette": {
            "base": (120, 152, 148),
            "accent": (48, 130, 118),
            "text_primary": (26, 42, 42),
            "text_secondary": (95, 118, 114),
            "surface": ((255, 255, 255), 0.50),
            "surface_input": ((255, 255, 255), 0.80),
        },
    },
]


def build_scene(cfg: dict) -> Image.Image:
    img = build_gradient(cfg["stops"]).convert("RGBA")
    if "glow" in cfg:
        g = cfg["glow"]
        img = Image.alpha_composite(
            img, radial_overlay(g["color"], g["center"], g["radius"], g["peak"])
        )
    if "ridges" in cfg:
        img = Image.alpha_composite(img, build_ridges(cfg["ridges"]))
    if "waves" in cfg:
        img = Image.alpha_composite(img, build_waves(cfg["waves"]))
    if "trees" in cfg:
        img = Image.alpha_composite(img, build_trees(cfg["trees"]))
    if "embers" in cfg:
        e = cfg["embers"]
        img = Image.alpha_composite(img, build_embers(e["count"], e["color"], e["seed"]))
    if "stars" in cfg:
        img = Image.alpha_composite(img, build_stars(cfg["stars"]))
    if "dunes" in cfg:
        img = Image.alpha_composite(img, build_dunes(cfg["dunes"]))
    if "snow_forest" in cfg:
        img = Image.alpha_composite(img, build_snow_forest(cfg["snow_forest"]))
    if "waterfall" in cfg:
        img = Image.alpha_composite(img, build_waterfall(cfg["waterfall"]))
    if "mist" in cfg:
        img = Image.alpha_composite(img, build_mist(cfg["mist"]))
    v = cfg["veil"]
    img = Image.alpha_composite(
        img, radial_overlay(v["color"], v["center"], v["radius"], v["peak"])
    )
    # 不烘焙颗粒: 运行时噪声叠加层 (assets/background/noise.png) 负责防抖带,
    # 与阶段 1 背景一致; 烘焙颗粒会让 PNG 体积膨胀约 4 倍。
    return img.convert("RGB")


def rust_color(rgb: tuple) -> str:
    return f"Color::from_srgb8({rgb[0]}, {rgb[1]}, {rgb[2]})"


def rust_rgba(rgba: tuple[tuple, float]) -> str:
    (r, g, b), a = rgba
    return f"Color::rgba({r / 255:.6}, {g / 255:.6}, {b / 255:.6}, {a:.2})"


def emit_scenes_rs(entries: list[dict]) -> None:
    lines = [
        "//! @author 十四叔",
        "//! @date 2026/07/23",
        "",
        "//! 场景资产与调色板常量 —— 由 tools/export-scenes.py 生成, 勿手改。",
        "",
        "use danqing::{Color, ScenePalette, SceneSpec};",
        "",
        "/// POC 场景清单 (数组顺序即 ◀/▶ 切换顺序)。",
        f"pub const SCENES: [SceneSpec; {len(entries)}] = [",
    ]
    for e in entries:
        p = e["palette"]
        lines += [
            "    SceneSpec {",
            f'        name: "{e["name"]}",',
            f'        image: "assets/scenes/{e["key"]}.png",',
            "        palette: ScenePalette {",
            f"            base: {rust_color(p['base'])},",
            f"            accent: {rust_color(p['accent'])},",
            f"            text_primary: {rust_color(p['text_primary'])},",
            f"            text_secondary: {rust_color(p['text_secondary'])},",
            f"            surface: {rust_rgba(p['surface'])},",
            f"            surface_input: {rust_rgba(p['surface_input'])},",
            f"            backdrop_light: {rust_color(p['backdrop_light'])},",
            f"            backdrop_dark: {rust_color(p['backdrop_dark'])},",
            "        },",
            "    },",
        ]
    lines.append("];")
    # Rust 侧护栏测试随模板一起生成: 与 check_guards 同规则,
    # 防止 scenes.rs 被手改后护栏静默失效 (spec: 大字 >=3:1, 控件 >=4:1)。
    lines += GUARD_TESTS_RS.splitlines()
    lines.append("")
    SCENES_RS.write_text("\n".join(lines), encoding="utf-8")


GUARD_TESTS_RS = '''
#[cfg(test)]
mod tests {
    //! 对比度护栏: 与 tools/export-scenes.py 生成期护栏同规则,
    //! 防止 scenes.rs 被手改后护栏静默失效 (spec: 大字 ≥3:1, 控件 ≥4:1)。
    use super::*;
    use danqing::{composite_over, contrast_ratio};

    /// 大字 (倒计时) 对场景背景极值的最低对比度。
    const DISPLAY_MIN: f32 = 3.0;
    /// 控件文字对玻璃合成底的最低对比度。
    const CONTROL_MIN: f32 = 4.0;

    #[test]
    fn all_scenes_pass_contrast_guards() {
        assert_eq!(SCENES.len(), 9, "沉浸世界应有 9 个场景");
        for spec in &SCENES {
            let p = &spec.palette;
            for (label, backdrop) in [
                ("backdrop_light", p.backdrop_light),
                ("backdrop_dark", p.backdrop_dark),
            ] {
                let display = contrast_ratio(p.text_primary, backdrop);
                assert!(
                    display >= DISPLAY_MIN,
                    "{}: 大字 vs {label} = {display:.2} < {DISPLAY_MIN}",
                    spec.name
                );
                let glass = composite_over(p.surface, backdrop);
                let control = contrast_ratio(p.text_primary, glass);
                assert!(
                    control >= CONTROL_MIN,
                    "{}: 控件文字 vs 玻璃({label}) = {control:.2} < {CONTROL_MIN}",
                    spec.name
                );
            }
        }
    }

    #[test]
    fn scene_images_are_unique_and_named() {
        for (i, a) in SCENES.iter().enumerate() {
            for b in &SCENES[i + 1..] {
                assert_ne!(a.image, b.image, "场景图路径不应重复");
                assert_ne!(a.name, b.name, "场景名不应重复");
            }
        }
    }
}
'''



def check_guards(name: str, palette: dict) -> list[str]:
    """Contrast guards; returns list of violation messages."""
    problems = []
    text = palette["text_primary"]
    surf_rgb, surf_a = palette["surface"]
    for label, backdrop in [
        ("backdrop_light", palette["backdrop_light"]),
        ("backdrop_dark", palette["backdrop_dark"]),
    ]:
        ratio = contrast(text, backdrop)
        if ratio < DISPLAY_MIN:
            problems.append(f"{name}: 文字 vs {label} = {ratio:.2f} < {DISPLAY_MIN}")
        glass = composite(surf_rgb, surf_a, backdrop)
        ratio = contrast(text, glass)
        if ratio < CONTROL_MIN:
            problems.append(f"{name}: 文字 vs 玻璃({label}) = {ratio:.2f} < {CONTROL_MIN}")
    return problems


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    entries = []
    all_problems = []
    for cfg in SCENES:
        img = build_scene(cfg)
        path = OUT_DIR / f"{cfg['key']}.png"
        img.save(path, "PNG", optimize=True)

        light, dark = sample_center_extremes(img)
        palette = dict(cfg["palette"])
        palette["backdrop_light"] = light
        palette["backdrop_dark"] = dark
        entries.append({"key": cfg["key"], "name": cfg["name"], "palette": palette})

        problems = check_guards(cfg["name"], palette)
        all_problems += problems
        status = "OK " if not problems else "FAIL"
        print(
            f"[{status}] {cfg['name']} ({cfg['key']}): {path.name} {img.width}x{img.height}, "
            f"extremes light={light} dark={dark}"
        )
        for p in problems:
            print(f"       {p}")

    emit_scenes_rs(entries)
    print(f"Emitted {SCENES_RS.relative_to(REPO_ROOT)} ({len(entries)} scenes)")

    if all_problems:
        print("\n对比度护栏未过, 请调整调色板后重新生成。", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
