// 窗口背景图渲染管线: 绘制全屏或等比缩放图片。
//
// 顶点携带归一化位置 (0..1) 与 UV;uniform 仅传递不透明度。

struct Uniforms {
    opacity: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0)
var<uniform> u: Uniforms;

@group(1) @binding(0)
var tex: texture_2d<f32>;

@group(1) @binding(1)
var samp: sampler;

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
    let color = textureSample(tex, samp, in.uv);
    return vec4<f32>(color.rgb, color.a * u.opacity);
}
