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

## 处理参数

```
ffmpeg -i <源> -filter_complex \
  "[0:a]asplit[a][b];[a]atrim=0:0.05,asetpts=PTS-STARTPTS[start];\
   [b]atrim=0.05,asetpts=PTS-STARTPTS[rest];\
   [rest][start]acrossfade=d=0.05:c1=tri:c2=tri[out]" \
  -map "[out]" -c:a libvorbis -q:a 4 <目标>.ogg
```

## 备注

- `mountain.ogg` 源仅 ~6s, 循环短, 听感偏"阵风"; 若不合适可换
  (OpenGameArt 搜 `wind` 过滤 CC0, 或 Freesound CC0 山风)。
- `bonfire.ogg` 源 ~12.5s, 柴火噼啪随机性较强, 短循环可接受。
- 换源后请同步更新本表并重跑 `tests/assets.rs` 体积护栏。
