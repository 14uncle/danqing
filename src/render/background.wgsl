// 窗口背景图渲染管线: 绘制全屏或等比缩放图片。
//
// 顶点携带归一化位置 (0..1) 与 UV;uniform 传递不透明度与场景淡化进度。
// 场景切换时绑定 from/to 两张场景图, 按 fade 交叉淡化;
// 单图与叠加层 (光晕/噪声) 把同一张图绑到两个槽位, fade 恒 0。

struct Uniforms {
    opacity: f32,
    fade: f32,
    _pad0: f32,
    _pad1: f32,
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
    let color = mix(c_from, c_to, u.fade);
    return vec4<f32>(color.rgb, color.a * u.opacity);
}
