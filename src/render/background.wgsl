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

// 碎点: 分列 hash, 位置基本不动, 亮度低频明灭 (频率档位 {1,2}/8 Hz, 整数倍)。
const GLINT_DENSITY: f32 = 120.0;      // 列密度 (960px 窗 ≈ 8px/列)
const GLINT_RADIUS: f32 = 0.004;       // 点半径 (纵向 uv; 960px 窗 ≈ 5px 直径)
const GLINT_ASPECT: f32 = 1.5;         // 场景画布宽高比 (1536×1024), 圆点修正
const GLINT_BAND_TOP: f32 = 0.72;      // 散布带上缘 (uv.y, 对齐静态图第一叠波带)
const GLINT_BAND_SPAN: f32 = 0.26;     // 散布带纵向跨度 (至 uv.y ≈ 0.98)
const GLINT_GAIN: f32 = 0.14;          // 点亮度上限 (乘性提亮; 0.30 目测突兀, 调参轮 1)
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
    // 明灭频率取档位 {1,2}/8 Hz (整数倍, 保 8s 公共周期); smoothstep 缓起缓落。
    let k = 1.0 + floor(rnd * 2.0);
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
// 关键: 旧版用 sin(x * scale + ...) (x 主导) → 垂直条纹 + 漂移 = 水平光束 (Tyndall 效应)。
// 修正: 4 个 sin 全部用 comparable x 与 y 系数 (y/x ratio 0.7-1.0), 接近 45° 方向;
//       不同 ± sign + 不同 phase 打破对齐, 造 2D 噪声, 无 dominant direction。
//
// 系数 6/8/12/16 (旧 2/2.5/3.5/4.5) — 用户 2026-07-30 反馈 "灰蒙蒙一片":
//   旧系数 + scale=2 → 空间周期 1.57-0.70 uv = 1500-672 px, 几乎跟屏幕一样大,
//   每像素都在大梯度中段, 漂移时每像素变化小 (±0.05), 视觉无感, 看起来是平雾。
//   新系数 → 周期 0.52-0.20 uv = 503-192 px, 真正 fog 团尺寸, 漂移时每像素变化 ±0.20+,
//   视觉能感到"风在吹"。

// 漂移: x 累加 t*speed 偏移造"风吹过"。 调用方必须用 wrap-clean 速度 (1/16 = 0.0625),
// 才能保证 8s wrap 时 pattern 连续: 8 * 0.0625 * k = k/2 (k 为偶数), sin(k/2 * π) = 0,
// 所有 sin 项在 wrap 处都 = 0, 无 1 帧跳变 (用户 2026-07-30 反馈 "卡顿" 修复)。
// 当前所有调用方都用 speed = 0.0625 (1/16), 系数 k = 6/8/12/16 全部偶数, wrap-clean。
fn mist_pattern(uv: vec2<f32>, t: f32, speed: f32, scale: f32, phase: f32) -> f32 {
    let x = uv.x * scale + t * speed + phase;
    let y = uv.y * scale;
    // 4 个 diagonal sin: y/x 比例 0.7-0.9, 都接近 45° 方向, 空间周期 0.20-0.52 uv (192-503 px)。
    let v = sin(x * 6.0 + y * 5.0 + phase) * 0.30
          + sin(x * 8.0 - y * 7.0 + phase * 1.7) * 0.25
          + sin(x * 12.0 + y * 10.0 + phase * 2.3) * 0.25
          + sin(x * 16.0 - y * 13.0 + phase * 3.1) * 0.20;
    return v * 0.5 + 0.5; // 0..1
}

// ---- 山动效 (山场景) ----
// 云雾 (用户 2026-07-30 终审反馈 "山效果不好" + 图)。
// 旧版问题: 3 层 pattern 干涉 → 横向条纹("cloud bank"), 雾色冷灰在暖粉暮色上
// 高对比, alpha 0.40 + 整片提亮 = 失去 atmospheric 感, 读作"云团"而非"暮色山雾"。
//
// 修复: 单层 + 暖粉融入暮色 + 低 alpha, 让雾作为"暮色渐变上的微弱变化"而非"独立云团"。
// mask 收窄 0.30-0.85 → 0.40-0.95 (集中在山脊附近, 不覆盖整片天空)。
const MOUNTAIN_RIDGE_MIST_Y_TOP: f32 = 0.40;
const MOUNTAIN_RIDGE_MIST_Y_FULL: f32 = 0.75;
const MOUNTAIN_RIDGE_MIST_Y_END: f32 = 0.95;
const MOUNTAIN_RIDGE_MIST_ALPHA: f32 = 0.30;
// 雾色 (240, 200, 195) sRGB→linear: 暖粉, 接近暮色 (199, 172, 178) 但更亮,
// additive 叠加在暮色上读作"暮色加深", 不像"冷云覆盖"。
const MOUNTAIN_RIDGE_MIST_COLOR: vec3<f32> = vec3<f32>(0.870, 0.604, 0.572);

fn mountain_ridge_mist(uv: vec2<f32>, t: f32) -> vec3<f32> {
    // y mask: 0.40 软入 (暮色天区中部) → 0.75 满 (山脊上空) → 0.95 软出 (山脊)。
    let band = smoothstep(MOUNTAIN_RIDGE_MIST_Y_TOP, MOUNTAIN_RIDGE_MIST_Y_FULL, uv.y)
             * (1.0 - smoothstep(MOUNTAIN_RIDGE_MIST_Y_END, 1.0, uv.y));
    // 单层 pattern — 旧 3 层 (主 0.05 + 副反向 0.03 + 副副 0.025) 干涉出
    // 横向"cloud bank"条纹, 改单层消除干涉, 读作"风在暮色上轻吹"。
    // speed 0.04 → 0.0625 (1/16) — 8s wrap 修复 (2026-07-30 用户反馈 "卡顿")。
    let p = mist_pattern(uv, t, 0.0625, 2.0, 0.0);
    return MOUNTAIN_RIDGE_MIST_COLOR * p * band * MOUNTAIN_RIDGE_MIST_ALPHA;
}

// ---- 森林动效 (森林场景) ----
// 雾不烘焙 (用户 2026-07-30 终审反馈, 参考雨场景改造范式):
// export-scenes.py 森林配置已去掉 mist 字段, forest.png 不再绘制底雾;
// 运行时 forest_mist 全程序化生成 3 层云雾 (雨幕同构, 但 2D 各向同性,
// 无 dominant direction — 旧版 Tyndall 翻车教训)。
// 设计: 3 层 y-分层 + 不同 speed + 2D pattern, 总 alpha 峰 ≈ 0.44,
// 平均 ≈ 0.22, 雾带内可见但不过曝。
// 暂停 500ms 沉降: forest_intensity = 0, 雾消失, 回到裸静态图 (无底雾)。
// 命名: forest_mist (无 _motion 后缀, 因为现在没有"叠加在静态底雾上"这一说,
// 雾就是全程序化的主效果)。
const FOREST_MIST_COLOR: vec3<f32> = vec3<f32>(0.512, 0.604, 0.548);
// Layer A (中间重雾) 已删除 — 用户 2026-07-30 反馈 "森林去掉中层雾带"。
// 旧 Layer A: y=0.40, half 0.15, alpha 0.20, speed 0.04。
// 只保留下层 (B), 视觉重心下移到中林 y=0.68 与近林 y=0.88 之间,
// 读作"林下贴地雾气"。
// Layer B: 下半亮 (y=0.65, 半高 0.12), 快漂 0.06 uv/s — 用户 2026-07-30 反馈
// "靠下的雾气再明显一点", 半高 0.15 → 0.18 → 0.12 (聚焦下半),
// alpha 0.18 → 0.30 → 0.25 (去呼吸后密度减半, alpha 略减保持视觉剂量)。
const FOREST_MIST_B_Y: f32 = 0.691;
const FOREST_MIST_B_HALF: f32 = 0.159;
const FOREST_MIST_B_ALPHA: f32 = 0.25;
const FOREST_MIST_B_SPEED: f32 = 0.0625;
// Layer C (顶部轻雾) 已删除 — 用户 2026-07-30 反馈 "森林去掉靠上的雾带"。
// 旧 Layer C: y=0.22, half 0.12, alpha 0.10, speed -0.03。
// 保留中 (A) + 下 (B) 两层, 视觉重心下移, 顶光区 (静态 PNG 已有) 不被雾覆盖。

fn forest_mist_layer(uv: vec2<f32>, t: f32, y_peak: f32, y_half: f32, speed: f32, phase: f32, alpha: f32) -> f32 {
    // y mask: 1.0 at peak, 0 at ±half (smoothstep 0.5×half→full×half 软入)
    let band = 1.0 - smoothstep(y_half * 0.5, y_half, abs(uv.y - y_peak));
    let pattern = mist_pattern(uv, t, speed, 2.0, phase);
    return pattern * band * alpha;
}

fn forest_mist(uv: vec2<f32>, t: f32) -> vec3<f32> {
    // 单层雾带 (下), 用户 2026-07-30 反馈 "去掉中层 + 下层下移 50px"。
    // 中林 y=0.68 与近林 y=0.88 之间, 贴林下雾, 其它不动。
    // 无时间脉动 — 用户 2026-07-30 反馈 "去掉呼吸效果, 又不是篝火",
    // 雾是风驱(空间漂移, 持续), 不是火(中心辐射, 时间脉动)。
    let b = forest_mist_layer(uv, t, FOREST_MIST_B_Y, FOREST_MIST_B_HALF, FOREST_MIST_B_SPEED, 1.7, FOREST_MIST_B_ALPHA);
    return FOREST_MIST_COLOR * b;
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
    return vec4<f32>(color.rgb, color.a * u.opacity);
}
