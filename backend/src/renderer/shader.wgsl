struct VertexInput {
    @location(0) position: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> color: vec4<f32>;

@group(0) @binding(1)
var<uniform> proj_mat: mat4x4<f32>;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = proj_mat * model.position;
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return color;
}