#!/usr/bin/env python3
"""Export danqing Phase 2 scene assets (pomodoro POC).

Six procedural scenes spanning dark/bright families:
    bonfire   篝火 (dark, warm fire glow)
    sea       海   (bright, cyan)
    rain      雨   (gray-blue)
    mountain  山   (neutral dusk, ridgelines)
    forest    森林 (misty conifer green, treelines + fog bands)
    starry    星夜 (deep indigo night, dark hills; starfield is runtime-rendered)

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

from PIL import Image, ImageChops, ImageDraw, ImageFilter

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
    """Sinusoidal silhouette banks layered toward the bottom (SS x for AA).

    Each layer: base_y/amp (height fractions), freq (cycles across width),
    phase (radians), color, alpha, optional diag (right-edge downward slant,
    in y fractions — makes ridges run diagonally instead of horizontal, e.g.
    desert dunes vs horizontal sea swell). Filled below the curve like
    ridges, but the sinusoid reads as swell instead of a mountain line.
    """
    w, h = WIDTH * SS, HEIGHT * SS
    overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    for layer in layers:
        base_y = layer["base_y"] * h
        amp = layer["amp"] * h
        freq = layer["freq"]
        phase = layer.get("phase", 0.0)
        diag = layer.get("diag", 0.0) * h
        steps = 160
        pts = [
            (
                w * i / steps,
                base_y
                + amp * math.sin(2.0 * math.pi * freq * i / steps + phase)
                + diag * (i / steps),
            )
            for i in range(steps + 1)
        ]
        draw.polygon(pts + [(w, h), (0, h)], fill=(*layer["color"], layer["alpha"]))
    overlay = overlay.resize(SIZE, Image.LANCZOS)
    return overlay.filter(ImageFilter.GaussianBlur(radius=1.2))


def build_trees(layers: list[dict]) -> Image.Image:
    """Forest treelines: proper conifer trees with trunks and canopies (SS x for AA).

    Each layer: base_y (baseline fraction), h_min/h_max (tree height
    fractions), color, alpha, blur (native-res gaussian radius), seed;
    optional und (baseline undulation amplitude) / freq (undulation
    cycles). Trees have vertical trunks + conical canopy; light shafts
    between trees add depth. Layers composite far-to-near.
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

        # Generate trees with vertical trunks + conical canopy.
        x = -rnd() * 40 * SS
        while x < w:
            th = (layer["h_min"] + rnd() * (layer["h_max"] - layer["h_min"])) * h
            trunk_h = th * (0.35 + rnd() * 0.15)  # trunk is 35-50% of total height
            canopy_h = th - trunk_h
            trunk_w = th * (0.03 + rnd() * 0.02)  # thin trunk
            canopy_half = th * (0.18 + rnd() * 0.08)  # canopy width at base
            by = baseline(x)

            # Draw trunk (vertical rectangle).
            trunk_top = by - trunk_h
            draw.rectangle(
                [x - trunk_w, trunk_top, x + trunk_w, by],
                fill=(*layer["color"], layer["alpha"]),
            )

            # Draw canopy (triangle on top of trunk).
            canopy_base = trunk_top
            canopy_top = trunk_top - canopy_h
            draw.polygon(
                [(x - canopy_half, canopy_base), (x, canopy_top), (x + canopy_half, canopy_base)],
                fill=(*layer["color"], layer["alpha"]),
            )

            x += canopy_half * 2 * (0.6 + rnd() * 0.5)  # spacing between trees

        # Add light shafts between trees (subtle vertical streaks).
        if layer.get("light_shafts", False):
            shaft_overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
            shaft_draw = ImageDraw.Draw(shaft_overlay)
            shaft_color = tuple(min(255, c + 30) for c in layer["color"])  # slightly lighter
            x = -rnd() * 60 * SS
            while x < w:
                if rnd() > 0.3:  # 70% chance to place a shaft
                    shaft_h = th * (0.4 + rnd() * 0.3)
                    shaft_w = th * (0.01 + rnd() * 0.01)
                    by = baseline(x)
                    shaft_top = by - shaft_h
                    # Gradient alpha: brighter at top, fading down.
                    for sy in range(int(shaft_top), int(by)):
                        t = (sy - shaft_top) / max(1, shaft_h)
                        alpha = int(layer["alpha"] * 0.3 * (1.0 - t))
                        shaft_draw.line([(x, sy), (x + shaft_w, sy)], fill=(*shaft_color, alpha))
                x += th * (0.8 + rnd() * 0.6)

            # Composite shafts onto overlay at SS resolution before resize.
            overlay = Image.alpha_composite(overlay, shaft_overlay)

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


# ---------------------------------------------------------------------------
# 银河 (星夜场景): 光带 + 尘埃暗隙 + 星点雾。
# 光带中心线严格落在 py=0 — 与 export-stars.py 投影 (THETA/SHIFT/FOV) 和
# background.wgsl galactic_py 同源常量, 三层 (底图光带 / 亮星带 / 暗星雾带)
# 对齐靠构造保证而非事后测量。Task 8 调角/带宽时三处同步回填。
# 灰度防线 (spec): 不烘焙可分辨亮星 — 星点雾全部单像素、增量封顶,
# 可分辨亮星一律来自运行时星野纹理层 (烘焙星不闪 = 穿帮)。
# ---------------------------------------------------------------------------


def galactic_pxy(u: float, v: float, theta_deg: float, shift: tuple) -> tuple[float, float]:
    """画面 UV → 银道面坐标 (px 沿带 / py 跨带), wgsl galactic_py 的 Python 镜像。

    py=0 ⟺ 银道面 (光带中心线); px 沿带, 银心方向在 px = (0 - L_CENTER)/FOV_U。
    """
    t = math.radians(theta_deg)
    rx = u - 0.5 - shift[0]
    ry = 0.5 - v + shift[1]
    px = rx * math.cos(t) + ry * math.sin(t)
    py = -rx * math.sin(t) + ry * math.cos(t)
    return px, py


def _lcg(seed: int):
    """与 build_ridges/build_embers 同款的确定性 LCG 序列。"""
    state = seed
    while True:
        state = (state * 1103515245 + 12345) & 0x7FFFFFFF
        yield (state >> 16) / 32768.0


def _value_noise(cols: int, rows: int, seed: int) -> tuple[list, int, int]:
    g = _lcg(seed)
    return [next(g) for _ in range(cols * rows)], cols, rows


def _noise_at(noise: tuple, x: float, y: float) -> float:
    """双线性 + smoothstep 插值采样; x∈[0,cols), y∈[0,rows)。"""
    grid, cols, rows = noise
    x = min(max(x, 0.0), cols - 1e-4)
    y = min(max(y, 0.0), rows - 1e-4)
    x0, y0 = int(x), int(y)
    x1, y1 = min(x0 + 1, cols - 1), min(y0 + 1, rows - 1)
    fx = (x - x0) ** 2 * (3.0 - 2.0 * (x - x0))
    fy = (y - y0) ** 2 * (3.0 - 2.0 * (y - y0))
    a = grid[y0 * cols + x0]
    b = grid[y0 * cols + x1]
    c = grid[y1 * cols + x0]
    d = grid[y1 * cols + x1]
    return lerp(lerp(a, b, fx), lerp(c, d, fx), fy)


def _fbm(noises: list, x: float, y: float, weights: tuple = (0.5, 0.3, 0.2)) -> float:
    """多倍频值噪声叠加; noises 为 [(noise, sx, sy), ...], 返回 ~[0,1]。"""
    return sum(
        w * _noise_at(noise, x * sx, y * sy)
        for (noise, sx, sy), w in zip(noises, weights)
    )


def _smoothstep(a: float, b: float, x: float) -> float:
    t = min(1.0, max(0.0, (x - a) / (b - a)))
    return t * t * (3.0 - 2.0 * t)


def fbm3(noises: list, x: float, y: float) -> float:
    """等权 N 倍频 fbm (len(noises) 归一), 无权重偏置, 返回 ~[0,1]。"""
    return sum(_noise_at(n, x * sx, y * sy) for n, sx, sy in noises) / len(noises)


def apply_milkyway(img: Image.Image, cfg: dict) -> Image.Image:
    """把银河烘焙进星夜底图 (RGBA in/out, alpha 恒 255)。

    合成顺序: ① 光带 screen 提亮 (低分辨率场 + LANCZOS 放大, 平滑元素惯例);
    ② 星点雾单像素加点 (native res, 增量封顶); ③ 尘埃暗隙 multiply 压暗
    (尘埃在星光之前, 暗隙同时吃掉光带与星点雾 — 物理次序)。
    """
    theta = cfg["theta_deg"]
    shift = cfg["shift"]
    half = cfg["band_half"]
    gc_px = cfg["gc_px"]
    falloff = cfg["along_falloff"]
    floor = cfg["band_floor"]
    gain = cfg["peak_gain"]

    # 低分辨率光带/尘埃场 (平滑元素, 惯例不 SS)。
    lw, lh = WIDTH // 4, HEIGHT // 4
    band_noises = [
        (_value_noise(24, 16, cfg["seed"] + 1), 6.0, 26.0),
        (_value_noise(48, 32, cfg["seed"] + 2), 12.0, 52.0),
        (_value_noise(96, 64, cfg["seed"] + 3), 24.0, 104.0),
    ]
    warp_noises = [
        (_value_noise(16, 12, cfg["seed"] + 31), 2.5, 9.0),
        (_value_noise(32, 24, cfg["seed"] + 32), 5.0, 18.0),
        (_value_noise(64, 48, cfg["seed"] + 33), 10.0, 36.0),
    ]
    dust_noises = [
        (_value_noise(24, 32, cfg["seed"] + 11), 4.0, 90.0),
        (_value_noise(48, 64, cfg["seed"] + 12), 8.0, 180.0),
        (_value_noise(96, 128, cfg["seed"] + 13), 16.0, 360.0),
    ]
    light = Image.new("RGB", (lw, lh))
    dust = Image.new("L", (lw, lh))
    light_px = light.load()
    dust_px = dust.load()
    for y in range(lh):
        v = (y + 0.5) / lh
        for x in range(lw):
            u = (x + 0.5) / lw
            px_, py = galactic_pxy(u, v, theta, shift)
            # 负半带 (px<0 左下段 / py<0 银道面下侧) 噪声贴第 0 列/行采样:
            # 单负半带结构只随单轴变化, 双负角区 (px<0 且 py<0) 两轴俱钉为
            # 恒定 = 沿带拉伸的平滑观感 — 2026-08-03
            # 用户多轮挑片裁定的审美, 显式保留 (评审曾作 Major 上报, 裁定为
            # 有意不对称; Task 8 调参注意两半带对频率参数的响应不同)。
            sx_, sy_ = max(px_, 0.0), max(py, 0.0)
            # 域扭曲: 中心线低频摆动 + 边缘羽化, 破探照灯式直边。
            warp = (fbm3(warp_noises, sx_, sy_) - 0.5) * 2.0 * cfg["warp_amp"]
            py_w = py + warp
            # 带宽沿路径起伏 (真实银河宽度不均, 破柏油马路感)。
            width_n = 0.85 + 0.55 * (fbm3(warp_noises, px_ * 0.6 + 3.7, 11.3) - 0.5) * 2.0
            half_w = half * min(1.45, max(0.60, width_n))
            cross = math.exp(-((py_w / half_w) ** 2))
            # 出画溶解: 带延伸到画布边时渐隐于天, 不成悬崖切片。
            edge = min(u, 1.0 - u, v, 1.0 - v)
            dissolve = _smoothstep(0.0, 0.07, edge)
            along_n = 0.25 + 0.50 * fbm3(band_noises[:1], max(px_ * 0.5, 0.0), 0.0)
            along = (floor + (1.0 - floor) * math.exp(-(((px_ - gc_px) / falloff) ** 2))) * along_n
            # 带内斑驳结构 (亮星云气块状感)。
            struct = 0.42 + 1.15 * max(0.0, _fbm(band_noises, sx_, sy_) - 0.26)
            intensity = min(1.0, cross * along * struct * gain * dissolve)
            warm_mix = math.exp(-(((px_ - gc_px) / 0.09) ** 2)) * math.exp(-((py_w / (half * 0.55)) ** 2))
            color = lerp_rgb(cfg["core_color"], cfg["warm_color"], warm_mix)
            light_px[x, y] = (
                int(color[0] * intensity),
                int(color[1] * intensity),
                int(color[2] * intensity),
            )
            # 尘埃暗隙: 中线偏移的窄带 × 脊状噪声, 越靠银心越强 (大暗隙)。
            lane = math.exp(-(((py_w - cfg["dust_offset"]) / cfg["dust_width"]) ** 2))
            ridge = max(0.0, (_fbm(dust_noises, sx_, sy_) - 0.48) * 2.4)
            dk = min(1.0, lane * ridge * (0.55 + 0.45 * along) * cfg["dust_strength"] * dissolve)
            dust_px[x, y] = int(255 * (1.0 - dk))

    light = light.resize(SIZE, Image.LANCZOS).filter(ImageFilter.GaussianBlur(radius=3))
    dust = dust.resize(SIZE, Image.LANCZOS).filter(ImageFilter.GaussianBlur(radius=2.5))

    rgb = ImageChops.screen(img.convert("RGB"), light)

    # 星点雾: 单像素, 密度沿银道面聚集, 增量封顶 (灰度防线)。
    g = _lcg(cfg["seed"] + 21)
    cap = cfg["haze_alpha_max"]
    pix = rgb.load()
    placed = 0
    attempts = cfg["haze_count"] * 3
    for _ in range(attempts):
        if placed >= cfg["haze_count"]:
            break
        u, v = next(g), next(g)
        _, py = galactic_pxy(u, v, theta, shift)
        accept = 0.06 + 0.94 * math.exp(-((py / (half * 1.5)) ** 2))
        if next(g) > accept:
            continue
        delta = int(14 + next(g) * (cap - 14))
        tint = 0.88 + next(g) * 0.12  # 蓝白微调 (0.88~1.0)
        x, y = int(u * (WIDTH - 1)), int(v * (HEIGHT - 1))
        r, gb, b = pix[x, y]
        pix[x, y] = (
            min(255, r + int(delta * tint)),
            min(255, gb + int(delta * (0.5 + tint / 2))),
            min(255, b + delta),
        )
        placed += 1

    if placed < cfg["haze_count"]:
        print(f"       WARNING: 星点雾投放 {placed}/{cfg['haze_count']} (接受率不足)")
    rgb = ImageChops.multiply(rgb, dust.convert("RGB"))
    out = rgb.convert("RGBA")

    # 自检输出: 带心 vs 带外亮度采样 (供挑片数值参照)。
    # 注意: (0.587, 0.320) 是银心在 theta/shift 下的反投影, Task 8 调角须联动。
    def lum_at(u: float, v: float) -> float:
        return luminance(out.convert("RGB").getpixel((int(u * (WIDTH - 1)), int(v * (HEIGHT - 1)))))

    print(
        f"       银河: 星点雾 {placed} 颗 (增量≤{cap}); "
        f"带心(GC) lum={lum_at(0.587, 0.320):.4f} vs "
        f"带外 lum={lum_at(0.12, 0.12):.4f}"
    )
    return out


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
        # AI 生成底图 (元宝, 2026-08-04): 写实松林 + 晨雾 + 暖色天光。
        # 替换原程序化三角形树; 动效 (forest_mist) 仍由运行时 shader 渲染。
        "ai_base": "forest_yuanbao_clean.png",
        # 渐变/光晕/暗纱仍保留, 用于统一风格与对比度护栏。
        "stops": [
            (0.00, (168, 185, 171)),
            (0.30, (126, 146, 130)),
            (0.55, (82, 104, 88)),
            (0.80, (50, 72, 59)),
            (1.00, (34, 52, 43)),
        ],
        # 顶部天光 (穿雾), 克制峰值避免中央采样区过亮。
        "glow": {"color": (214, 228, 214), "center": (0.5, 0.10), "radius": 0.42, "peak": 45},
        # 暗纱加强: AI 底图天空较亮, 需要更强中央压暗保对比度 (大字 ≥3:1)。
        "veil": {"color": (8, 14, 10), "center": (0.5, 0.42), "radius": 0.65, "peak": 95},
        # 雾不烘焙: 运行时 shader 渲染 (forest_mist)。
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
        # 深靛蓝夜空 → 暗地; 中央保持暗, 保白字对比度。星点不烘焙 —
        # 运行时由 shader 程序化渲染 (雨场景范式, starry_base 常驻)。
        "stops": [
            (0.00, (10, 12, 30)),
            (0.45, (22, 26, 52)),
            (0.72, (38, 42, 74)),
            (1.00, (16, 18, 40)),
        ],
        "ridges": [
            {"base_y": 0.88, "amp": 0.06, "color": (12, 14, 32), "alpha": 235, "seed": 0x501},
            {"base_y": 0.97, "amp": 0.05, "color": (8, 9, 22), "alpha": 255, "seed": 0x502},
        ],
        "veil": {"color": (0, 0, 0), "center": (0.5, 0.48), "radius": 0.55, "peak": 50},
        # 银河光带 + 尘埃暗隙 + 星点雾 (Task 7, spec: pomodoro-scene-starry-milkyway)。
        # theta/shift/band_half/gc_px 与 export-stars.py 投影、background.wgsl
        # galactic_py 同源 — Task 8 调参时三处同步, 不得只改一处。
        "milkyway": {
            "theta_deg": 60.0,      # = export-stars.py THETA_DEG
            "shift": (0.0, -0.03),  # = export-stars.py SHIFT_X/Y
            "band_half": 0.10,      # 跨带高斯半宽 (py) = wgsl HAZE_BAND
            "gc_px": 45.0 / 260.0,  # 银心沿带坐标 = (0 - L_CENTER) / FOV_U
            "along_falloff": 0.17,  # 沿带衰减 (px): 最亮段压银心 (UV.y≈0.32, 上 1/3)
            "band_floor": 0.45,     # 远银心残余亮度 (带仍斜跨全天)
            "peak_gain": 0.74,      # 光带峰值增益 (screen 提亮上限)
            "core_color": (212, 208, 236),  # 带核心 (淡紫白, 冷)
            "warm_color": (255, 226, 186),  # 银心暖调
            "dust_offset": 0.025,   # 暗隙中线 py 偏移 (银道面一侧)
            "dust_width": 0.045,    # 暗隙跨带宽度 (py)
            "dust_strength": 1.0,  # 暗隙压暗强度
            "warp_amp": 0.055,    # 中心线域扭曲幅度 (py): 破直边, 羽化带缘
            "haze_count": 24000,    # 星点雾颗数 (单像素, 不可分辨)
            "haze_alpha_max": 72,   # 星点雾增量封顶 (灰度防线: 不许可分辨亮星)
            "seed": 0x57A9,
        },
        "palette": {
            "base": (22, 26, 52),
            "accent": (255, 224, 160),
            "text_primary": (246, 247, 255),
            "text_secondary": (185, 192, 220),
            "surface": ((0, 0, 0), 0.25),
            "surface_input": ((0, 0, 0), 0.38),
        },
    },
]


def build_scene(cfg: dict) -> Image.Image:
    # AI 底图模式: 加载预制 AI 生成图, 跳过程序化渐变/树。
    if "ai_base" in cfg:
        ai_path = OUT_DIR / cfg["ai_base"]
        if not ai_path.exists():
            raise FileNotFoundError(f"AI 底图不存在: {ai_path}")
        img = Image.open(ai_path).convert("RGBA")
        # 统一尺寸 (容错: AI 生图可能不是精确 1536x1024)
        if img.size != SIZE:
            img = img.resize(SIZE, Image.LANCZOS)
    else:
        img = build_gradient(cfg["stops"]).convert("RGBA")

    if "glow" in cfg:
        g = cfg["glow"]
        img = Image.alpha_composite(
            img, radial_overlay(g["color"], g["center"], g["radius"], g["peak"])
        )
    if "milkyway" in cfg:
        # 银河在山脊之前烘焙: 山脊剪影遮挡带底 (带自山后升起)。
        img = apply_milkyway(img, cfg["milkyway"])
    if "ridges" in cfg:
        img = Image.alpha_composite(img, build_ridges(cfg["ridges"]))
    if "waves" in cfg:
        img = Image.alpha_composite(img, build_waves(cfg["waves"]))
    if "trees" in cfg:
        img = Image.alpha_composite(img, build_trees(cfg["trees"]))
    if "embers" in cfg:
        e = cfg["embers"]
        img = Image.alpha_composite(img, build_embers(e["count"], e["color"], e["seed"]))
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
        assert_eq!(SCENES.len(), 6, "沉浸世界应有 6 个场景");
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
