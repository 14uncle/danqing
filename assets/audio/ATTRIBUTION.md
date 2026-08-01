# 环境音资产来源与许可 (Attribution)

番茄钟 5 场景环境音, 全部来自 OpenGameArt 的 **CC0** (Public Domain) 素材,
2026-07-27 选取。处理: ffmpeg 循环点 50ms 微 crossfade 消接缝 + OGG Vorbis q4 转码
(内容未做其他改动)。CC0 无需署名, 此处记录出处备查。

| 文件 | 场景 | 原作 | 作者 | 来源 | 许可 |
|------|------|------|------|------|------|
| `bonfire.ogg` | 篝火 | Fireplace Sound loop (`fire.wav`) | PagDev | https://opengameart.org/content/fireplace-sound-loop | CC0 |
| `sea.ogg` | 海 | Sea and river wave sounds (`VistulaShort_0.mp3`) | RandomMind | https://opengameart.org/content/sea-and-river-wave-sounds | CC0 |
| `rain.ogg` | 雨 | AMB Rain Loop 1 (`amb_rain_loop_1.ogg`) | Kresiek The Furry | https://opengameart.org/content/amb-rain-loop-1 | CC0 |
| `mountain.ogg` | 山 | wind whoosh loop (`wind woosh loop.ogg`) | SketchMan3 | https://opengameart.org/content/wind-whoosh-loop | CC0 |
| `forest.ogg` | 森林 | Forest Ambience (`Forest_Ambience.mp3`) | TinyWorlds | https://opengameart.org/content/forest-ambience | CC0 |

## 程序化音景 (2026-08-01, 9 场景补全)

`starry` / `snowfield` / `desert` / `cloudsea` 四条为 **`tools/export-ambient.py` 程序化合成**的「风系」音景
(星夜=夜风, 雪原=雪风, 沙漠=干风, 云海=高空风)。FFT 频谱整形噪声 + 缓阵风包络 + 接缝 crossfade,
确定性、零外部资产、循环安全。**初版占位** —— 若终审听感不佳, 可换 OpenGameArt CC0 源
(处理参数与下方一致) 后同步更新本表与 `tests/assets.rs` 体积护栏。

| 文件 | 场景 | 生成方式 | 频谱 | RMS 目标 |
|------|------|----------|------|----------|
| `starry.ogg` | 星夜 | 程序化 (`tools/export-ambient.py`) | 深低鸣 + 微空气感 | 0.05 |
| `desert.ogg` | 沙漠 | 程序化 (同上) | 干爽中段风声 | 0.08 |

## 处理参数

```
ffmpeg -i <源> -filter_complex \
  "[0:a]volume=<增益>dB,asplit[a][b];[a]atrim=0:0.05,asetpts=PTS-STARTPTS[start];\
   [b]atrim=0.05,asetpts=PTS-STARTPTS[rest];\
   [rest][start]acrossfade=d=0.05:c1=tri:c2=tri[out]" \
  -map "[out]" -c:a libvorbis -q:a 4 <目标>.ogg
```

响度统一 (2026-07-28): 源响度差异极大 (-13.8 ~ -46.4 LUFS), 按静态增益归一到
**-28 LUFS** (以雨/山听感为锚): 篝火 +12.7dB / 海 -14.2dB / 雨 +1.7dB / 森林 +18.4dB /
山 0dB (未动)。增益后峰值均 ≤ -3.3dBFS, 无削波。

## 备注

- `mountain.ogg` 源仅 ~6s, 循环短, 听感偏"阵风"; 若不合适可换
  (OpenGameArt 搜 `wind` 过滤 CC0, 或 Freesound CC0 山风)。
- `bonfire.ogg` 源 ~12.5s, 柴火噼啪随机性较强, 短循环可接受。
- 换源后请同步更新本表并重跑 `tests/assets.rs` 体积护栏。
