pub mod types;

use std::f32::consts::PI;

use bevy::{
    asset::{Assets, RenderAssetUsages},
    ecs::system::{Commands, NonSend, Query, Res, ResMut},
    image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    math::{Quat, Vec3, primitives::Plane3d},
    mesh::{Mesh, Mesh3d},
    pbr::MeshMaterial3d,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        storage::ShaderStorageBuffer,
    },
    time::Time,
    transform::components::Transform,
};
use bevy_svg::prelude::Origin;
use colored_text::Colorize;

use crate::{
    audiolink::types::{
        Audiolink, AudiolinkDataTexture, AudiolinkMaterial, AudiolinkMaterialHandle,
    },
    pipewire::PipewireInput,
};

pub const SHADER_ASSET_PATH: &str = "audiolink.wgsl";
pub const SAMPLE_HISTORY: usize = 4096;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<AudiolinkMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let mut audiolink_data_texture = Image::new_uninit(
        Extent3d {
            width: 128,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    );

    audiolink_data_texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        label: Some("audiolink data texture image sampler".to_owned()),
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        lod_min_clamp: 1.0,
        lod_max_clamp: 1.0,
        compare: None,
        anisotropy_clamp: 1,
        ..ImageSamplerDescriptor::default()
    });

    let audiolink_data_texture_handle = images.add(audiolink_data_texture);

    commands.insert_resource(AudiolinkDataTexture(audiolink_data_texture_handle.clone()));

    let audiolink_data_lut: [f32; 240] = [
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
        0.907, 0.905, 0.903, 0.9,
    ];
    let audiolink_data_audio_data: [[f32; 4]; SAMPLE_HISTORY] =
        core::array::from_fn(|_| [0.0, 0.0, 0.0, 0.0]);

    let audiolink_data_lut = buffers.add(ShaderStorageBuffer::from(audiolink_data_lut));
    let audiolink_data_audio_data =
        buffers.add(ShaderStorageBuffer::from(audiolink_data_audio_data));

    let audiolink_material_handle = materials.add(AudiolinkMaterial {
        self_texture: audiolink_data_texture_handle,

        audiolink_data_gain: 1.0,
        audiolink_data_bass: 1.0,
        audiolink_data_trebble: 1.0,
        audiolink_data_fade_length: 0.8,
        audiolink_data_lut,
        audiolink_data_audio_data,
    });
    commands.insert_resource(AudiolinkMaterialHandle(audiolink_material_handle.clone()));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(audiolink_material_handle),
        Origin::Center,
        Transform {
            translation: Vec3::new(0.0, 0.0, -1000.0),
            scale: Vec3::new(750.0, 750.0, 750.0),
            rotation: Quat::from_rotation_x(PI * 0.5),
        },
    ));

    commands.spawn(Audiolink {
        cursor_move: false,

        left_smoothed_max: 0.0,

        left_on_alternate_sample: false,
        left_full_rate_buffer: vec![0.0; SAMPLE_HISTORY],
        left_half_rate_buffer: vec![0.0; SAMPLE_HISTORY],

        right_smoothed_max: 0.0,
        right_on_alternate_sample: false,
        right_full_rate_buffer: vec![0.0; SAMPLE_HISTORY],
        right_half_rate_buffer: vec![0.0; SAMPLE_HISTORY],
    });
}

pub fn update(
    time: Res<Time>,
    pipewire_input: NonSend<PipewireInput>,
    audiolink_material_handles: Res<AudiolinkMaterialHandle>,
    audiolink_data_texture: Res<AudiolinkDataTexture>,
    mut materials: ResMut<Assets<AudiolinkMaterial>>,
    mut query: Query<&mut Audiolink>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    if let Ok(mut audiolink) = query.single_mut() {
        let delta_time = time.delta_secs();

        let mut captured_samples_left = 0;
        let mut captured_samples_right = 0;

        while let Ok(pipewire_message) = pipewire_input.from_pipewire.try_recv() {
            match pipewire_message {
                crate::pipewire::PipewireIncomingMessage::LeftChannelData(data) => {
                    captured_samples_left += data.len();

                    audiolink.left_full_rate_buffer.reserve(1);
                    audiolink.left_half_rate_buffer.reserve(1);

                    for sample in data {
                        audiolink.left_full_rate_buffer.insert(0, sample);
                        audiolink.left_full_rate_buffer.pop();

                        audiolink.left_on_alternate_sample = !audiolink.left_on_alternate_sample;
                        if audiolink.left_on_alternate_sample {
                            audiolink.left_half_rate_buffer.insert(0, sample);
                            audiolink.left_half_rate_buffer.pop();
                        }
                    }
                }
                crate::pipewire::PipewireIncomingMessage::RightChannelData(data) => {
                    captured_samples_right += data.len();

                    audiolink.right_full_rate_buffer.reserve(1);
                    audiolink.right_half_rate_buffer.reserve(1);

                    for sample in data {
                        audiolink.right_full_rate_buffer.insert(0, sample);
                        audiolink.right_full_rate_buffer.pop();

                        audiolink.right_on_alternate_sample = !audiolink.right_on_alternate_sample;
                        if audiolink.right_on_alternate_sample {
                            audiolink.right_half_rate_buffer.insert(0, sample);
                            audiolink.right_half_rate_buffer.pop();
                        }
                    }
                }
                _ => {}
            }
        }

        if audiolink.cursor_move {
            print!("\x1B[{}A", 3);
        }
        println!(
            "Captured: Left {captured_samples_left} Samples, Right {captured_samples_right} Samples                   "
        );

        let mut left_max: f32 = 0.0;
        let mut right_max: f32 = 0.0;

        let material = materials.get_mut(&audiolink_material_handles.0).unwrap();

        let audio_data_buffer = buffers
            .get_mut(&material.audiolink_data_audio_data)
            .unwrap();

        let mut new_audiolink_data_audio_data: [[f32; 4]; SAMPLE_HISTORY] =
            [[0.0; 4]; SAMPLE_HISTORY];

        #[allow(clippy::needless_range_loop)]
        for i in 0..SAMPLE_HISTORY {
            let mut left_full_sample = 0.0;
            let mut right_full_sample = 0.0;

            let mut left_half_sample = 0.0;
            let mut right_half_sample = 0.0;

            if let Some(left_buffered_sample) = audiolink.left_full_rate_buffer.get(i) {
                left_full_sample = *left_buffered_sample;
                left_max = left_max.max(left_buffered_sample.abs());
            }
            if let Some(right_buffered_sample) = audiolink.right_full_rate_buffer.get(i) {
                right_full_sample = *right_buffered_sample;
                right_max = right_max.max(right_buffered_sample.abs());
            }

            if let Some(left_buffered_sample) = audiolink.left_full_rate_buffer.get(i) {
                left_half_sample = *left_buffered_sample;
                left_max = left_max.max(left_buffered_sample.abs());
            }
            if let Some(right_buffered_sample) = audiolink.right_full_rate_buffer.get(i) {
                right_half_sample = *right_buffered_sample;
                right_max = right_max.max(right_buffered_sample.abs());
            }

            new_audiolink_data_audio_data[i] = [
                (left_full_sample + right_full_sample) / 2.0,
                (left_full_sample - right_full_sample) / 2.0,
                (left_half_sample + right_half_sample) / 2.0,
                (left_half_sample - right_half_sample) / 2.0,
            ];
        }

        audio_data_buffer.set_data(new_audiolink_data_audio_data);

        print_vu(
            " Left",
            left_max,
            &mut audiolink.left_smoothed_max,
            delta_time,
        );
        print_vu(
            "Right",
            right_max,
            &mut audiolink.right_smoothed_max,
            delta_time,
        );

        audiolink.cursor_move = true;
    }
}

fn print_vu(name: &str, max: f32, smoothed_max: &mut f32, delta_time: f32) {
    *smoothed_max = max.max(*smoothed_max - 0.3 * delta_time);

    let peak = ((max * 30.0) as usize).clamp(0, 39);
    let smooth_peak = ((*smoothed_max * 30.0) as usize).clamp(0, 39);

    let smooth_peak_major_color = if *smoothed_max >= 1.0 {
        "ff0000"
    } else if *smoothed_max >= 0.80 {
        "ffff00"
    } else {
        "00ff00"
    };
    let smooth_peak_minor_color = if *smoothed_max >= 1.0 {
        "4b0000"
    } else if *smoothed_max >= 0.80 {
        "4b4b00"
    } else {
        "004b00"
    };

    print!("{name} Channel: {}", "▕".hex("ffffff"));
    if smooth_peak != peak {
        print!("{}{}", "█".hex("4b4b4b").repeat(peak), "█".hex("ffffff"));
        print!(
            "{}{}",
            "█"
                .hex(smooth_peak_minor_color)
                .repeat((smooth_peak - peak) - 1),
            "▕"
                .hex(smooth_peak_major_color)
                .on_hex(smooth_peak_minor_color)
        );
    } else {
        print!(
            "{}{}",
            "█".hex("4b4b4b").repeat(peak),
            "▉".on_hex(smooth_peak_major_color)
        );
    }
    print!(
        "{}{} Peak: {max:.3} ~ {:.3}",
        " ".repeat(40 - smooth_peak),
        "▏".hex("ffffff"),
        *smoothed_max
    );
    println!();
}
