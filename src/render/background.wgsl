// 窗口背景图渲染管线: 绘制全屏或等比缩放图片。
//
// 顶点携带归一化位置 (0..1) 与 UV;uniform 传递不透明度与场景淡化进度。
// 场景切换时绑定 from/to 两张场景图, 按 fade 交叉淡化;
// 单图与叠加层 (光晕/噪声) 把同一张图绑到两个槽位, fade 恒 0。
//
// uniform 携带场景动效参数 (雨丝强度 + 动效时间 + 篝火强度 + 海强度 + 山强度 + 森林强度);
// 各效果强度为 0 时零贡献, 输出与静态逐像素一致。
// 雨、火、海、山、森林是并存标量而非互斥选择子: 交叉淡化期间两端可同时非零。

struct Uniforms {
    opacity: f32,
    fade: f32,
    rain_intensity: f32,
    time: f32,
    fire_intensity: f32,
    sea_intensity: f32,
    rain_time: f32,
    mountain_intensity: f32,
    forest_intensity: f32,
    starry_intensity: f32,
    starry_base: f32,
    screen_w: f32,
    screen_h: f32,
    blacksmith_intensity: f32,
    cave_intensity: f32,
    nightmarket_intensity: f32,
    train_intensity: f32,
    // ---- 时辰调色 + 双蒙版 (桌景 scene-world 天级「活的时间」) ----
    // 全 f32 紧排 (本 struct 无 vec3, 避开 uniform 16B 对齐位错;
    // Rust 侧 cast_slice 同序 24 字段, 护栏测试锁字段数)。
    tod_tint_r: f32,       // 时辰色调 R (乘性; 1.0 = 原样)
    tod_tint_g: f32,
    tod_tint_b: f32,
    tod_brightness: f32,   // 时辰亮度乘性
    tod_saturation: f32,   // 时辰饱和度乘性 (0 = 灰度)
    sky_amount: f32,       // 天空蒙版夜空化进度 0..1 (0 = 天空原样)
    glow_amount: f32,      // 发光蒙版点亮进度 0..1 (灰度阈值错落亮灯)
    // ---- 微事件 (桌景 scene-world 分钟级; 包络由产品调度器算好, 本层只播放) ----
    event_firefly: f32,    // 萤火虫包络 0..1 (程序化萤绿光点, 免资产)
    flash_intensity: f32,  // 闪电亮度包络 0..1 (全屏 additive 蓝白)
}

// ---- 雨幕 (雨场景; 静态图已去丝, 雨全部由本段程序化渲染) ----
// 2026-07-29 用户裁定: 静态背景图不烘焙雨丝 (export-scenes.py 雨配置去 streaks),
// 运行时本段三层雨丝即全部雨效 — 计时运行下落, 暂停雨钟冻结、雨丝定格可见。
// 参数集中于本段, 调参只动这里。三层速度取整数比 (0.125/0.25/0.375 周期/秒),
// 公共周期 8s, 与 Rust 侧 `RAIN_WRAP_SECS` 一致 (上传前取模, 保 f32 精度)。
const RAIN_SLANT: f32 = 0.12;        // 斜率: 雨落朝右下 (\ 形), 与原静态雨图一致
const RAIN_YSCALE: f32 = 0.5;        // 纵向压缩: 同屏每列最多一段雨丝
const RAIN_GAIN: f32 = 0.20;         // 总亮度上限 (线性空间 additive)

// 丝宽为 y 循环空间单位, 屏高占比 ≈ 丝宽 × 2.5 (尾羽) / YSCALE。
// 密度与有雨列门槛对照 (去丝后雨幕独挑, 列数对齐原静态图 ~300 丝的观感密度,
// 丝宽保终审裁定 2~3px: 列密度 480/360/320 ≈ 2.0/2.7/3.0px @960px 窗)。
const RAIN_DENSITY_FAR: f32 = 480.0; // 远层: 密、细、慢、淡
const RAIN_SPEED_FAR: f32 = 0.125;
const RAIN_WIDTH_FAR: f32 = 0.02;    // 尾羽占屏高 ~10%
const RAIN_BRIGHT_FAR: f32 = 0.16;
const RAIN_ON_FAR: f32 = 0.70;       // hash > 此值的列才有雨 (~144 列有雨)

const RAIN_DENSITY_MID: f32 = 360.0; // 中层
const RAIN_SPEED_MID: f32 = 0.25;
const RAIN_WIDTH_MID: f32 = 0.025;   // 尾羽占屏高 ~12%
const RAIN_BRIGHT_MID: f32 = 0.22;
const RAIN_ON_MID: f32 = 0.72;       // ~100 列

const RAIN_DENSITY_NEAR: f32 = 320.0; // 近层: 疏、粗、快、亮
const RAIN_SPEED_NEAR: f32 = 0.375;
const RAIN_WIDTH_NEAR: f32 = 0.03;   // 尾羽占屏高 ~15%
const RAIN_BRIGHT_NEAR: f32 = 0.30;
const RAIN_ON_NEAR: f32 = 0.85;      // ~48 列

fn rain_hash(p: f32) -> f32 {
    return fract(sin(p * 127.1) * 43758.5453);
}

// 单层雨丝: density 列密度, speed 下落速度 (fract 周期/秒),
// width 丝头宽度, bright 亮度权重, on 有雨列的 hash 门槛。
fn rain_layer(
    uv: vec2<f32>,
    t: f32,
    density: f32,
    speed: f32,
    width: f32,
    bright: f32,
    on: f32,
) -> f32 {
    let x = uv.x - uv.y * RAIN_SLANT; // 斜向拉条 (\ 形, 朝右下)
    let col = floor(x * density);
    let rnd = rain_hash(col * 1.37);
    // 相位随机 (常量), 速度全列一致: 雨的真实感来自同速不同相,
    // 同时保证公共周期成立 (速度不带逐列抖动)。
    let y = fract(uv.y * RAIN_YSCALE - t * speed + rnd * 7.0);
    // 近似均匀亮度的一段丝 (亮头长尾会读出流星/烟花感, 尾羽刻意短)。
    let streak = smoothstep(0.0, width, y) * (1.0 - smoothstep(width, width * 2.5, y));
    let visible = step(on, rain_hash(col * 3.1 + 17.0));
    return streak * visible * bright;
}

fn rain_overlay(uv: vec2<f32>, t: f32) -> f32 {
    var acc = rain_layer(uv, t, RAIN_DENSITY_FAR, RAIN_SPEED_FAR, RAIN_WIDTH_FAR, RAIN_BRIGHT_FAR, RAIN_ON_FAR);
    acc += rain_layer(uv, t, RAIN_DENSITY_MID, RAIN_SPEED_MID, RAIN_WIDTH_MID, RAIN_BRIGHT_MID, RAIN_ON_MID);
    acc += rain_layer(uv, t, RAIN_DENSITY_NEAR, RAIN_SPEED_NEAR, RAIN_WIDTH_NEAR, RAIN_BRIGHT_NEAR, RAIN_ON_NEAR);
    return min(acc, 1.0) * RAIN_GAIN;
}

// ---- 篝火动效 (篝火场景) ----
// UV 位移: 火焰区域采样坐标偏移, 底图纹理自身舞动 (参考 sea_swell 范式)。
// 余烬粒子保留作为微量点缀。参数集中于本段, 调参只动这里。
// 所有频率/速度取 1/8 Hz 整数倍, 与雨共用 8s 公共周期 (Rust 侧 MOTION_WRAP_SECS)。
const FIRE_W: f32 = 0.7853982;         // 2π/8: 动效基频角速度 (1/8 Hz)
const FIRE_CENTER: vec2<f32> = vec2<f32>(0.4093, 0.3315); // 位移中心 (火焰尖端)
const FIRE_MASK_RADIUS: f32 = 0.03;    // 位移径向衰减半径 (uv, 只包住火焰尖)

// 余烬: 分列 hash, 每列一颗, 相位随机、速度全列一致 (保公共周期)。
const EMBER_DENSITY: f32 = 80.0;       // 列密度 (960px 窗 ≈ 12px/列; 加密, 增火星数)
const EMBER_SPEED: f32 = 0.25;         // 上浮速度 (循环/秒; 放慢, 悠然浮动)
const EMBER_Y: f32 = 0.598;            // 发射原点 y (0.50+100px≈0.098, 下移对齐柴堆底)
const EMBER_SPAN: f32 = 0.345;         // 行程: 0.15+200px≈0.195, 升至 y≈0.253
const EMBER_RADIUS: f32 = 0.0025;      // 点半径 (纵向 uv; 960px 窗 ≈ 3px 直径)
const EMBER_HEIGHT_MIN: f32 = 1.3;     // 纵向拉伸最小倍率 (短余烬, 微椭)
const EMBER_HEIGHT_MAX: f32 = 2.0;     // 纵向拉伸最大倍率 (长余烬, 尾部微翘)
const EMBER_SWAY: f32 = 0.002;         // 横摆幅度 (uv ≈ 2px; 收窄, 聚焦柴堆)
const EMBER_BRIGHT: f32 = 0.90;        // 点亮度上限 (additive; 略提亮)
const EMBER_ON: f32 = 0.66;            // hash > 此值的列才有余烬 (~27 列, 带内 ~15-18 颗)
const EMBER_COLOR: vec3<f32> = vec3<f32>(1.0, 0.28, 0.06); // 深红余烬色

// 余烬层: 自底部升起, 横向轻摆, 随行程 (life) 淡出。
// 形状: 纵向拉伸椭圆 (高>宽), 逐粒子随机高宽比 + 尖尾上翘, 模拟真实余烬。
fn ember_layer(uv: vec2<f32>, t: f32) -> f32 {
    let col = floor(uv.x * EMBER_DENSITY);
    let rnd = rain_hash(col * 1.37 + 53.0); // 与雨不同种子, 避免位置相关
    let on = step(EMBER_ON, rain_hash(col * 3.1 + 71.0));
    // 横摆频率取档位 {1,2,3}/8 Hz (整数倍, 保 8s 公共周期)。
    let k = 1.0 + floor(rnd * 3.0);
    let cx = (col + 0.5) / EMBER_DENSITY + sin(t * FIRE_W * k + rnd * 6.2831853) * EMBER_SWAY;
    let life = fract(t * EMBER_SPEED + rnd * 7.0); // 0=点燃(底部) → 1=熄灭(顶端)
    let cy = EMBER_Y - life * EMBER_SPAN;
    // 发射带: 聚焦柴堆区域, 带心 x≈0.38, 带外柔裁。
    let band = smoothstep(0.24, 0.28, cx) * (1.0 - smoothstep(0.48, 0.57, cx));
    // 逐粒子纵向拉伸倍率 (1.3~2.0 倍), 高宽比随 life 递增 (上翘尾)。
    let hscale = mix(EMBER_HEIGHT_MIN, EMBER_HEIGHT_MAX, rnd);
    let tail = 1.0 + life * 0.5; // 尾部微翘, 不过度拉伸
    // 椭圆距离: dy 除以拉伸因子 → 纵向延伸, 横向保持 (水滴/余烬形)。
    let dx = uv.x - cx;
    let dy = (uv.y - cy) / (hscale * tail);
    let d = sqrt(dx * dx + dy * dy);
    let spot = 1.0 - smoothstep(EMBER_RADIUS * 0.4, EMBER_RADIUS, d);
    // 亮度: 底部亮 → 顶部暗 (头部核心, 尾部余韵), 叠低频闪烁。
    let fade = (1.0 - life * 0.6) * (0.7 + 0.3 * sin(t * FIRE_W * 4.0 + rnd * 9.0));
    return spot * on * band * fade * EMBER_BRIGHT;
}

// 火焰 UV 位移: 以火焰中心为原点, 多频正弦叠加造有机摇曳。
// 采样坐标偏移 → 静态底图火焰纹理自身舞动 (参考 sea_swell 范式)。
fn fire_sway(uv: vec2<f32>, t: f32) -> vec2<f32> {
    let d = uv - FIRE_CENTER;
    let r = length(d);
    let radial_mask = 1.0 - smoothstep(FIRE_MASK_RADIUS * 0.3, FIRE_MASK_RADIUS, r);
    // 纯横向摇曳: 火焰左右摆动。
    let fx = sin(t * 0.785 * 2.0 + d.y * 10.0) * 0.4
           + sin(t * 0.785 * 3.0 - d.x * 8.0 + 1.7) * 0.3
           + cos(t * 0.785 * 5.0 + d.y * 12.0 + 4.1) * 0.2;
    let fy = 0.0;
    return vec2<f32>(fx, fy) * radial_mask * 0.006;
}

// ---- 海动效 (海场景) ----
// 波带涌动 (UV 纵向位移: 采样坐标本身起伏, 波带剪影随波行进 — 用户终审
// 裁定: 亮度调制读作"光沿静态波形移动的车", 路没动; 要路自己动) +
// 波光碎点 (乘性提亮软圆点, 原地明灭不漂移)。
// 位移随 sea_intensity 缩放: 暂停沉降逐像素回静态, 暗启动纪律不破。
// 参数集中于本段, 调参只动这里。所有频率取 1/8 Hz 整数倍,
// 与雨/火共用 8s 公共周期 (Rust 侧 MOTION_WRAP_SECS)。
const SEA_W: f32 = 0.7853982;          // 2π/8: 动效基频角速度 (1/8 Hz)

// 涌动: 2 层空间频率错开的同向行进正弦叠加成纵向位移场;
// 天空区 mask 为 0 不动, 越靠下的水层位移越大 (近水透视感)。
const SEA_MASK_TOP: f32 = 0.55;        // 位移区纵向软入起点 (uv.y, 波带上缘略上方)
const SEA_MASK_FULL: f32 = 0.72;       // 软入终点 (以下全量)
const SEA_SWELL_GAIN: f32 = 0.015;     // 位移幅度上限 (纵向 uv; 960x640 窗 ≈ ±9.6px)

// 碎点: 分列 hash, 位置基本不动, 亮度低频明灭 (频率档位 {2,3,4}/8 Hz → 周期 4s/2.67s/2s, 同星夜)。
const GLINT_DENSITY: f32 = 120.0;      // 列密度 (960px 窗 ≈ 8px/列)
const GLINT_RADIUS: f32 = 0.005;       // 点半径 (纵向 uv; 960px 窗 ≈ 6px 直径)
const GLINT_ASPECT: f32 = 1.5;         // 场景画布宽高比 (1536×1024), 圆点修正
const GLINT_BAND_TOP: f32 = 0.48;      // 散布带上缘 (uv.y, 对齐 AI 底图浪花带)
const GLINT_BAND_SPAN: f32 = 0.26;     // 散布带纵向跨度 (至 uv.y ≈ 0.98)
const GLINT_GAIN: f32 = 0.22;          // 点亮度上限 (乘性提亮; 0.14 太隐, 0.30 目测突兀)
const GLINT_ON: f32 = 0.88;            // hash > 此值的列才有碎点 (~14 颗)

// 水汽: 浪花破碎带的飘散水雾 (additive, 低 alpha)。
// 对齐 AI 底图浪花带 (Y≈0.42-0.58), 用 mist_pattern 生成飘动雾气。
const SEA_MIST_Y: f32 = 0.55;          // 雾带中心 (uv.y, 靠近浪花线)
const SEA_MIST_HALF: f32 = 0.08;       // 雾带半宽
const SEA_MIST_ALPHA: f32 = 0.25;      // 峰值 alpha (加浓可见)
const SEA_MIST_COLOR: vec3<f32> = vec3<f32>(0.75, 0.80, 0.82); // 淡青白, 匹配浪花水雾

// 波带涌动位移场: 返回纵向采样偏移 (uv 单位, 值域约 ±SWELL_GAIN)。
// 同一偏移施加于 from/to 两张场景图, 交叉淡化两端一致无跳变。
// 2026-08-06: 改为从远(上)往近(下)行进 — 相位主轴 uv.y, 时间反向。
fn sea_swell(uv: vec2<f32>, t: f32) -> f32 {
    let mask = smoothstep(SEA_MASK_TOP, SEA_MASK_FULL, uv.y);
    let depth = smoothstep(SEA_MASK_TOP, 1.0, uv.y); // 0 天空 → 1 底部, 近水动得多
    // 相位主轴 uv.y (纵向), 小 x 项破横向对齐; +t 使波从远(上)往近(下)行进。
    let w1 = sin(6.2831853 * (2.5 * uv.y + 0.3 * uv.x) + t * SEA_W * 2.0);
    let w2 = sin(6.2831853 * (4.0 * uv.y - 0.5 * uv.x) + t * SEA_W * 3.0 + 2.3);
    return (0.6 * w1 + 0.4 * w2) * mask * (0.4 + 0.6 * depth) * SEA_SWELL_GAIN;
}

// 波光碎点层: 波带内原地明灭的软圆点 (乘性提亮)。
fn sea_glints(uv: vec2<f32>, t: f32) -> f32 {
    let col = floor(uv.x * GLINT_DENSITY);
    let rnd = rain_hash(col * 1.37 + 97.0);  // 与雨/余烬不同种子, 避免位置相关
    let on = step(GLINT_ON, rain_hash(col * 3.1 + 131.0));
    // 列内 x 抖动避免网格感; y 落在散布带内 (常量, 不漂移)。
    let cx = (col + 0.3 + 0.4 * rnd) / GLINT_DENSITY;
    let cy = GLINT_BAND_TOP + GLINT_BAND_SPAN * rain_hash(col * 3.1 + 113.0);
    // 明灭频率取档位 {2,3,4}/8 Hz (整数倍, 保 8s 公共周期); smoothstep 缓起缓落。
    let k = 2.0 + floor(rnd * 3.0);
    let s = 0.5 + 0.5 * sin(t * SEA_W * k + rnd * 6.2831853);
    let twinkle = s * s * (3.0 - 2.0 * s);
    // 软圆点 (宽高比修正, 同余烬范式); 宽羽化边缘 (0.15R 起软) 避免硬点突兀感。
    let d = distance(
        vec2<f32>(uv.x * GLINT_ASPECT, uv.y),
        vec2<f32>(cx * GLINT_ASPECT, cy),
    );
    let spot = 1.0 - smoothstep(GLINT_RADIUS * 0.15, GLINT_RADIUS, d);
    return spot * on * twinkle * GLINT_GAIN;
}

// 水汽层: 浪花拍打处向上升起的水雾。
// 用 haze_noise (双线性 value noise) 生成无结构感的絮状雾,
// 采样 y 随时间上移模拟升腾。
fn sea_mist(uv: vec2<f32>, t: f32) -> vec3<f32> {
    let aspect = u.screen_w / max(u.screen_h, 1.0);
    // 上升: 采样 y 随时间递减 → 雾团从浪花线向上漂浮。
    let rise_speed = 0.015;
    let sample_y = uv.y + t * rise_speed;
    // 双层 noise: 低频定大势, 高频添碎屑。
    let p1 = vec2<f32>(uv.x * aspect * 4.0, sample_y * 3.0);
    let p2 = vec2<f32>(uv.x * aspect * 8.0, sample_y * 6.0);
    let n = haze_noise(p1) * 0.65 + haze_noise(p2) * 0.35;
    // 浓度: 靠近浪花线最浓, 向上渐淡。
    let dist = uv.y - SEA_MIST_Y;
    let rise_fade = smoothstep(0.18, 0.0, dist);
    let base_fade = 1.0 - smoothstep(0.0, SEA_MIST_HALF, abs(uv.y - SEA_MIST_Y));
    let x_fade = smoothstep(0.0, 0.12, uv.x) * smoothstep(1.0, 0.88, uv.x);
    let density = n * rise_fade * base_fade * x_fade;
    return SEA_MIST_COLOR * density * SEA_MIST_ALPHA;
}

// ---- 共享: 风驱雾纹 (sum-of-sines 伪噪声, 2D 各向同性, 不动采样坐标) ----
// 4 个 sin 全部用 comparable x 与 y 系数 (y/x ratio 0.7-1.0), 接近 45° 方向;
// 不同 ± sign + 不同 phase 打破对齐, 造 2D 噪声, 无 dominant direction (无 Tyndall)。
// 系数 6/8/12/16 → 空间周期 0.52-0.20 uv (503-192 px), 真正 fog 团尺寸
// (旧 2/2.5/3.5/4.5 → 1500-672 px, 太大读作"灰蒙蒙一片")。
// 调用方: speed 恒定, t = u.rain_time (非 wrap, 永不重置)。
// 速度必须恒定 — 若 speed 含 sin/cos(t) 调制, 则 t·speed 的导数为
// speed + t·speed', t 增大后摆幅线性增长, 雾气加速失控。
fn mist_pattern(uv: vec2<f32>, t: f32, speed: f32, scale: f32, phase: f32) -> f32 {
    let x = uv.x * scale + t * speed + phase;
    let y = uv.y * scale;
    let v = sin(x * 6.0 + y * 5.0 + phase) * 0.30
          + sin(x * 8.0 - y * 7.0 + phase * 1.7) * 0.25
          + sin(x * 12.0 + y * 10.0 + phase * 2.3) * 0.25
          + sin(x * 16.0 - y * 13.0 + phase * 3.1) * 0.20;
    return v * 0.5 + 0.5; // 0..1
}

// ---- 山动效 (山场景) ----
// 2026-08-04: 适配 AI 底图 (元宝生成山脊+云海)。
// 图中云海在 Y=0.25-0.55 (山谷间), 山脊间薄雾在 Y=0.45-0.65。
// 双层动效: 主云海 (缓慢流动) + 薄雾 (轻柔飘动), 颜色匹配图中粉紫色调。
// alpha 克制, 增强而非遮盖现有云层。

// 主云海: Y=0.25-0.55, 匹配图中云海位置
const MOUNTAIN_CLOUD_Y_TOP: f32 = 0.25;
const MOUNTAIN_CLOUD_Y_FULL: f32 = 0.40;
const MOUNTAIN_CLOUD_Y_END: f32 = 0.55;
const MOUNTAIN_CLOUD_ALPHA: f32 = 0.18;
const MOUNTAIN_CLOUD_COLOR: vec3<f32> = vec3<f32>(0.850, 0.720, 0.750);  // 粉紫色, 匹配落日照射的云

// 薄雾: Y=0.45-0.65, 山脊间的流动雾气
const MOUNTAIN_MIST_Y_TOP: f32 = 0.45;
const MOUNTAIN_MIST_Y_FULL: f32 = 0.55;
const MOUNTAIN_MIST_Y_END: f32 = 0.65;
const MOUNTAIN_MIST_ALPHA: f32 = 0.12;
const MOUNTAIN_MIST_COLOR: vec3<f32> = vec3<f32>(0.780, 0.680, 0.720);  // 淡粉紫, 更透明

fn mountain_ridge_mist(uv: vec2<f32>, t: f32) -> vec3<f32> {
    // 主云海: 缓慢水平流动, 增强图中云层
    let cloud_band = smoothstep(MOUNTAIN_CLOUD_Y_TOP, MOUNTAIN_CLOUD_Y_FULL, uv.y)
                   * (1.0 - smoothstep(MOUNTAIN_CLOUD_Y_FULL, MOUNTAIN_CLOUD_Y_END, uv.y));
    let cloud_p = mist_pattern(uv, t, 0.04, 2.5, 0.0);  // 更慢速度, 更大尺度
    let cloud = MOUNTAIN_CLOUD_COLOR * cloud_p * cloud_band * MOUNTAIN_CLOUD_ALPHA;

    // 薄雾: 轻柔飘动, 山脊间流动感
    let mist_band = smoothstep(MOUNTAIN_MIST_Y_TOP, MOUNTAIN_MIST_Y_FULL, uv.y)
                  * (1.0 - smoothstep(MOUNTAIN_MIST_Y_FULL, MOUNTAIN_MIST_Y_END, uv.y));
    let mist_p = mist_pattern(uv, t, 0.0625, 3.0, 1.7);  // 标准速度, 加相位偏移
    let mist = MOUNTAIN_MIST_COLOR * mist_p * mist_band * MOUNTAIN_MIST_ALPHA;

    return cloud + mist;
}

// ---- 森林动效 (森林场景) ----
// 雾不烘焙 (参考雨场景改造范式): export-scenes.py 森林配置已去 mist
// 字段, 运行时 forest_mist 全程序化生成。
// 暂停 500ms 沉降: forest_intensity = 0, 雾消失, 回到裸静态图。
//
// 速度恒定不调制: 调制 × unwrapped rain_time 产生 t·d(speed)/dt 项,
// t 增大后速度摆幅线性增长 (t=100s 时 ±0.75, 远超基准 0.0625),
// 视觉读作雾气越来越快 + 方向来回狂暴。副层已去 (反向对冲造成方向感混乱)。
const FOREST_MIST_Y: f32 = 0.691;
const FOREST_MIST_HALF: f32 = 0.159;
const FOREST_MIST_ALPHA: f32 = 0.25;
const FOREST_MIST_SPEED: f32 = 0.0625;
const FOREST_MIST_SCALE: f32 = 2.0;
const FOREST_MIST_COLOR: vec3<f32> = vec3<f32>(0.512, 0.604, 0.548);

fn forest_mist(uv: vec2<f32>, t: f32) -> vec3<f32> {
    let band = 1.0 - smoothstep(FOREST_MIST_HALF * 0.5, FOREST_MIST_HALF, abs(uv.y - FOREST_MIST_Y));
    let pattern = mist_pattern(uv, t, FOREST_MIST_SPEED, FOREST_MIST_SCALE, 1.7);
    return FOREST_MIST_COLOR * pattern * band * FOREST_MIST_ALPHA;
}

// ---- 洞穴水滴动效 (洞穴场景) ----
const CAVE_DROP1_X: f32 = 0.32;
const CAVE_DROP1_WATER_Y: f32 = 0.76;
const CAVE_DROP1_START_Y: f32 = 0.15;
const CAVE_DROP2_X: f32 = 0.44;
const CAVE_DROP2_WATER_Y: f32 = 0.68;
const CAVE_DROP2_START_Y: f32 = 0.10;
const CAVE_DROP_PERIOD: f32 = 5.0;
const CAVE_DROP_RADIUS: f32 = 0.003;
const CAVE_RIPPLE_DUR: f32 = 1.5;
const CAVE_RIPPLE_MAX_R: f32 = 0.08;
const CAVE_RIPPLE_WIDTH: f32 = 0.004;
const CAVE_RIPPLE_DISP: f32 = 0.02;

fn cave_single_drop(uv: vec2<f32>, t: f32, drop_x: f32, water_y: f32, start_y: f32, phase: f32,
drop_vis: f32, ripple_scale: f32, mask_below: f32, mask_above: f32) -> vec2<f32> {
    let ct = fract((t + phase) / CAVE_DROP_PERIOD);
    var brightness = 0.0;
    var displacement = 0.0;
    if (ct < 0.5) {
        let progress = ct / 0.5;
        let ease = progress * progress;
        let drop_y = mix(start_y, water_y, ease);
        let dx = (uv.x - drop_x) / (CAVE_DROP_RADIUS * drop_vis);
        let dy = (uv.y - drop_y) / (CAVE_DROP_RADIUS * drop_vis * 2.5);
        let dd = dx * dx + dy * dy;
        brightness += (1.0 - smoothstep(0.0, 1.0, dd)) * drop_vis * 0.5;
    }
    if (ct >= 0.5) {
        let ripple_t = ct - 0.5;
        let progress = ripple_t / 0.5;
        let radius = progress * CAVE_RIPPLE_MAX_R * ripple_scale;
        let d = distance(vec2<f32>(uv.x, uv.y), vec2<f32>(drop_x, water_y));
        let ring1 = smoothstep(radius - CAVE_RIPPLE_WIDTH, radius, d) * smoothstep(radius + CAVE_RIPPLE_WIDTH, radius, d);
        let r2 = radius * 0.6;
        let ring2 = smoothstep(r2 - CAVE_RIPPLE_WIDTH * 0.6, r2, d) * smoothstep(r2 + CAVE_RIPPLE_WIDTH * 0.6, r2, d) * 0.3;
        let ring = ring1 + ring2;
        let water_mask = smoothstep(water_y - mask_below, water_y + mask_above, uv.y);
        let fade = (1.0 - progress);
        brightness += ring * fade * water_mask * ripple_scale * 0.25;
        displacement += ring * fade * CAVE_RIPPLE_DISP * ripple_scale * water_mask * -1.0;
    }
    return vec2<f32>(brightness, displacement);
}

fn cave_droplets(uv: vec2<f32>, t: f32) -> vec2<f32> {
    let d1 = cave_single_drop(uv, t, CAVE_DROP1_X, CAVE_DROP1_WATER_Y, CAVE_DROP1_START_Y, 0.0, 1.2, 1.2, 0.08, 0.02);
    let d2 = cave_single_drop(uv, t, CAVE_DROP2_X, CAVE_DROP2_WATER_Y, CAVE_DROP2_START_Y, 1.5, 0.8, 0.30, 0.04, 0.01);
    return d1 + d2;
}

// ---- 夜市动效 (夜市场景) ----
// 灯笼光晕闪烁 (additive 暖色光斑): 随机散布的光点明灭, 营造夜市氛围。
// 2026-08-10: 适配 AI 底图 (挂满红黄灯笼的夜市街道)。
// 蒸汽已放弃: 静态底图已有烘焙蒸汽, additive 叠加在亮区被吃掉。
// UV 位移已放弃: 构图复杂, 位移会扭曲建筑。
// 频率取 1/8 Hz 整数倍, 保 8s 公共周期 (Rust 侧 MOTION_WRAP_SECS)。

// 灯笼光晕: 暖色光斑随机散布在灯笼区域闪烁, 非均匀带状。
// 二维 hash 网格布点 (非单列), 避免带状感; 半径/亮度逐点随机。
const NM_GLOW_COLS: f32 = 12.0;         // 横向网格列数
const NM_GLOW_ROWS: f32 = 8.0;          // 纵向网格行数
const NM_GLOW_RADIUS: f32 = 0.028;      // 基准光晕半径 (uv)
const NM_GLOW_ASPECT: f32 = 1.5;        // 宽高比修正
const NM_GLOW_BAND_TOP: f32 = 0.05;     // 光晕散布带上缘
const NM_GLOW_BAND_BOT: f32 = 0.58;     // 光晕散布带下缘 (避开人群)
const NM_GLOW_ALPHA: f32 = 0.28;        // 峰值 alpha (additive)
const NM_GLOW_COLOR: vec3<f32> = vec3<f32>(1.0, 0.75, 0.35);  // 暖橙色
const NM_GLOW_ON: f32 = 0.55;           // hash > 此值的 cell 才有光晕 (~40%)

// 灯笼光晕层: 二维网格布点, 随机散布 + 随机大小/亮度闪烁。
fn nightmarket_glow(uv: vec2<f32>, t: f32) -> f32 {
    let cell = vec2<f32>(floor(uv.x * NM_GLOW_COLS), floor(uv.y * NM_GLOW_ROWS));
    // 逐 cell 独立 hash: 位置偏移、大小、亮度、频率全部随机。
    let h1 = rain_hash(cell.x * 17.3 + cell.y * 31.7 + 701.0);  // 位置偏移 x
    let h2 = rain_hash(cell.x * 23.1 + cell.y * 41.3 + 713.0);  // 位置偏移 y
    let h3 = rain_hash(cell.x * 29.7 + cell.y * 47.9 + 737.0);  // 大小
    let h4 = rain_hash(cell.x * 37.1 + cell.y * 53.3 + 751.0);  // 亮度基线
    let h5 = rain_hash(cell.x * 43.9 + cell.y * 59.7 + 769.0);  // 频率档位
    // 是否点亮: ~45% 的 cell 有光晕。
    let on = step(NM_GLOW_ON, rain_hash(cell.x * 13.7 + cell.y * 19.3 + 787.0));
    // cell 内随机偏移 (0.15~0.85 避免贴边)。
    let cx = (cell.x + 0.15 + 0.7 * h1) / NM_GLOW_COLS;
    let band_h = NM_GLOW_BAND_BOT - NM_GLOW_BAND_TOP;
    let cy = NM_GLOW_BAND_TOP + band_h * (0.15 + 0.7 * h2);
    // 光晕半径: 基准 × (0.6~1.4) 逐点随机, 大小不一更自然。
    let radius = NM_GLOW_RADIUS * (0.6 + 0.8 * h3);
    // 亮度: 基线 × (0.5~1.0) 逐点随机。
    let brightness = 0.5 + 0.5 * h4;
    // 明灭频率取档位 {2,3,4}/8 Hz; smoothstep 缓起缓落。
    let k = 2.0 + floor(h5 * 3.0);
    let phase = rain_hash(cell.x * 61.1 + cell.y * 67.3 + 809.0);
    let s = 0.5 + 0.5 * sin(t * FIRE_W * k + phase * 6.2831853);
    let twinkle = s * s * (3.0 - 2.0 * s);
    // 软圆点 (宽高比修正); 宽羽化边缘。
    let d = distance(
        vec2<f32>(uv.x * NM_GLOW_ASPECT, uv.y),
        vec2<f32>(cx * NM_GLOW_ASPECT, cy),
    );
    let spot = 1.0 - smoothstep(radius * 0.15, radius, d);
    return spot * on * brightness * twinkle * NM_GLOW_ALPHA;
}

// ---- 铁匠铺动效 (铁匠铺场景) ----
// 三层 additive 叠加: 炉火呼吸 + 火花粒子 + 金属反光。
// 2026-08-10: 启用动效, 底图烘焙火花保留作为背景层次, 运行时粒子叠加增强动态感。
// 频率取 1/8 Hz 整数倍, 保 8s 公共周期 (Rust 侧 MOTION_WRAP_SECS)。
const BS_W: f32 = 0.7853982;  // 2π/8: 动效基频角速度 (1/8 Hz)

// 炉火呼吸: 双炉径向光晕脉动, 模拟熔炉火光闪烁。
// 左侧主炉 (较大) + 右侧副炉 (较小)。
const BS_FURNACE_L_CENTER: vec2<f32> = vec2<f32>(0.27, 0.45);  // 左炉位置
const BS_FURNACE_L_RADIUS: f32 = 0.07;    // 左炉光晕半径 (uv, 缩小)
const BS_FURNACE_R_CENTER: vec2<f32> = vec2<f32>(0.82, 0.43);  // 右炉位置
const BS_FURNACE_R_RADIUS: f32 = 0.06;    // 右炉光晕半径 (uv, 更小)
const BS_FURNACE_COLOR: vec3<f32> = vec3<f32>(1.0, 0.45, 0.1);  // 橙红色, 匹配图中炉火
const BS_FURNACE_L_ALPHA: f32 = 0.18;      // 左炉峰值 alpha
const BS_FURNACE_R_ALPHA: f32 = 0.12;      // 右炉峰值 alpha (较暗)

// 火花粒子: 从红热金属向左上方飞溅, 带重力弧线。
// 固定 36 颗火花, 每个像素遍历所有火花检查距离。
// 每颗火花有独立的发射时间、角度、速度、重力, 造自然散射。
// 匹配静态图: 细长线条, 扇形散开, 重力弧线。

// 金属反光: 铁砧表面高光闪烁, 低频脉动。
const BS_GLINT_CENTER: vec2<f32> = vec2<f32>(0.56, 0.59);  // 铁砧高光位置
const BS_GLINT_RADIUS: f32 = 0.030;      // 反光区域半径
const BS_GLINT_COLOR: vec3<f32> = vec3<f32>(0.9, 0.85, 0.75);  // 暖白色金属反光
const BS_GLINT_ALPHA: f32 = 0.45;        // 峰值 alpha
const BS_GLINT_FREQ_K: f32 = 2.0;        // 频率档位 (1/8 Hz * K)

// 炉火呼吸层: 双炉径向光晕脉动。
fn blacksmith_furnace(uv: vec2<f32>, t: f32) -> f32 {
    // 左炉: 较大光晕, 双频呼吸。
    let d_l = length(uv - BS_FURNACE_L_CENTER);
    let radial_l = 1.0 - smoothstep(BS_FURNACE_L_RADIUS * 0.2, BS_FURNACE_L_RADIUS, d_l);
    let breath_l = 0.5 + 0.3 * sin(t * BS_W * 3.0) + 0.2 * sin(t * BS_W * 5.0 + 1.7);
    let glow_l = radial_l * breath_l * BS_FURNACE_L_ALPHA;

    // 右炉: 较小光晕, 相位偏移, 频率略不同。
    let d_r = length(uv - BS_FURNACE_R_CENTER);
    let radial_r = 1.0 - smoothstep(BS_FURNACE_R_RADIUS * 0.2, BS_FURNACE_R_RADIUS, d_r);
    let breath_r = 0.5 + 0.3 * sin(t * BS_W * 4.0 + 2.3) + 0.2 * sin(t * BS_W * 6.0 + 4.1);
    let glow_r = radial_r * breath_r * BS_FURNACE_R_ALPHA;

    return glow_l + glow_r;
}

// 金属反光层: 铁砧表面高光闪烁。
fn blacksmith_glint(uv: vec2<f32>, t: f32) -> f32 {
    let d = length(uv - BS_GLINT_CENTER);
    let radial = 1.0 - smoothstep(BS_GLINT_RADIUS * 0.3, BS_GLINT_RADIUS, d);
    // 低频脉动, 模拟锤击间隔节奏感。
    let pulse = 0.5 + 0.5 * sin(t * BS_W * BS_GLINT_FREQ_K);
    return radial * pulse * BS_GLINT_ALPHA;
}

// ---- 火车动效 (火车场景) ----
// 车窗雨滴 (由上往下滑落) + 车厢内光呼吸 (暖色径向渐变)。
const TR_DROP_DENSITY: f32 = 60.0;
const TR_DROP_RADIUS: f32 = 0.006;
const TR_DROP_ON: f32 = 0.65;
const TR_DROP_GAIN: f32 = 0.3;
const TR_DROP_BAND_L: f32 = 0.48;
const TR_DROP_BAND_R: f32 = 0.94;
const TR_DROP_BAND_TOP: f32 = 0.05;
const TR_DROP_BAND_BOT: f32 = 0.88;
const TR_DROP_SPEED: f32 = 0.07;
const TR_TRAIL_LEN: f32 = 0.03;
const TR_TRAIL_ALPHA: f32 = 0.2;

const TR_GLOW_CENTER: vec2<f32> = vec2<f32>(0.15, 0.12);
const TR_GLOW_RADIUS: f32 = 0.35;
const TR_GLOW_COLOR: vec3<f32> = vec3<f32>(1.0, 0.85, 0.5);
const TR_GLOW_ALPHA: f32 = 0.12;
const TR_GLOW_FREQ: f32 = 0.125;

fn train_window_drops(uv: vec2<f32>, t: f32) -> f32 {
    let col = floor(uv.x * TR_DROP_DENSITY);
    let rnd = rain_hash(col * 1.37 + 501.0);
    let on = step(TR_DROP_ON, rain_hash(col * 3.1 + 537.0));
    let cx = (col + 0.3 + 0.4 * rnd) / TR_DROP_DENSITY;

    let base_y = rain_hash(col * 3.1 + 513.0);
    let speed_factor = 0.7 + 0.6 * rain_hash(col * 7.3 + 601.0);
    let drop_speed = TR_DROP_SPEED * speed_factor;

    let travel = TR_DROP_BAND_BOT - TR_DROP_BAND_TOP;
    let raw_y = base_y + t * drop_speed;
    let cycle_y = fract(raw_y);
    let cy = TR_DROP_BAND_TOP + cycle_y * travel;

    let in_window = step(TR_DROP_BAND_L, cx) * step(cx, TR_DROP_BAND_R);
    let fade_out = 1.0 - smoothstep(0.6, 1.0, cycle_y);

    let k = 1.0 + floor(rnd * 2.0);
    let s = 0.7 + 0.3 * sin(t * FIRE_W * k + rnd * 6.2831853);
    let glint = s * s;

    let d = distance(vec2<f32>(uv.x, uv.y), vec2<f32>(cx, cy));
    let spot = 1.0 - smoothstep(TR_DROP_RADIUS * 0.2, TR_DROP_RADIUS, d);

    let trail_top = cy - TR_TRAIL_LEN;
    let in_trail_y = smoothstep(trail_top - 0.001, trail_top, uv.y) * smoothstep(cy + 0.001, cy, uv.y);
    let trail_x_dist = abs(uv.x - cx);
    let trail_lat = 1.0 - smoothstep(0.0, TR_DROP_RADIUS * 1.5, trail_x_dist);
    let trail = in_trail_y * trail_lat * trail_lat * TR_TRAIL_ALPHA;

    let total = max(spot, trail) * on * in_window * glint * fade_out * TR_DROP_GAIN;
    return total;
}

fn train_interior_glow(uv: vec2<f32>, t: f32) -> f32 {
    let d = length(uv - TR_GLOW_CENTER);
    let radial = 1.0 - smoothstep(TR_GLOW_RADIUS * 0.2, TR_GLOW_RADIUS, d);
    let interior = 1.0 - smoothstep(0.40, 0.48, uv.x);
    let breath = 0.5 + 0.5 * sin(t * 6.2831853 * TR_GLOW_FREQ);
    return radial * interior * breath * TR_GLOW_ALPHA;
}

// ---- 星夜动效 (星夜场景) ----
// 雨场景范式: 静态图去星, 星野全部运行时渲染。
// 2026-08-03 银河升级 (Task 5, spec: docs/specs/pomodoro-scene-starry-milkyway.md):
// 星点布点从 48×28 hash 网格 (~100 颗均匀随机) 迁移到真实星表 (Yale BSC5,
// 6743 颗, CPU 启动烘焙成 starfield_tex)。hash 网格常量 (SF_COLS/ROWS/ON/BIG/
// WARM/ASPECT) 与 star_cell/star_color 随之退役; 山脊遮挡沿用 SF_BAND_BOT。
// - 基础星野 (star_field): 采样星野纹理, 常驻 (starry_base = 场景权重, 暂停定格可见)。
// - 星闪 (star_twinkle): 脉冲场调制纹理采样, 随 starry_intensity 沉降, {2,3,4}/8 Hz 档位。
// - 流星 (meteor): 随 starry_intensity, rain_time 连续触发 (非 wrap 无跳变), 淡入淡出, 压暗。
const STAR_W: f32 = 0.7853982;  // 2π/8: 动效基频角速度 (1/8 Hz)
const SF_BAND_BOT: f32 = 0.80;  // 星带下缘 (山脊上方; 底图山脊 base_y 0.88/0.97, 留缓冲)
const SF_TWINKLE_AMP: f32 = 0.42; // 星闪明暗双向摆动幅度 (±; 2026-08-02 用户裁定, 勿回调)
const TW_COLS: f32 = 96.0;      // 星闪脉冲场网格列 (cell ≈16px @1536 画布)
const TW_ROWS: f32 = 54.0;      // 星闪脉冲场网格行 (cell ≈19px @1024 高)

// 山脊遮挡 mask: 星带下缘以下渐隐 (作用于星野与星闪)。
fn star_band(y: f32) -> f32 {
    return 1.0 - smoothstep(SF_BAND_BOT, SF_BAND_BOT + 0.04, y);
}

// 基础星野 (静态, 常驻): 采样 CPU 烘焙的真实星表纹理 — 位置/亮度/暖色全部
// 来自星表 (Yale BSC5), 暂停时定格可见 (定格语义)。
fn star_field(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(starfield_tex, extras_smp, uv).rgb * star_band(uv.y);
}

// 星闪: 细网格脉冲场**调制**星野纹理采样 (不再自绘光点)。
// cell ≈16×19px ≥ 亮星光点 (≤8px), 绝大多数格 ≤1 颗亮星 → 读作逐星明灭;
// 亮星贴格边时两半可能不同步, 点径 ≤3px, 可接受。
// 脉冲逻辑不变 (2026-08-02 裁定): {2,3,4}/8 Hz 档位 → 周期 4s/2.67s/2s;
// 双极 sin [-1,1] ± SF_TWINKLE_AMP 明暗双向 (单向加亮读作「静态」)。
fn star_twinkle(uv: vec2<f32>, t: f32) -> vec3<f32> {
    let cell = vec2<f32>(floor(uv.x * TW_COLS), floor(uv.y * TW_ROWS));
    let freq_h = rain_hash(cell.x * 19.0 + cell.y * 23.0 + 8.0);  // 独立 hash → 频率真随机
    let phase_h = rain_hash(cell.x * 31.0 + cell.y * 47.0 + 9.0); // 独立 hash → 相位真随机
    let k = 2.0 + floor(freq_h * 3.0);  // {2,3,4}/8 Hz
    let pulse = sin(t * STAR_W * k + phase_h * 6.2831853);   // [-1,1] 双极: 明暗双向
    let star = textureSample(starfield_tex, extras_smp, uv).rgb;
    return star * star_band(uv.y) * pulse * SF_TWINKLE_AMP;
}

// ---- 暗星雾 (star_haze): 银河「深邃」体量的来源 ----
// 星表 (≤6.5 等) 给真实结构, 但肉眼银河的密度感主要来自无数暗星 ——
// 这里用 value noise 生成连续雾密度 (非离散网格点阵, 避免像素画感),
// 密度按银纬解析 mask 聚集 (b≈0 最密), 与星表亮星带、底图光带共用
// 同一坐标系 (三层对齐)。
// mask 常量与 tools/export-stars.py 的固定观测姿态互逆 (Task 8 对齐底图时
// 两侧同步回填)。
const HAZE_THETA: f32 = 1.0471976;          // 60° (弧度) = export-stars.py THETA_DEG
const HAZE_SHIFT: vec2<f32> = vec2<f32>(0.0, -0.03); // = export-stars.py SHIFT_X/Y
const HAZE_BAND: f32 = 0.10;     // 银纬半宽 (py 单位 ≈ 15° 银纬)

// 银纬 proxy: UV → 逆旋转平面坐标 py (py=0 ⟺ 银道面)。
// 与 export-stars.py 投影互逆: py=0 的位置只依赖 THETA+SHIFT (与 L_CENTER/FOV
// 无关); 但 HAZE_BAND 的度数含义随 FOV_V (改 FOV_V 须联动带宽, Task 8 同步回填)。
fn galactic_py(uv: vec2<f32>) -> f32 {
    let rx = uv.x - 0.5 - HAZE_SHIFT.x;
    let ry = 0.5 - uv.y + HAZE_SHIFT.y;
    return -rx * sin(HAZE_THETA) + ry * cos(HAZE_THETA);
}

// 双线性 value noise: 在4个整数格点采样 hash, 双线性插值,
// 输出 [0,1] 连续标量。无离散网格边界, 自然平滑。
fn haze_noise(p: vec2<f32>) -> f32 {
    let ix = floor(p.x);
    let iy = floor(p.y);
    let fx = fract(p.x);
    let fy = fract(p.y);
    // smoothstep 插值核: 三次 Hermite 消除格点处的导数不连续
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let a = rain_hash(ix * 127.1 + iy * 311.7);
    let b = rain_hash((ix + 1.0) * 127.1 + iy * 311.7);
    let c = rain_hash(ix * 127.1 + (iy + 1.0) * 311.7);
    let d = rain_hash((ix + 1.0) * 127.1 + (iy + 1.0) * 311.7);
    return mix(mix(a, b, ux), mix(c, d, ux), uy);
}

// 暗星雾 (静态, 常驻): 用 value noise 生成连续雾密度, 沿银道面聚集。
// 非离散点阵 — 避免网格结构造成的像素画/半调印刷感。
// 挂 starry_base 与星野同生灭。
fn star_haze(uv: vec2<f32>) -> vec3<f32> {
    let band = 1.0 - smoothstep(HAZE_BAND, HAZE_BAND + 0.08, abs(galactic_py(uv)));
    // 双层 noise 叠加: 低频定大势 (银河带宽), 高频添碎屑 (自然颗粒感)。
    // 采样坐标乘以画布宽高比修正, 保证各向同性。
    let aspect = u.screen_w / max(u.screen_h, 1.0);
    let p = vec2<f32>(uv.x * aspect, uv.y);
    let n1 = haze_noise(p * 8.0);      // 低频: ~192px 周期 (1536/8)
    let n2 = haze_noise(p * 24.0) * 0.4; // 高频: ~64px 周期, 振幅衰减
    let density = (n1 + n2) * band;    // 银纬调制
    // 微蓝白 (暗星普遍偏冷), 亮度极低, 不闪 — 闪是亮星 (纹理层) 的事。
    // 底图已含银河光带细节 (尘埃暗隙), haze 只在几乎不可见层面增加深空颗粒感,
    // 勿喧宾夺主冲掉暗部层次。
    return vec3<f32>(0.9, 0.93, 1.0) * density * 0.01 * star_band(uv.y);
}

// 流星: 周期性斜向流星 (rain_time 连续触发, ~24s 一颗, 存续 ~1.4s)。
// 头部从右上斜向左下, 尾迹朝右上 (头部后方) 指数衰减; 淡入淡出, 压暗避免「爆闪」。
const METEOR_PERIOD: f32 = 24.0;
const METEOR_HEAD: f32 = 0.5;   // 头部亮度 (原 0.9 像爆闪灯, 压暗)

fn meteor(uv: vec2<f32>, rt: f32) -> f32 {
    let idx = floor(rt / METEOR_PERIOD);
    let phase = rt - idx * METEOR_PERIOD;
    if (phase >= 1.4) { return 0.0; }
    let h = rain_hash(idx * 7.31 + 9.1);   // 该颗流星的水平位置 (确定性)
    let life = phase / 1.4;
    let head = vec2<f32>(0.80 - h * 0.50 - life * 0.28, 0.14 + h * 0.26 + life * 0.20);
    let d = uv - head;
    let dir = normalize(vec2<f32>(0.28, -0.20));   // 尾迹方向 (右上)
    let along = dot(d, dir);
    let perp = length(d - dir * along);
    let trail = exp(-perp * 40.0) * exp(-clamp(along, 0.0, 4.0) * 5.0);
    let core = exp(-dot(d, d) * 900.0);
    // 淡入 (避免突然闪光) + 淡出; 整体压暗。
    let appear = smoothstep(0.0, 0.25, life);
    return (trail * 0.4 + core * METEOR_HEAD) * appear * (1.0 - life) * 0.9;
}

@group(0) @binding(0)
var<uniform> u: Uniforms;

@group(1) @binding(0)
var tex_from: texture_2d<f32>;

@group(1) @binding(1)
var samp_from: sampler;

@group(2) @binding(0)
var tex_to: texture_2d<f32>;

@group(2) @binding(1)
var samp_to: sampler;

// group 3 附加纹理组 (三纹理 + 共享采样器): 星野 / 天空蒙版 / 发光蒙版。
// 合一原因: wgpu 默认 max_bind_groups=4 (2026-08-30 实测超组 panic),
// 星野纹理无运行时替换 API (启动后静态), 合并无副作用。
// 未配置的槽位 Rust 侧绑 1×1 全黑回退 — 三槽恒可绑。
@group(3) @binding(0)
var starfield_tex: texture_2d<f32>;

// 天空蒙版 (桌景天级路线 B): 灰度图, 天空区域白 —— sky_amount 驱动天空
// 向深夜蓝暗沉降。
@group(3) @binding(1)
var sky_mask_tex: texture_2d<f32>;

// 发光蒙版 (桌景签名时刻): 灰度图, 发光区灰度值 = 亮灯时序阈值
// (灰度高先亮) —— glow_amount 推进 + 抖动阈值, 灯一盏盏亮起而非全街跳闸。
@group(3) @binding(2)
var glow_mask_tex: texture_2d<f32>;

@group(3) @binding(3)
var extras_smp: sampler;

// 发光蒙版点亮色温 (暖琥珀, 窗内灯光); additive 发射, 不受时辰调色影响
// (光源恒暖, 调色作用于被照物)。
const GLOW_WARM: vec3<f32> = vec3<f32>(1.0, 0.72, 0.42);
// 天空夜空化目标色相 (深夜蓝暗, 乘性沉降)。
const NIGHT_SKY_TINT: vec3<f32> = vec3<f32>(0.30, 0.36, 0.55);

// ---- 萤火虫 (桌景分钟级微事件): 程序化萤绿光点, 免资产 ----
// 5 只, 下半区慢漂 (基位 hash 固定 + 低频正弦游移), 明灭节奏错开
// (大部分时间是暗的, 亮时忽然 —— 萤火虫的「眨眼」感); 包络由产品调度器
// 驱动 (事件开演淡入, 落幕淡出), 本层只播放。
const FF_COUNT: f32 = 5.0;
const FF_COLOR: vec3<f32> = vec3<f32>(0.72, 1.0, 0.45); // 萤绿偏暖
const FF_RADIUS: f32 = 0.004;      // 点半径 (纵向 uv; 小窗里 ~1-2px 亮点)
const FF_DRIFT_X: f32 = 0.045;     // 横向游移幅度
const FF_DRIFT_Y: f32 = 0.030;     // 纵向游移幅度

fn firefly_layer(uv: vec2<f32>, t: f32) -> f32 {
    var acc = 0.0;
    let aspect = u.screen_w / max(u.screen_h, 1.0);
    for (var i = 0.0; i < FF_COUNT; i += 1.0) {
        let h1 = rain_hash(i * 12.9 + 3.1);
        let h2 = rain_hash(i * 7.7 + 9.3);
        // 基位下半区 (y 0.50~0.92, 贴地面/小屋生活层, 不上天空)。
        let cx = 0.08 + h1 * 0.84 + sin(t * 0.35 * (0.7 + 0.5 * h2) + h1 * 6.2831853) * FF_DRIFT_X;
        let cy = 0.50 + h2 * 0.42 + cos(t * 0.50 * (0.6 + 0.4 * h1) + h2 * 6.2831853) * FF_DRIFT_Y;
        // 明灭: 频率错开, 暗多亮少 (smoothstep 抬门槛)。
        let tw = 0.5 + 0.5 * sin(t * (1.2 + h1 * 2.2) + h2 * 6.2831853);
        let blink = smoothstep(0.55, 0.95, tw);
        let d = distance(vec2<f32>(uv.x * aspect, uv.y), vec2<f32>(cx * aspect, cy));
        let spot = 1.0 - smoothstep(FF_RADIUS * 0.3, FF_RADIUS, d);
        acc += spot * blink;
    }
    return min(acc, 1.0);
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
) -> VsOut {
    // pos 为 0..1 的归一化窗口坐标,左上角 (0,0),右下角 (1,1)
    let clip = vec4<f32>(
        pos.x * 2.0 - 1.0,
        1.0 - pos.y * 2.0,
        0.0,
        1.0,
    );
    var out: VsOut;
    out.clip = clip;
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // 场景 UV 位移: 海波涌动 (纵向) 作用于采样坐标本身; 位移随强度缩放,
    // 强度 0 时采样原坐标, 输出与静态逐像素一致。
    var sample_uv = in.uv;
    if (u.fire_intensity > 0.0) {
        sample_uv += fire_sway(in.uv, u.time) * u.fire_intensity;
    }
    if (u.sea_intensity > 0.0) {
        sample_uv += vec2<f32>(0.0, sea_swell(in.uv, u.time) * u.sea_intensity);
    }
    // 森林不动采样坐标 — 树梢保持静止,雾作为独立程序化层 additive 叠加在采样之上
    // (避免与中林 (y=0.68) 重叠的中林雾带 (y=0.55-0.69) 让树跟着横移读作"海草")。
    let c_from = textureSample(tex_from, samp_from, sample_uv);
    let c_to = textureSample(tex_to, samp_to, sample_uv);
    var color = mix(c_from, c_to, u.fade);
    if (u.rain_intensity > 0.0) {
        // 线性空间 additive 亮度叠加 (sRGB 纹理采样已转线性)。
        // 雨丝走独立雨钟: 暂停时雨钟冻结, 雨丝定格可见 (2026-07-29 用户裁定,
        // 不再随包络沉降); 强度常驻场景权重, 冻结/推进节奏由 Rust 侧控制。
        color = vec4<f32>(
            color.rgb + vec3<f32>(rain_overlay(in.uv, u.rain_time) * u.rain_intensity),
            color.a,
        );
    }
    // 篝火: UV 位移已在 sample_uv 中应用 (fire_sway)。
    // 底图火焰纹理自身舞动; 余烬粒子作为微量点缀 additive 叠加。
    if (u.fire_intensity > 0.0) {
        color = vec4<f32>(
            color.rgb + EMBER_COLOR * ember_layer(in.uv, u.time) * u.fire_intensity,
            color.a,
        );
    }
    if (u.sea_intensity > 0.0) {
        // 亮场景乘性碎点提亮 (不改色相); 涌动已在上方采样坐标中体现。
        color = vec4<f32>(
            color.rgb * (1.0 + sea_glints(in.uv, u.time) * u.sea_intensity),
            color.a,
        );
        // 水汽: 浪花破碎带 additive 雾气, 增氛围。
        color = vec4<f32>(
            color.rgb + sea_mist(in.uv, u.rain_time) * u.sea_intensity,
            color.a,
        );
    }
    if (u.mountain_intensity > 0.0) {
        // 山脊云雾缭绕,随风而动 (用户 2026-07-30 终审反馈, additive 叠加, 不动采样)。
        // t 改用 u.rain_time (非 wrap) — 8s wrap_motion_time 重置会让 pattern 跳变
        // (用户 2026-07-30 反馈 "还是有重置的情况")。rain_time 是 Rust rain_clock,
        // 每帧 +=dt*motion_gain, 无 8s wrap, 持续累加, 雾漂移连续无跳变。
        // 雨和雾共用 rain_time, 都是非 wrap 持续动效, 语义一致。
        color = vec4<f32>(
            color.rgb + mountain_ridge_mist(in.uv, u.rain_time) * u.mountain_intensity,
            color.a,
        );
    }
    if (u.forest_intensity > 0.0) {
        // 全程序化云雾 (用户 2026-07-30 终审反馈 "去静态底雾, 运行时动态渲染",
        // 参考雨场景改造范式)。 t 同上, 用 u.rain_time 避免 8s wrap 跳变。
        color = vec4<f32>(
            color.rgb + forest_mist(in.uv, u.rain_time) * u.forest_intensity,
            color.a,
        );
    }
    if (u.cave_intensity > 0.0) {
        color = vec4<f32>(
            color.rgb + vec3<f32>(cave_droplets(in.uv, u.time).x) * vec3<f32>(0.8, 0.9, 1.0) * u.cave_intensity,
            color.a,
        );
    }
    if (u.blacksmith_intensity > 0.0) {
        // 铁匠铺: 炉火呼吸 + 金属反光。
        color = vec4<f32>(
            color.rgb + BS_FURNACE_COLOR * blacksmith_furnace(in.uv, u.time) * u.blacksmith_intensity,
            color.a,
        );
        color = vec4<f32>(
            color.rgb + BS_GLINT_COLOR * blacksmith_glint(in.uv, u.time) * u.blacksmith_intensity,
            color.a,
        );
    }
    if (u.train_intensity > 0.0) {
        color = vec4<f32>(
            color.rgb + vec3<f32>(train_window_drops(in.uv, u.time)) * u.train_intensity,
            color.a,
        );
        color = vec4<f32>(
            color.rgb + TR_GLOW_COLOR * train_interior_glow(in.uv, u.time) * u.train_intensity,
            color.a,
        );
    }
    if (u.nightmarket_intensity > 0.0) {
        // 夜市: 灯笼光晕闪烁 (additive 暖色光斑, 在暗区明显可见)。
        color = vec4<f32>(
            color.rgb + NM_GLOW_COLOR * nightmarket_glow(in.uv, u.time) * u.nightmarket_intensity,
            color.a,
        );
    }
    if (u.starry_base > 0.0 || u.starry_intensity > 0.0) {
        // 星夜 (雨场景范式): 基础星野常驻 (starry_base = 场景权重, 暂停定格可见);
        // 星闪 + 流星随 starry_intensity (包络×权重) 沉降, 暂停 500ms 回静态星野。
        // 三层合成: 星表亮星 (纹理) + 暗星雾 (银纬聚集) 挂 starry_base;
        // 星闪/流星挂 starry_intensity。
        color = vec4<f32>(
            color.rgb
                + (star_field(in.uv) + star_haze(in.uv)) * u.starry_base
                + (star_twinkle(in.uv, u.time) + vec3<f32>(meteor(in.uv, u.rain_time)))
                    * u.starry_intensity,
            color.a,
        );
    }
    // ---- 时辰调色 + 双蒙版合成 (桌景 scene-world; 恒等值时逐像素零漂移) ----
    // 顺序: 场景动效合成 → 分级调色 (乘性色调 + 饱和度绕亮度轴 + 亮度) →
    // 天空夜空化 (蒙版区向深夜蓝沉降) → 发光蒙版 additive 点亮 (发射恒暖,
    // 不被调色 —— 灯是光源不是被照物)。
    var rgb = color.rgb * vec3<f32>(u.tod_tint_r, u.tod_tint_g, u.tod_tint_b);
    let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    rgb = mix(vec3<f32>(luma), rgb, u.tod_saturation) * u.tod_brightness;
    if (u.sky_amount > 0.0) {
        let sky = textureSample(sky_mask_tex, extras_smp, in.uv).r;
        if (sky > 0.003) {
            rgb = mix(rgb, rgb * NIGHT_SKY_TINT, sky * u.sky_amount);
        }
    }
    if (u.glow_amount > 0.0) {
        let g = textureSample(glow_mask_tex, extras_smp, in.uv).r;
        if (g > 0.003) {
            // 灰度即亮灯时序: 阈值 = 1 - g (灰度高的灯先亮);
            // 逐像素 hash 抖动破机械边沿 (错落感 =「活」与「假」的分水岭)。
            let dither = rain_hash(in.uv.x * 719.0 + in.uv.y * 911.0) * 0.06;
            let lit = clamp((u.glow_amount - (1.0 - g) + dither) * 8.0, 0.0, 1.0);
            rgb += GLOW_WARM * lit * g * 0.85;
        }
    }
    // 萤火虫 (发射光点, 调色之后叠加 —— 与发光蒙版同层语义: 光源不被调色)。
    if (u.event_firefly > 0.0) {
        rgb += FF_COLOR * firefly_layer(in.uv, u.time) * u.event_firefly * 0.7;
    }
    // 闪电最后 (压过一切的全屏提亮, 蓝白色温; 双闪脉冲形状在产品侧)。
    if (u.flash_intensity > 0.0) {
        rgb += vec3<f32>(0.82, 0.88, 1.05) * u.flash_intensity * 0.85;
    }
    return vec4<f32>(rgb, color.a * u.opacity);
}
