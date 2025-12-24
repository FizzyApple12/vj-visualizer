#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

const ALPASS_DFT = vec2<i32>(0, 4);
const ALPASS_WAVEFORM = vec2<i32>(0, 6);

const TEXEL_WIDTH = 128;
const TEXEL_HEIGHT = 64;

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var audiolink_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var audiolink_sampler: sampler;

// fn oklab_to_linear_srgb(c: vec3<f32>) -> vec3<f32> {
//     let L = c.x;
//     let a = c.y;
//     let b = c.z;

//     let l_ = L + 0.3963377774 * a + 0.2158037573 * b;
//     let m_ = L - 0.1055613458 * a - 0.0638541728 * b;
//     let s_ = L - 0.0894841775 * a - 1.2914855480 * b;

//     let l = l_ * l_ * l_;
//     let m = m_ * m_ * m_;
//     let s = s_ * s_ * s_;

//     return vec3<f32>(
//         4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
//         -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
//         -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
//     );
// }

// fn get_audio_power(uv: vec2<f32>) -> vec3<f32> {
//     let lf_power: f32 = 0.0;

//     for (var bi = 0; bi < AUDIOLINK_ETOTALBINS / 3; bi++) {
//         lf_power += AudioLinkLerpMultiline(ALPASS_DFT + vec2<u32>(bi, 0)).r;
//     }

//     lf_power /= AUDIOLINK_ETOTALBINS / 3.0;
//     lf_power = clamp(lf_power * 5, 0.0, 1.0);

//     let tuv: vec2<f32> = uv;
//     let tuvp: vec2<f32> = mix(uv * 0.1 + 0.45, uv, lf_power);

//     let col: vec3<f32> = float3(
//         (((
//             AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuv.x * 2048, 2048.0), 0)).rrr
//             - AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuv.x * 2048, 2048.0), 0)).ggg
//         )
//         -
//         (
//             AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuv.y * 2048, 2048.0), 0)).rrr
//             + AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuv.y * 2048, 2048.0), 0)).ggg
//         )) +
//         ((
//             AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuv.x * 2048 + tuv.y * 2048, 2048.0), 0)).bbb
//             - AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuv.x * 2048 + tuv.y * 2048, 2048.0), 0)).aaa
//         )
//         -
//         (
//             AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuv.x * 2048 - tuv.y * 2048 + 2048, 2048.0), 0)).bbb
//             + AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuv.x * 2048 - tuv.y * 2048 + 2048, 2048.0), 0)).aaa
//         ))) * 0.5 +
//         (((
//             AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuvp.x * 2048, 2048.0), 0)).rrr
//             - AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuvp.x * 2048, 2048.0), 0)).ggg
//         )
//         -
//         (
//             AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuvp.y * 2048, 2048.0), 0)).rrr
//             + AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuvp.y * 2048, 2048.0), 0)).ggg
//         )) +
//         ((
//             AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuvp.x * 2048 + tuvp.y * 2048, 2048.0), 0)).bbb
//             - AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuvp.x * 2048 + tuvp.y * 2048, 2048.0), 0)).aaa
//         )
//         -
//         (
//             AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuvp.x * 2048 - tuvp.y * 2048 + 2048, 2048.0), 0)).bbb
//             + AudioLinkLerpMultiline(ALPASS_WAVEFORM + vec2<u32>(modf(tuvp.x * 2048 - tuvp.y * 2048 + 2048, 2048.0), 0)).aaa
//         ))) * 0.5
//     );

//     return pow(clamp(col, vec3<f32>(0, 0, 0), vec3<f32>(1, 1, 1)), 2);
// }

// @fragment
// fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
//     let power: vec3<f32> = get_audio_power(in.uv);

//     return vec4<f32>(oklab_to_linear_srgb(power), 1.0);
// }

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return textureSample(audiolink_texture, audiolink_sampler, vec2<f32>(in.uv.x, in.uv.y));
}
