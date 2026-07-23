// 矩形族渲染管线:实例化 quad + fragment SDF 圆角/抗锯齿。
//
// 每个实例:屏幕像素坐标 pos/size、RGBA 颜色、圆角半径、旋转角度。
// 顶点着色器用 vertex_index 生成单位 quad,像素坐标 → clip 空间;
// 片元着色器用 SDF 求圆角矩形有向距离,smoothstep 做边缘抗锯齿。

struct Uniforms {
    screen_size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,     // 相对矩形中心的像素坐标
    @location(1) half_size: vec2<f32>, // 矩形半尺寸
    @location(2) color: vec4<f32>,
    @location(3) radii: vec4<f32>,     // 四角圆角半径: 左上、右上、右下、左下
    @location(4) px: vec2<f32>,        // 片段像素坐标(用于裁剪)
    @location(5) clip_min: vec2<f32>,
    @location(6) clip_max: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) radii: vec4<f32>,
    @location(4) rotation: f32,
    @location(5) clip_min: vec2<f32>,
    @location(6) clip_max: vec2<f32>,
) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let c = corners[vi];
    let cos_r = cos(rotation);
    let sin_r = sin(rotation);
    // 绕矩形中心 (0.5,0.5) 旋转单位角点,再映射到像素坐标。
    let d = c - vec2<f32>(0.5, 0.5);
    let rd_pos = vec2<f32>(
        d.x * cos_r - d.y * sin_r,
        d.x * sin_r + d.y * cos_r
    );
    let rc = vec2<f32>(0.5, 0.5) + rd_pos;
    let px = pos + rc * size;
    // 像素坐标(原点左上,y 向下)→ clip(中心原点,y 向上)
    let clip = vec4<f32>(
        px.x / u.screen_size.x * 2.0 - 1.0,
        1.0 - px.y / u.screen_size.y * 2.0,
        0.0,
        1.0,
    );
    var out: VsOut;
    out.clip = clip;
    // local 保持未旋转的本地坐标;顶点位置旋转后,线性插值会自动
    // 把 screen-space 点映射回未旋转本地坐标,使 SDF 正确判定。
    out.local = d * size;
    out.half_size = size * 0.5;
    out.color = color;
    out.radii = radii;
    out.px = px;
    out.clip_min = clip_min;
    out.clip_max = clip_max;
    return out;
}

// 逐角圆角矩形有向距离(内部为负)。
// radii 顺序: 左上、右上、右下、左下。
fn sd_rounded_box_per_corner(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var ix: i32;
    if (p.x >= 0.0) {
        if (p.y >= 0.0) { ix = 2; } else { ix = 1; }
    } else {
        if (p.y >= 0.0) { ix = 3; } else { ix = 0; }
    }
    let rc = r[ix];
    let q = abs(p) - b + vec2<f32>(rc);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - rc;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // 按 clip 矩形剔除视口外片段(clip_max 不含)
    if in.px.x < in.clip_min.x || in.px.x >= in.clip_max.x
        || in.px.y < in.clip_min.y || in.px.y >= in.clip_max.y {
        discard;
    }
    let max_r = min(in.half_size.x, in.half_size.y);
    let r = min(in.radii, vec4<f32>(max_r));
    let d = sd_rounded_box_per_corner(in.local, in.half_size, r);
    // 过渡带取距离变化率, 但不超过矩形半尺寸:
    // 1px 细线的片元最多只深入边缘 0.5px, 过渡带收窄到半尺寸后
    // 中心行才能拿到满覆盖, 细线不再随亚像素相位发虚。
    let w = max(min(fwidth(d), max_r), 1e-4);
    let alpha = 1.0 - smoothstep(-w, w, d);
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
