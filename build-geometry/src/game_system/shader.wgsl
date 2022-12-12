// Vertex shader

struct BezierCurve {
    p0: vec2<f32>,
    p1: vec2<f32>,
    p2: vec2<f32>,
    p3: vec2<f32>,
    segment_size: u32,
    stroke_width_half: f32,
};
@group(0) @binding(0)
var<uniform> curve: BezierCurve;

@group(1) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

fn segment_point(curve: BezierCurve, i: u32) -> vec2<f32> {
    let t = f32(i / 2u) / f32(curve.segment_size);

    let q0 = mix(curve.p0, curve.p1, t);
    let q1 = mix(curve.p1, curve.p2, t);
    let q2 = mix(curve.p2, curve.p3, t);

    let v0 = mix(q0, q1, t);
    let v1 = mix(q1, q2, t);

    let tangent = v1 - v0;
    let pos = tangent * t + v0;
    var normal = normalize(vec2<f32>(-tangent.y, tangent.x)) * curve.stroke_width_half;
    if (i & 1u) == 1u {
        normal.x = -normal.x;
        normal.y = -normal.y;
    }
    
    return pos + normal;
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