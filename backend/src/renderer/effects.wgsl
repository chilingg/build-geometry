@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var<storage, read_write> buf: array<u32>;

let ALIGNMENT = 256;

let LUMA = vec3<f32>(0.3, 0.55, 0.15);
let FXAA_ABSOLUTE_LUMA_THRESHoLD = 0.03;
let FXAA_RELATIVE_LUMA_THRESHOLD = 0.06;

var<private> LUMA_WEIGHT: array<array<f32, 3>, 3> = array<array<f32, 3>, 3>(
    array<f32, 3>(1.0, 2.0, 1.0),
    array<f32, 3>(2.0, 0.0, 2.0),
    array<f32, 3>(1.0, 2.0, 1.0),
);

fn color_to_argb8(color: vec4<f32>) -> u32 {
    return
          u32(color.a * 255.0) << 24u
        | u32(color.r * 255.0) << 16u
        | u32(color.g * 255.0) << 8u
        | u32(color.b * 255.0);
}

fn anti_aliasomg(coords: vec2<i32>, width: i32) -> vec4<f32> {
    var samples: array<array<vec4<f32>, 3>, 3>;
    var samples_luma: array<array<f32, 3>, 3>;
    for (var i = -1; i < 2; i += 1) {
        for (var j = -1; j < 2; j += 1) {
            samples[j+1][i+1] = textureLoad(input_texture, vec2<i32>(coords.x + i, coords.y +j), 0);
            samples_luma[j+1][i+1] = dot(samples[j+1][i+1].rgb, LUMA);
        }
    }

    let luma_max = max(
        max(samples_luma[0][1], max(samples_luma[2][1], samples_luma[1][1])),
        max(samples_luma[1][0], samples_luma[1][2])
    );
    let luma_min = min(
        min(samples_luma[0][1], min(samples_luma[2][1], samples_luma[1][1])),
        min(samples_luma[1][0], samples_luma[1][2])
    );
    let luma_contrast = luma_max - luma_min;

    if luma_contrast < max(FXAA_ABSOLUTE_LUMA_THRESHoLD, FXAA_RELATIVE_LUMA_THRESHOLD * luma_max) {
        return samples[1][1];
    }
    
    var filter_value = 0.0;
    var count_weight = 0.0;
    for (var i = 0; i < 3; i += 1) {
        for (var j = 0; j < 3; j += 1) {
            filter_value += samples_luma[i][j] * LUMA_WEIGHT[i][j];
            count_weight += LUMA_WEIGHT[i][j];
        }
    }
    filter_value = abs(filter_value / count_weight - samples_luma[1][1]);
    filter_value = clamp(filter_value / luma_contrast, 0.0, 1.0);

    // let pixel_blend = pow(smoothstep(0.0, 1.0, filter_value), 2.0);

    let vertical_luma = 
        abs(samples_luma[0][1] + samples_luma[2][1] - 2.0 * samples_luma[1][1]) * 2.0 
        + abs(samples_luma[0][2] + samples_luma[2][2] - 2.0 * samples_luma[1][2])
        + abs(samples_luma[0][0] + samples_luma[2][0] - 2.0 * samples_luma[1][0]);

    let horizontal_luma = 
        abs(samples_luma[1][0] + samples_luma[1][2] - 2.0 * samples_luma[1][1]) * 2.0 
        + abs(samples_luma[0][0] + samples_luma[0][2] - 2.0 * samples_luma[0][1])
        + abs(samples_luma[2][0] + samples_luma[2][2] - 2.0 * samples_luma[2][1]);
    
    var pixel_index: vec2<i32>;
    if vertical_luma > horizontal_luma {
        if abs(samples_luma[0][1] - samples_luma[1][1]) < abs(samples_luma[2][1] - samples_luma[1][1]) {
            pixel_index = vec2<i32>(1, 2);
        } else {
            pixel_index = vec2<i32>(1, 0);
        }
    } else {
        if abs(samples_luma[1][0] - samples_luma[1][1]) < abs(samples_luma[1][2] - samples_luma[1][1]) {
            pixel_index = vec2<i32>(2, 1);
        } else {
            pixel_index = vec2<i32>(0, 1);
        }
    }

    return mix(samples[1][1], samples[pixel_index.y][pixel_index.x], filter_value);
}

@compute @workgroup_size(16, 16)
fn cp_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dimensions = textureDimensions(input_texture);
    let coords = vec2<i32>(global_id.xy);

    if(coords.x >= dimensions.x || coords.y >= dimensions.y) {
        return;
    }

    let buffer_width = (ALIGNMENT - dimensions.x * 4 % ALIGNMENT) / 4 + dimensions.x;
    let color = anti_aliasomg(coords, buffer_width);

    buf[buffer_width*coords.y+coords.x] = color_to_argb8(color);
}