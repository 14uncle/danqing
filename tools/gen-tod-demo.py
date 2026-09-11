# @author 十四叔
# @date 2026/08/30
# 时辰调色 + 双蒙版演示资产 (showcase 时辰卡用, scene-world Task 4 以用代测)。
# 底图是中性日光态 (不烤夜景) —— 时辰变化全部由 tint/sky/glow 参数驱动,
# 这正是引擎能力的演示点。用法: python tools/gen-tod-demo.py

from pathlib import Path

from PIL import Image, ImageDraw

W, H = 1536, 1024
OUT = Path(__file__).resolve().parent.parent / "assets" / "background"

HORIZON = 620
CABIN_X, CABIN_Y = 480, 560
CABIN_W, CABIN_H = 560, 300
ROOF_H = 130
WIN_A = (CABIN_X + 90, CABIN_Y + 110, 130, 120)
WIN_B = (CABIN_X + 330, CABIN_Y + 110, 130, 120)
LAMP = (CABIN_X + CABIN_W - 60, CABIN_Y + 60, 28, 44)


def lerp(a: int, b: int, t: float) -> int:
    return int(a + (b - a) * t)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)

    # 底图: 中性日光 (天蓝渐变 + 地面 + 小屋; 窗玻璃暗灰未点亮态)
    img = Image.new("RGB", (W, H))
    d = ImageDraw.Draw(img)
    for y in range(HORIZON):
        t = y / HORIZON
        d.line([(0, y), (W, y)], fill=(lerp(150, 196, t), lerp(178, 214, t), lerp(205, 228, t)))
    d.rectangle([0, HORIZON, W, H], fill=(96, 104, 88))
    d.rectangle([CABIN_X, CABIN_Y, CABIN_X + CABIN_W, CABIN_Y + CABIN_H], fill=(140, 108, 82))
    d.polygon(
        [(CABIN_X - 30, CABIN_Y), (CABIN_X + CABIN_W + 30, CABIN_Y),
         (CABIN_X + CABIN_W // 2, CABIN_Y - ROOF_H)],
        fill=(90, 66, 52),
    )
    for (x, y, w, h) in (WIN_A, WIN_B):
        d.rectangle([x, y, x + w, y + h], fill=(52, 58, 66))  # 未点亮窗玻璃
        d.rectangle([x, y, x + w, y + h], outline=(30, 24, 18), width=6)
        d.line([(x + w // 2, y), (x + w // 2, y + h)], fill=(30, 24, 18), width=4)
    lx, ly, lw, lh = LAMP
    d.ellipse([lx, ly, lx + lw, ly + lh], fill=(64, 62, 58))  # 未点亮廊灯
    img.save(OUT / "tod-demo.png")

    # 天空蒙版: 天空白, 地面黑, 天际线 ±40px 渐变
    mask = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(mask)
    for y in range(HORIZON + 40):
        t = min(1.0, max(0.0, (HORIZON + 40 - y) / 80))
        d.line([(0, y), (W, y)], fill=int(255 * t))
    mask.save(OUT / "tod-demo-sky.png")

    # 发光蒙版: 窗 A=255 (先亮) / 窗 B=210 / 廊灯=160 (后亮)
    mask = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(mask)
    for rect, level in ((WIN_A, 255), (WIN_B, 210), (LAMP, 160)):
        x, y, w, h = rect
        d.rectangle([x, y, x + w, y + h], fill=level)
    mask.save(OUT / "tod-demo-glow.png")

    print(f"tod demo assets -> {OUT}")


if __name__ == "__main__":
    main()
