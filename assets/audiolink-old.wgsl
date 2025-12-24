#import bevy_pbr::forward_io::VertexOutput
// partly ported from https://github.com/llealloo/audiolink/blob/master/Packages/com.llealloo.audiolink/Runtime/Shaders/AudioLink.shader

const TWO_PI = 6.28318530718;

const ALPASS_DFT = vec2<f32>(0.0, 4.0);
const ALPASS_WAVEFORM = vec2<f32>(0.0, 6.0);

const TEXEL_WIDTH = 128.0;
const TEXEL_HEIGHT = 64.0;

const AUDIOLINK_SAMPHIST = 3069;
const AUDIOLINK_EXPBINS = 24;
const AUDIOLINK_EXPOCT = 10;
const AUDIOLINK_ETOTALBINS = (AUDIOLINK_EXPBINS * AUDIOLINK_EXPOCT);
const AUDIOLINK_WIDTH = 128;
const AUDIOLINK_SPS = 48000;
const AUDIOLINK_BOTTOM_FREQUENCY = 13.75;
const AUDIOLINK_BASE_AMPLITUDE = 2.5;
const AUDIOLINK_DELAY_COEFFICIENT_MIN = 0.3;
const AUDIOLINK_DELAY_COEFFICIENT_MAX = 0.9;
const AUDIOLINK_DFT_Q = 4.0;
const AUDIOLINK_TREBLE_CORRECTION = 5.0;

const AUDIOLINK_LUT = array<f32, 240>(
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.001,
    0.002, 0.003, 0.004, 0.005, 0.006, 0.008, 0.01, 0.012, 0.014, 0.017, 0.02, 0.022, 0.025,
    0.029, 0.032, 0.036, 0.04, 0.044, 0.048, 0.053, 0.057, 0.062, 0.067, 0.072, 0.078, 0.083,
    0.089, 0.095, 0.101, 0.107, 0.114, 0.121, 0.128, 0.135, 0.142, 0.149, 0.157, 0.164, 0.172,
    0.18, 0.188, 0.196, 0.205, 0.213, 0.222, 0.23, 0.239, 0.248, 0.257, 0.266, 0.276, 0.285,
    0.294, 0.304, 0.313, 0.323, 0.333, 0.342, 0.352, 0.362, 0.372, 0.381, 0.391, 0.401, 0.411,
    0.421, 0.431, 0.441, 0.451, 0.46, 0.47, 0.48, 0.49, 0.499, 0.509, 0.519, 0.528, 0.538,
    0.547, 0.556, 0.565, 0.575, 0.584, 0.593, 0.601, 0.61, 0.619, 0.627, 0.636, 0.644, 0.652,
    0.66, 0.668, 0.676, 0.684, 0.691, 0.699, 0.706, 0.713, 0.72, 0.727, 0.734, 0.741, 0.747,
    0.754, 0.76, 0.766, 0.772, 0.778, 0.784, 0.79, 0.795, 0.801, 0.806, 0.811, 0.816, 0.821,
    0.826, 0.831, 0.835, 0.84, 0.844, 0.848, 0.853, 0.857, 0.861, 0.864, 0.868, 0.872, 0.875,
    0.879, 0.882, 0.885, 0.888, 0.891, 0.894, 0.897, 0.899, 0.902, 0.904, 0.906, 0.909, 0.911,
    0.913, 0.914, 0.916, 0.918, 0.919, 0.921, 0.922, 0.924, 0.925, 0.926, 0.927, 0.928, 0.928,
    0.929, 0.929, 0.93, 0.93, 0.93, 0.931, 0.931, 0.93, 0.93, 0.93, 0.93, 0.929, 0.929, 0.928,
    0.927, 0.926, 0.925, 0.924, 0.923, 0.922, 0.92, 0.919, 0.917, 0.915, 0.913, 0.911, 0.909,
    0.907, 0.905, 0.903, 0.9
);

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var last_pixel_data_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var last_pixel_data_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> audiolink_data_gain: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> audiolink_data_bass: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> audiolink_data_trebble: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> audiolink_data_fade_length: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var<storage, read> audiolink_data_audio_data: array<vec4<f32>, 4096>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let guv: vec2<f32> = in.uv.xy;
    let coordinateGlobal: vec2<f32> = vec2<f32>(round(guv * vec2<f32>(TEXEL_WIDTH, TEXEL_HEIGHT) - 0.5));

    // i know this is bad for performance but idk how to properly bind multiple passes in wgsl so it's all i get
    if coordinateGlobal.y > 3.0 && coordinateGlobal.y <= 5.0 {
        let coordinateLocal: vec2<f32> = vec2<f32>(coordinateGlobal.x - ALPASS_DFT.x, coordinateGlobal.y - ALPASS_DFT.y);
        let last: vec4<f32> = textureSample(last_pixel_data_texture, last_pixel_data_sampler, coordinateGlobal);

        let note: i32 = i32(i32(coordinateLocal.y) * AUDIOLINK_WIDTH + i32(coordinateLocal.x));
        var amplitude: vec2<f32> = vec2<f32>(0.0, 0.0);
        var phase: f32 = 0.0;
        var phaseDelta: f32 = pow(2.0, f32(note) / f32(AUDIOLINK_EXPBINS));

        phaseDelta = ((phaseDelta * AUDIOLINK_BOTTOM_FREQUENCY) / AUDIOLINK_SPS) * TWO_PI * 2.0; // 2 here because we're at 24kSPS
        phase = -phaseDelta * f32(AUDIOLINK_SAMPHIST) / 2.0;     // Align phase so 0 phase is center of window.

        // DFT Window
        let halfWindowSize: f32 = AUDIOLINK_DFT_Q / (phaseDelta / TWO_PI);
        let windowRange: i32 = i32(floor(halfWindowSize) + 1);
        var totalWindow: f32 = 0.0;

        // For ??? reason, this is faster than doing a clever indexing which only searches the space that will be used.
        for (var idx = 0; idx < AUDIOLINK_SAMPHIST / 2; idx++) {
            // XXX TODO: Try better windows, this is just a triangle.
            let window: f32 = max(0.0, halfWindowSize - abs(f32(idx) - (f32(AUDIOLINK_SAMPHIST) / 2.0 - halfWindowSize)));
            let af: f32 = audiolink_data_audio_data[idx].x;

            // Sin and cosine components to convolve.
            let sinCos: vec2<f32> = vec2<f32>(sin(phase), cos(phase));

            // Step through, one sample at a time, multiplying the sin and cos values by the incoming signal.
            amplitude += sinCos * af * window;
            totalWindow += window;
            phase += phaseDelta;
        }
        let magPhase: f32 = atan2(amplitude.y, amplitude.x);
        var mag: f32 = (length(amplitude) / totalWindow) * AUDIOLINK_BASE_AMPLITUDE * audiolink_data_gain;

        // Treble compensation
        mag *= (AUDIOLINK_LUT[min(note, 239)] * AUDIOLINK_TREBLE_CORRECTION + 1);

        // Filtered output, also use FadeLength to lerp delay coefficient min/max for added smoothing effect
        var magFilt: f32 = mix(mag, last.z, mix(AUDIOLINK_DELAY_COEFFICIENT_MIN, AUDIOLINK_DELAY_COEFFICIENT_MAX, audiolink_data_fade_length));

        // Filtered EQ'd output
        var freqNormalized: f32 = f32(note) / f32(AUDIOLINK_EXPOCT * AUDIOLINK_EXPBINS);
        var magEQ: f32 = magFilt * (((1.0 - freqNormalized) * audiolink_data_bass) + (freqNormalized * audiolink_data_trebble));

        // Red:   Spectrum power, served straight up
        // Green: Filtered power EQ'd
        // Blue:  Filtered spectrum
        // Alpha: Phase of the signal
        return vec4<f32>(mag, magEQ, magFilt, magPhase);
    } else if coordinateGlobal.y > 5.0 && coordinateGlobal.y <= 21.0 {
        let coordinateLocal: vec2<f32> = vec2<f32>(coordinateGlobal.x - ALPASS_WAVEFORM.x, coordinateGlobal.y - ALPASS_WAVEFORM.y);
        var frame: i32 = i32(coordinateLocal.x) + i32(coordinateLocal.y) * AUDIOLINK_WIDTH;

        var incomingGain: f32 = 1.0;

        var ret: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0); // [ native 48k mono, difference between left and right at 48k, native 24k mono, difference between left and right at 24k]

        if frame < 4092 {
            ret.x = audiolink_data_audio_data[frame].x;
            ret.y = audiolink_data_audio_data[frame].y;
            ret.z = audiolink_data_audio_data[frame].z;
            ret.w = audiolink_data_audio_data[frame].w;
        }

        return ret;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
