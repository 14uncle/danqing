#!/usr/bin/env python3
"""Export the Yale Bright Star Catalog (BSC5) into danqing's compact star binary.

Pipeline (spec: docs/specs/pomodoro-scene-starry-milkyway.md):
    ybsc5.gz (ASCII, 197-byte records, NASA ADC via Harvard CfA mirror)
        -> parse GLON/GLAT/Vmag/B-V (catalog ships galactic coordinates,
           no RA/Dec -> galactic conversion needed)
        -> fixed observing attitude: window on galactic longitude, then a
           2D rotation so the Milky Way band runs lower-left -> upper-right
        -> cull stars outside the UV frame
        -> assets/stars.bin

Binary layout:
    header  8B: magic "DQST" + version u16 + count u16 (little-endian)
    record  6B: x u16, y u16 (UV normalized 0..65535),
                vmag u8 (v = q/27 - 2.0), bv u8 (q = (bv+0.5)*85; 0xFF = none)

License (Task 1, 2026-08-03): BSC5 machine-readable edition was produced and
disseminated by NASA ADC (NSSDC/ADC, GSFC); star positions/magnitudes are
uncopyrightable facts; attribution to Hoffleit & Warren / NASA ADC is custom
and will be included in product credits. HYG (CC BY-SA 4.0) was rejected.
Raw download is cached under tools/.cache/ (gitignored) — only stars.bin
is committed.

Dependency: Python >= 3.10, stdlib only (gzip/urllib/struct/math).

Usage:
    python tools/export-stars.py
"""

import gzip
import math
import struct
import sys
import urllib.request
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CACHE_DIR = REPO_ROOT / "tools" / ".cache"
OUT_BIN = REPO_ROOT / "assets" / "stars.bin"

SOURCE_URL = "http://tdc-www.harvard.edu/catalogs/ybsc5.gz"
SOURCE_NAME = "ybsc5.gz"

# --- Fixed observing attitude (placeholder; Task 8 re-measures against the
#     final AI base image and back-fills these constants) ---
L_CENTER = -45.0    # deg: window center in galactic longitude (keeps the
                    # galactic-center bright segment in the upper right)
THETA_DEG = 60.0    # deg: band tilt, lower-left -> upper-right
FOV_U = 260.0       # deg: longitude span mapped to full width (overscan for rotation)
FOV_V = 150.0       # deg: latitude span mapped to full height
SHIFT_X = 0.0       # UV translation after rotation
SHIFT_Y = -0.03     # pushes the band up so GC lands in the upper third

MAGIC = b"DQST"
VERSION = 1

# Anchor stars for parse/projection self-checks (HR numbers from the catalog).
# (HR, label, expected GLON deg, expected GLAT deg, expected Vmag)
ANCHORS = [
    (7001, "织女 Vega",   67.45, +19.24, 0.03),
    (7557, "牛郎 Altair", 47.74, -8.91, 0.77),
    (2491, "天狼 Sirius", 227.23, -8.89, -1.44),
]
ANCHOR_TOL_DEG = 0.02
ANCHOR_TOL_MAG = 0.05


def download_cache() -> Path:
    """Fetch ybsc5.gz into tools/.cache/ if absent (raw data never committed)."""
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    path = CACHE_DIR / SOURCE_NAME
    if not path.exists():
        print(f"下载 {SOURCE_URL} -> {path.relative_to(REPO_ROOT)}")
        tmp = path.with_suffix(".tmp")
        urllib.request.urlretrieve(SOURCE_URL, tmp)  # 断网留 .tmp 不污染缓存
        tmp.replace(path)
    return path


def parse_catalog(path: Path) -> list[dict]:
    """Parse 197-byte fixed-length records. Blank fields = missing data."""
    stars = []
    with gzip.open(path, "rt", encoding="ascii", errors="replace") as f:
        for line in f:
            if len(line.rstrip("\n")) < 114:
                continue  # truncated record guard

            def field(lo: int, hi: int) -> str:
                return line[lo:hi].strip()

            hr_s = field(0, 4)
            glon_s = field(90, 96)
            glat_s = field(96, 102)
            vmag_s = field(102, 107)
            if not (hr_s and glon_s and glat_s and vmag_s):
                continue  # no position or no magnitude -> useless for rendering
            bv_s = field(109, 114)
            stars.append(
                {
                    "hr": int(hr_s),
                    "name": field(4, 14),
                    "glon": float(glon_s),
                    "glat": float(glat_s),
                    "vmag": float(vmag_s),
                    "bv": float(bv_s) if bv_s else None,
                }
            )
    return stars


def project(glon: float, glat: float) -> tuple[float, float] | None:
    """Fixed-attitude projection: longitude window -> 2D rotation -> UV.

    Returns None when the star falls outside the UV frame (culled).
    """
    u = ((glon - L_CENTER + 180.0) % 360.0) - 180.0  # deg, centered window
    v = glat
    px = u / FOV_U
    py = v / FOV_V
    t = math.radians(THETA_DEG)
    rx = px * math.cos(t) - py * math.sin(t)
    ry = px * math.sin(t) + py * math.cos(t)
    x = 0.5 + rx + SHIFT_X
    y = 0.5 - ry + SHIFT_Y  # screen y grows downward
    if 0.0 <= x <= 1.0 and 0.0 <= y <= 1.0:
        return (x, y)
    return None


def quant_vmag(v: float) -> int:
    return max(0, min(254, int((v + 2.0) * 27.0 + 0.5)))


def quant_bv(bv: float | None) -> int:
    if bv is None:
        return 0xFF
    return max(0, min(254, int((bv + 0.5) * 85.0 + 0.5)))


def ascii_density_map(kept: list[dict], cols: int = 96, rows: int = 24) -> str:
    """Terminal density map of kept stars — lets us eyeball the diagonal band
    without a GUI. Only bright stars (V<4) are plotted: with all stars the
    cells saturate and the band contrast washes out."""
    bright = [s for s in kept if s["vmag"] < 4.0]
    grid = [[0] * cols for _ in range(rows)]
    for s in bright:
        cx = min(cols - 1, int(s["x"] * cols))
        cy = min(rows - 1, int(s["y"] * rows))
        grid[cy][cx] += 1
    ramp = " .:*#"
    lines = []
    for row in grid:
        lines.append("".join(ramp[min(len(ramp) - 1, c)] for c in row))
    return f"(仅 V<4 亮星, {len(bright)} 颗)\n" + "\n".join(lines)


def check_anchors(by_hr: dict[int, dict]) -> list[str]:
    problems = []
    for hr, label, exp_lon, exp_lat, exp_mag in ANCHORS:
        s = by_hr.get(hr)
        if s is None:
            problems.append(f"锚点缺失: {label} (HR {hr}) 未解析到")
            continue
        if abs(s["glon"] - exp_lon) > ANCHOR_TOL_DEG or abs(s["glat"] - exp_lat) > ANCHOR_TOL_DEG:
            problems.append(
                f"{label}: 银道坐标 ({s['glon']:.2f},{s['glat']:.2f}) "
                f"与已知 ({exp_lon},{exp_lat}) 不符"
            )
        if abs(s["vmag"] - exp_mag) > ANCHOR_TOL_MAG:
            problems.append(f"{label}: V 星等 {s['vmag']:.2f} 与已知 {exp_mag} 不符")
        if s.get("uv") is None:
            problems.append(f"{label}: 投影后被剔除出画面 — 检查观测姿态常量")
    return problems


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    src = download_cache()
    stars = parse_catalog(src)
    by_hr = {s["hr"]: s for s in stars}

    problems = []
    if len(stars) < 9000:
        # 锚点完好但整表大面积丢记录(镜像内容变更)不能静默通过。
        problems.append(f"解析数 {len(stars)} < 9000 — 目录完整性存疑, 检查镜像源")

    kept, culled = [], 0
    for s in stars:
        uv = project(s["glon"], s["glat"])
        if uv is None:
            culled += 1
            continue
        s["uv"] = uv
        s["x"], s["y"] = uv
        kept.append(s)
    if not kept:
        problems.append("投影后保留 0 颗 — 观测姿态常量把视窗调飞了")

    # 银心方向 (l=0,b=0) 的落点 — 构图自检: 应在上三分之一、避开中央倒计时区。
    gc = project(0.0, 0.0)

    problems += check_anchors(by_hr)

    # --- 写 stars.bin ---
    OUT_BIN.parent.mkdir(parents=True, exist_ok=True)
    with open(OUT_BIN, "wb") as f:
        f.write(MAGIC)
        f.write(struct.pack("<HH", VERSION, len(kept)))
        for s in kept:
            xq = int(s["x"] * 65535.0 + 0.5)
            yq = int(s["y"] * 65535.0 + 0.5)
            f.write(
                struct.pack(
                    "<HHBB", xq, yq, quant_vmag(s["vmag"]), quant_bv(s["bv"])
                )
            )

    # --- 自检报告 ---
    if kept:
        mags = [s["vmag"] for s in kept]
        print(f"\n解析 {len(stars)} 颗 (目录共 9110 条记录), 保留 {len(kept)}, UV 外剔除 {culled}")
        print(f"星等范围: {min(mags):.2f} .. {max(mags):.2f}")
        hist = [0] * 8
        for m in mags:
            hist[min(7, max(0, int(m + 2)))] += 1
        for i, h in enumerate(hist):
            print(f"  星等 [{i - 2:>2},{i - 1:>2}): {'#' * (h * 60 // max(hist))} {h}")
    else:
        print(f"\n解析 {len(stars)} 颗, 保留 0 — 无统计可打印")
    print(f"银心 (l=0,b=0) 落点: {('(%.3f, %.3f)' % gc) if gc else '剔除!'}")
    for hr, label, *_ in ANCHORS:
        s = by_hr.get(hr)
        if s and s.get("uv"):
            print(f"{label}: UV ({s['x']:.3f}, {s['y']:.3f}), b = {s['glat']:+.2f}")
    print(f"\n星点密度图 ({len(kept)} 颗, 96x24):\n")
    print(ascii_density_map(kept))
    size = OUT_BIN.stat().st_size
    expect = 8 + 6 * len(kept)
    print(f"\n写出 {OUT_BIN.relative_to(REPO_ROOT)}: {size} B (预期 {expect} B)")
    if size != expect:
        problems.append(f"文件大小 {size} != 预期 {expect}")

    if problems:
        print("\n自检未过:", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        return 1
    print("自检通过。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
