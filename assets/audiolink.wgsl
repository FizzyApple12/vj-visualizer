@group(0) @binding(0) var input: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output: texture_storage_2d<rgba32float, write>;

@group(0) @binding(2) var<uniform> audiolink_data_gain: f32;
@group(0) @binding(3) var<uniform> audiolink_data_bass: f32;
@group(0) @binding(4) var<uniform> audiolink_data_trebble: f32;
@group(0) @binding(5) var<uniform> audiolink_data_fade_length: f32;
@group(0) @binding(6) var<storage, read> audiolink_data_audio_data: array<vec4<f32>, 4096>;

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    textureStore(output, location, vec4(0.0, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let color = vec4(1.0, 1.0, 1.0, 1.0);

    textureStore(output, location, color);
}
