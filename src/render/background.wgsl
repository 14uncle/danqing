// 窗口背景图渲染管线: 绘制全屏或等比缩放图片。
//
// 顶点携带归一化位置 (0..1) 与 UV;uniform 传递不透明度与场景淡化进度。
// 场景切换时绑定 from/to 两张场景图, 按 fade 交叉淡化;
// 单图与叠加层 (光晕/噪声) 把同一张图绑到两个槽位, fade 恒 0。
//
// uniform 后两个浮点位携带场景动效参数 (雨丝强度 + 动效时间);
// 雨丝强度为 0 时零贡献, 输出与静态逐像素一致。

struct Uniforms {
    opacity: f32,
    fade: f32,
    rain_intensity: f32,
    time: f32,
}

// ---- 雨丝动效 (雨场景试点) ----
// 参数集中于本段, 调参只动这里。三层速度取整数比 (0.25/0.5/0.75 周期/秒),
// 公共周期 4s, 与 Rust 侧 `RAIN_WRAP_SECS` 一致 (上传前取模, 保 f32 精度)。
const RAIN_SLANT: f32 = 0.12;        // 斜率: x 随 y 的偏移 (风向)
const RAIN_YSCALE: f32 = 0.5;        // 纵向压缩: 同屏每列最多一段雨丝
const RAIN_GAIN: f32 = 0.35;         // 总亮度上限 (线性空间 additive)

const RAIN_DENSITY_FAR: f32 = 25.0;  // 远层: 细、慢、淡
const RAIN_SPEED_FAR: f32 = 0.25;
const RAIN_WIDTH_FAR: f32 = 0.10;
const RAIN_BRIGHT_FAR: f32 = 0.30;

const RAIN_DENSITY_MID: f32 = 40.0;  // 中层
const RAIN_SPEED_MID: f32 = 0.5;
const RAIN_WIDTH_MID: f32 = 0.08;
const RAIN_BRIGHT_MID: f32 = 0.45;

const RAIN_DENSITY_NEAR: f32 = 60.0; // 近层: 粗、快、亮
const RAIN_SPEED_NEAR: f32 = 0.75;
const RAIN_WIDTH_NEAR: f32 = 0.06;
const RAIN_BRIGHT_NEAR: f32 = 0.60;

fn rain_hash(p: f32) -> f32 {
    return fract(sin(p * 127.1) * 43758.5453);
}

// 单层雨丝: density 列密度, speed 下落速度 (fract 周期/秒),
// width 丝头宽度, bright 亮度权重。
fn rain_layer(uv: vec2<f32>, t: f32, density: f32, speed: f32, width: f32, bright: f32) -> f32 {
    let x = uv.x + uv.y * RAIN_SLANT; // 斜向拉条
    let col = floor(x * density);
    let rnd = rain_hash(col * 1.37);
    // 相位随机 (常量), 速度全列一致: 雨的真实感来自同速不同相,
    // 同时保证公共周期成立 (速度不带逐列抖动)。
    let y = fract(uv.y * RAIN_YSCALE - t * speed + rnd * 7.0);
    let streak = smoothstep(0.0, width, y) * (1.0 - smoothstep(width, width * 4.0, y));
    let on = step(0.6, rain_hash(col * 3.1 + 17.0)); // 40% 的列有雨
    return streak * on * bright;
}

fn rain_overlay(uv: vec2<f32>, t: f32) -> f32 {
    var acc = rain_layer(uv, t, RAIN_DENSITY_FAR, RAIN_SPEED_FAR, RAIN_WIDTH_FAR, RAIN_BRIGHT_FAR);
    acc += rain_layer(uv, t, RAIN_DENSITY_MID, RAIN_SPEED_MID, RAIN_WIDTH_MID, RAIN_BRIGHT_MID);
    acc += rain_layer(uv, t, RAIN_DENSITY_NEAR, RAIN_SPEED_NEAR, RAIN_WIDTH_NEAR, RAIN_BRIGHT_NEAR);
    return min(acc, 1.0) * RAIN_GAIN;
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
    let c_from = textureSample(tex_from, samp_from, in.uv);
    let c_to = textureSample(tex_to, samp_to, in.uv);
    var color = mix(c_from, c_to, u.fade);
    if (u.rain_intensity > 0.0) {
        // 线性空间 additive 亮度叠加 (sRGB 纹理采样已转线性)。
        color = vec4<f32>(
            color.rgb + vec3<f32>(rain_overlay(in.uv, u.time) * u.rain_intensity),
            color.a,
        );
    }
    return vec4<f32>(color.rgb, color.a * u.opacity);
}
