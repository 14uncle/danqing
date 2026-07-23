#!/usr/bin/env python3
# @author 十四叔
# @date 2026/07/22
"""从 Noto Sans SC 可变字体导出 assets/fonts/ofl-sans.ttf (GB2312 子集)。

用法:
    python tools/subset-font.py [源字体路径] [字重]

源字体默认为 Windows 自带的 C:/Windows/Fonts/NotoSansSC-VF.ttf (OFL 许可)。
字重默认 350: 介于 Light 与 Regular 之间, 兼顾雅黑式细腻与正文可读性;
300 偏轻, 400 在浅色主题下发黑。
字符集取自 tools/gb2312-charset.txt (GB2312 全覆盖 + 常用标点)。
依赖: pip install fonttools
"""

import sys
import tempfile
from pathlib import Path

from fontTools import subset
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.ttLib import TTFont

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_SOURCE = Path("C:/Windows/Fonts/NotoSansSC-VF.ttf")
CHARSET_FILE = REPO_ROOT / "tools" / "gb2312-charset.txt"
OUTPUT_FILE = REPO_ROOT / "assets" / "fonts" / "ofl-sans.ttf"
DEFAULT_WEIGHT = 350
SIZE_LIMIT = 3 * 1024 * 1024  # 3 MB, 与 tests/assets.rs 断言一致


def main() -> None:
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_SOURCE
    weight = int(sys.argv[2]) if len(sys.argv) > 2 else DEFAULT_WEIGHT
    if not source.exists():
        sys.exit(f"源字体不存在: {source}")

    charset = CHARSET_FILE.read_text(encoding="utf-8")
    print(f"字符集: {len(charset)} 字, 源字体: {source}, 字重: {weight}")

    with tempfile.NamedTemporaryFile(suffix=".ttf", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    # 可变字体先实例化为静态字重, fontdue 不解析变体。
    # 注意: inplace=False (默认) 返回新字体对象, 原对象不变。
    font = TTFont(source)
    font = instantiateVariableFont(font, {"wght": weight})
    font.save(tmp_path)

    subset.main(
        [
            str(tmp_path),
            f"--text={charset}",
            f"--output-file={OUTPUT_FILE}",
            "--no-hinting",
            "--desubroutinize",
        ]
    )
    tmp_path.unlink()

    size = OUTPUT_FILE.stat().st_size
    print(f"输出: {OUTPUT_FILE} ({size / 1024 / 1024:.2f} MB)")
    if size > SIZE_LIMIT:
        sys.exit("超过 3 MB 上限, 请收缩字符集")


if __name__ == "__main__":
    main()
