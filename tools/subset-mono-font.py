#!/usr/bin/env python3
# @author 十四叔
# @date 2026/09/06
"""从 Sarasa Mono SC (静态 TTF) 导出 assets/fonts/ofl-mono.ttf (GB2312 子集)。

与 subset-font.py 同一工艺, 但源字是**静态 TTF** (Sarasa), 无需先实例化可变字重
(fontdue 不解析变体; Sarasa 静态 SemiBold 直接子集)。

用法:
    python tools/subset-mono-font.py [源字体路径]

源字体默认: _font_tmp 解出的 SarasaMonoSC-SemiBold.ttf; 若要重新下载, 见
tasks/plan-font-cjk-mono.md T1。字重 SemiBold (2026-09-06 pivot):
fontdue 解析式 AA 超出小字号锐化能力, 超采样不兑现 → 改粗字重增笔画对比
(Medium 在 Sarasa 无此档, 用 SemiBold 替)。

字符集: tools/gb2312-charset.txt (6763 常用字 + 95 ASCII + 标点) + 补空格。
依赖: pip install fonttools
"""

import sys
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_SOURCE = REPO_ROOT.parent / "_font_tmp" / "SarasaMonoSC-SemiBold.ttf"
CHARSET_FILE = REPO_ROOT / "tools" / "gb2312-charset.txt"
OUTPUT_FILE = REPO_ROOT / "assets" / "fonts" / "ofl-mono.ttf"
SIZE_LIMIT = 3 * 1024 * 1024  # 3 MB, 与 tests/assets.rs 断言一致


def main() -> None:
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_SOURCE
    if not source.exists():
        sys.exit(f"源字体不存在: {source} (先跑 T1 下载解出 SarasaMonoSC-Regular.ttf)")

    charset = CHARSET_FILE.read_text(encoding="utf-8")
    # 补 blank: 日志行空格常见, charset 未含; 一并纳入子集。
    charset += " \t"
    print(f"字符集: {len(charset)} 字, 源字体: {source}")

    # 静态字体验证 (可被 fontdue 打开的前提: 合法 TTF)。
    TTFont(source)
    subset.main(
        [
            str(source),
            f"--text={charset}",
            f"--output-file={OUTPUT_FILE}",
            "--no-hinting",
            "--desubroutinize",
        ]
    )

    size = OUTPUT_FILE.stat().st_size
    print(f"输出: {OUTPUT_FILE} ({size / 1024 / 1024:.2f} MB)")
    if size > SIZE_LIMIT:
        sys.exit("超过 3 MB 上限, 请收缩字符集")

    # 校验关键字形在子集后仍在。
    check = TTFont(OUTPUT_FILE)
    cmap = check.getBestCmap()
    for c in "你的丹青过滤·—123abcABC":
        if ord(c) not in cmap:
            print(f"警告: 字形缺失 {c!r}")


if __name__ == "__main__":
    main()
