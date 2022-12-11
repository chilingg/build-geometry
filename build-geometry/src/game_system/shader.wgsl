// Vertex shader

struct BezierCurve {
    p0: vec2<f32>,
    p1: vec2<f32>,
    p2: vec2<f32>,
    p3: vec2<f32>,
    segment_size: u32,
};
@group(0) @binding(0)
var<uniform> curve: BezierCurve;

@group(1) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

fn segment_point(curve: BezierCurve, i: u32) -> vec2<f32> {
    let t = f32(i) / f32(curve.segment_size);
    let t2 = t * t;
    let t3 = t2 * t;

    let m = 1.0 - t;
    let m2 = m * m;
    let m3 = m2 * m;

    return curve.p0 * m3 + 3.0 * curve.p1 * m2 * t + 3.0 * curve.p2 * m * t2 + curve.p3 * t3;
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    return view_proj * vec4<f32>(segment_point(curve, in_vertex_index), 0.0, 1.0);
}

// Fragment

@fragment
fn fs_main(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1., 1., 1., 1.,);
}