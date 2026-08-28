// 图像纹理渲染管线:实例化 quad,采样 RGBA 纹理。
//
// 实例:屏幕像素坐标 dst_pos/dst_size、纹理 uv(0..1 归一化)、裁剪矩形。
// 片元采样纹理 RGBA 颜色,支持裁剪。

struct Uniforms {
    screen_size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;
@group(1) @binding(0)
var img_tex: texture_2d<f32>;
@group(1) @binding(1)
var img_samp: sampler;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) px: vec2<f32>,        // 片段像素坐标(用于裁剪)
    @location(2) clip_min: vec2<f32>,
    @location(3) clip_max: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) dst_pos: vec2<f32>,
    @location(1) dst_size: vec2<f32>,
    @location(2) uv_min: vec2<f32>,
    @location(3) uv_max: vec2<f32>,
    @location(4) clip_min: vec2<f32>,
    @location(5) clip_max: vec2<f32>,
) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let c = corners[vi];
    let px = dst_pos + c * dst_size;
    let clip = vec4<f32>(
        px.x / u.screen_size.x * 2.0 - 1.0,
        1.0 - px.y / u.screen_size.y * 2.0,
        0.0,
        1.0,
    );
    var out: VsOut;
    out.clip = clip;
    out.uv = mix(uv_min, uv_max, c);
    out.px = px;
    out.clip_min = clip_min;
    out.clip_max = clip_max;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // 按 clip 矩形剔除视口外片段(clip_max 不含)
    if in.px.x < in.clip_min.x || in.px.x >= in.clip_max.x
        || in.px.y < in.clip_min.y || in.px.y >= in.clip_max.y {
        discard;
    }
    return textureSample(img_tex, img_samp, in.uv);
}
