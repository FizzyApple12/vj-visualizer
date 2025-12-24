#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

const TWO_PI = 6.28318530718;
const PI = 3.14159265359;

const ALPASS_DFT = vec2<f32>(0.0, 4.0);
const ALPASS_WAVEFORM = vec2<f32>(0.0, 6.0);

const AUDIOLINK_WIDTH = 128;
const AUDIOLINK_HEIGHT = 64;

const AUDIOLINK_EXPBINS = 24;
const AUDIOLINK_EXPOCT = 10;
const AUDIOLINK_ETOTALBINS = (AUDIOLINK_EXPBINS * AUDIOLINK_EXPOCT);

const START_HUE = 240.0;
const END_HUE = 360.0;

const SAMPLES_USED = 512.0;

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var audiolink_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var audiolink_sampler: sampler;

fn oklch_to_oklab(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        c.x,
        c.y * cos(c.z * PI / 180),
        c.y * sin(c.z * PI / 180)
    );
}

fn oklab_to_linear_srgb(c: vec3<f32>) -> vec3<f32> {
    let L = c.x;
    let a = c.y;
    let b = c.z;

    let l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = L - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    return vec3<f32>(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    );
}

fn audiolink_sample_multiline(xycoord: vec2<f32>) -> vec4<f32> {
    return textureSample(audiolink_texture, audiolink_sampler, vec2<f32>((xycoord.x % AUDIOLINK_WIDTH) / AUDIOLINK_WIDTH, (xycoord.y + xycoord.x / AUDIOLINK_WIDTH) / AUDIOLINK_HEIGHT));
}

fn audiolink_sample_lerp_multiline(xy: vec2<f32>) -> vec4<f32> {
    return mix(audiolink_sample_multiline(xy), audiolink_sample_multiline(xy + vec2<f32>(1.0, 0.0)), fract(xy.x));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = (in.uv.x) + (1.0 - in.uv.y);

    let sample_value: f32 = pow(audiolink_sample_lerp_multiline(ALPASS_WAVEFORM + vec2<f32>(f32(distance * SAMPLES_USED / 2), 0.0)).r + 1.0, 2.0) / 2.0;

    // return vec4<f32>(in.uv.x, in.uv.y, 1.0, 1.0);
    // return textureSample(audiolink_texture, audiolink_sampler, vec2<f32>(in.uv.x, in.uv.y));
    return vec4<f32>(oklab_to_linear_srgb(oklch_to_oklab(vec3<f32>(0.7101, 0.1301, mix(START_HUE, END_HUE, sample_value)))), 1.0);
}
