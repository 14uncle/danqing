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
// 光晕呼吸 (乘性, 只起伏已有辉光) + 火星余烬上浮 (暖色 additive 圆点,
// 形态对齐静态图已有火星点)。参数集中于本段, 调参只动这里。
// 所有频率/速度取 1/8 Hz 整数倍, 与雨共用 8s 公共周期 (Rust 侧 MOTION_WRAP_SECS)。
const FIRE_W: f32 = 0.7853982;         // 2π/8: 动效基频角速度 (1/8 Hz)

// 呼吸: 3 个正弦叠加 (2/8、3/8、5/8 Hz → 周期 4/2.67/1.6s) 叠出有机起伏。
const FIRE_CENTER: vec2<f32> = vec2<f32>(0.5, 0.86); // 光晕锚点 (下部火床, 对齐静态图辉光)
const FIRE_MASK_RADIUS: f32 = 0.48;    // 呼吸径向衰减半径 (uv)
const FIRE_BREATH_GAIN: f32 = 0.08;    // 呼吸幅度上限 (乘性; 4% 实测不可读, 翻倍)

// 余烬: 分列 hash, 每列一颗, 相位随机、速度全列一致 (保公共周期)。
const EMBER_DENSITY: f32 = 160.0;      // 列密度 (960px 窗 ≈ 6px/列)
const EMBER_SPEED: f32 = 0.25;         // 上浮速度 (循环/秒, 2/8; 一趟 ~4s)
const EMBER_SPAN: f32 = 0.52;          // 行程: 自火床 (y≈0.86) 升至 y≈0.34 折返
const EMBER_RADIUS: f32 = 0.0055;      // 点半径 (纵向 uv; 960px 窗 ≈ 7px 直径, 终审裁定)
const EMBER_ASPECT: f32 = 1.5;         // 场景画布宽高比 (1536×1024), 圆点修正
const EMBER_SWAY: f32 = 0.006;         // 横摆幅度 (uv ≈ 6px)
const EMBER_BRIGHT: f32 = 0.85;        // 点亮度上限 (additive; 0.5 在亮橙辉光上对比不足)
const EMBER_ON: f32 = 0.80;            // hash > 此值的列才有余烬 (~32 列, 带内 ~22-25 颗)
const EMBER_COLOR: vec3<f32> = vec3<f32>(1.0, 0.78, 0.45); // 热黄 (对齐静态火星点的淡黄)

fn fire_flicker(t: f32) -> f32 {
    return 0.6 * sin(t * FIRE_W * 2.0)
        + 0.3 * sin(t * FIRE_W * 3.0 + 1.7)
        + 0.2 * sin(t * FIRE_W * 5.0 + 4.1);
}

// 光晕呼吸: 径向 mask × 低频起伏, 返回值域约 ±BREATH_GAIN。
fn fire_breath(uv: vec2<f32>, t: f32) -> f32 {
    let d = distance(uv, FIRE_CENTER);
    let mask = 1.0 - smoothstep(FIRE_MASK_RADIUS * 0.4, FIRE_MASK_RADIUS, d);
    return fire_flicker(t) * mask * FIRE_BREATH_GAIN;
}

// 余烬层: 自底部升起, 横向轻摆, 随行程 (life) 淡出。
fn ember_layer(uv: vec2<f32>, t: f32) -> f32 {
    let col = floor(uv.x * EMBER_DENSITY);
    let rnd = rain_hash(col * 1.37 + 53.0); // 与雨不同种子, 避免位置相关
    let on = step(EMBER_ON, rain_hash(col * 3.1 + 71.0));
    // 横摆频率取档位 {1,2,3}/8 Hz (整数倍, 保 8s 公共周期)。
    let k = 1.0 + floor(rnd * 3.0);
    let cx = (col + 0.5) / EMBER_DENSITY + sin(t * FIRE_W * k + rnd * 6.2831853) * EMBER_SWAY;
    let life = fract(t * EMBER_SPEED + rnd * 7.0); // 0=点燃(底部) → 1=熄灭(顶端)
    let cy = FIRE_CENTER.y - life * EMBER_SPAN;
    // 发射带收窄: 对齐静态图火星散布带 (中部偏右), 带外软裁。
    let band = smoothstep(0.20, 0.35, cx) * (1.0 - smoothstep(0.75, 0.90, cx));
    // 圆点 (宽高比修正); 亮度随行程衰减 + 低频闪烁 (4/8 Hz, 整数倍)。
    let d = distance(
        vec2<f32>(uv.x * EMBER_ASPECT, uv.y),
        vec2<f32>(cx * EMBER_ASPECT, cy),
    );
    let spot = 1.0 - smoothstep(EMBER_RADIUS * 0.5, EMBER_RADIUS, d);
    let fade = (1.0 - life) * (0.7 + 0.3 * sin(t * FIRE_W * 4.0 + rnd * 9.0));
    return spot * on * band * fade * EMBER_BRIGHT;
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
const GLINT_BAND_TOP: f32 = 0.72;      // 散布带上缘 (uv.y, 对齐静态图第一叠波带)
const GLINT_BAND_SPAN: f32 = 0.26;     // 散布带纵向跨度 (至 uv.y ≈ 0.98)
const GLINT_GAIN: f32 = 0.22;          // 点亮度上限 (乘性提亮; 0.14 太隐, 0.30 目测突兀)
const GLINT_ON: f32 = 0.88;            // hash > 此值的列才有碎点 (~14 颗)

// 波带涌动位移场: 返回纵向采样偏移 (uv 单位, 值域约 ±SWELL_GAIN)。
// 同一偏移施加于 from/to 两张场景图, 交叉淡化两端一致无跳变。
fn sea_swell(uv: vec2<f32>, t: f32) -> f32 {
    let mask = smoothstep(SEA_MASK_TOP, SEA_MASK_FULL, uv.y);
    let depth = smoothstep(SEA_MASK_TOP, 1.0, uv.y); // 0 天空 → 1 底部, 近水动得多
    // 相位含小 y 项: 相邻行位移不同步, 波峰不成直线 (水面感); 同向行进 (调参轮 2)。
    let w1 = sin(6.2831853 * (2.0 * uv.x + 0.5 * uv.y) - t * SEA_W * 2.0);
    let w2 = sin(6.2831853 * (3.5 * uv.x - 0.8 * uv.y) - t * SEA_W * 3.0 + 2.3);
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
// 单层暖粉雾融入暮色。mask 0.50-0.88 集中在山脊上空。
// alpha 0.30 (终审 0.45 → 0.22 消除黄沙, 0.22 太隐回提至明显可见区间 b977da6;
// 山脊背景本已暖粉 ~170/255, additive 叠加勿再过饱和读作"黄沙")。
// scale 3.0 (升自 2.0, 雾团 ~125-320px 更细腻不读作"沙粒")。
const MOUNTAIN_MIST_Y_TOP: f32 = 0.50;
const MOUNTAIN_MIST_Y_FULL: f32 = 0.80;
const MOUNTAIN_MIST_Y_END: f32 = 0.88;
const MOUNTAIN_MIST_ALPHA: f32 = 0.30;
const MOUNTAIN_MIST_COLOR: vec3<f32> = vec3<f32>(0.920, 0.650, 0.620);

fn mountain_ridge_mist(uv: vec2<f32>, t: f32) -> vec3<f32> {
    let band = smoothstep(MOUNTAIN_MIST_Y_TOP, MOUNTAIN_MIST_Y_FULL, uv.y)
             * (1.0 - smoothstep(MOUNTAIN_MIST_Y_END, 1.0, uv.y));
    let p = mist_pattern(uv, t, 0.0625, 3.0, 0.0);
    return MOUNTAIN_MIST_COLOR * p * band * MOUNTAIN_MIST_ALPHA;
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
    return textureSample(starfield_tex, starfield_smp, uv).rgb * star_band(uv.y);
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
    let star = textureSample(starfield_tex, starfield_smp, uv).rgb;
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

// 星野纹理 (星夜场景): CPU 启动烘焙的真实星表星点层, 与场景图同画布,
// 共用 in.uv 采样 (同一组 Cover 裁剪, 星点与山脊线像素级对齐)。
// 未配置时 Rust 侧绑 1×1 全黑回退 — 本槽恒可绑。
@group(3) @binding(0)
var starfield_tex: texture_2d<f32>;

@group(3) @binding(1)
var starfield_smp: sampler;

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
    if (u.fire_intensity > 0.0) {
        // 呼吸乘性起伏已有辉光 (不改色相) + 余烬暖色 additive (线性空间)。
        color = vec4<f32>(
            color.rgb * (1.0 + fire_breath(in.uv, u.time) * u.fire_intensity)
                + EMBER_COLOR * ember_layer(in.uv, u.time) * u.fire_intensity,
            color.a,
        );
    }
    if (u.sea_intensity > 0.0) {
        // 亮场景乘性碎点提亮 (不改色相); 涌动已在上方采样坐标中体现。
        color = vec4<f32>(
            color.rgb * (1.0 + sea_glints(in.uv, u.time) * u.sea_intensity),
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
    return vec4<f32>(color.rgb, color.a * u.opacity);
}
