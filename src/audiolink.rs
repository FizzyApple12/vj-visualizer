use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        graph::CameraDriverLabel,
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            binding_types::{storage_buffer_read_only, texture_storage_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
    },
    shader::PipelineCacheError,
};
use colored_text::Colorize;
use std::borrow::Cow;

use crate::pipewire::PipewireInput;

pub const SHADER_ASSET_PATH: &str = "audiolink.wgsl";

pub const SAMPLE_HISTORY: usize = 4096;

pub const AUDIOLINK_WIDTH: u32 = 128;
pub const AUDIOLINK_HEIGHT: u32 = 64;

pub const WORKGROUP_SIZE: u32 = 8;

#[derive(Component)]
pub struct Audiolink {
    pub cursor_move: bool,

    pub left_smoothed_max: f32,
    pub left_on_alternate_sample: bool,
    pub left_full_rate_buffer: Vec<f32>,
    pub left_half_rate_buffer: Vec<f32>,

    pub right_smoothed_max: f32,
    pub right_on_alternate_sample: bool,
    pub right_full_rate_buffer: Vec<f32>,
    pub right_half_rate_buffer: Vec<f32>,
}

pub struct AudiolinkComputePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct AudiolinkLabel;

#[derive(Resource, Clone, ExtractResource)]
pub struct AudiolinkImages {
    texture_a: Handle<Image>,
    texture_b: Handle<Image>,
}

#[derive(Clone, Resource)]
pub struct AudiolinkDataTexture(pub Handle<Image>);

#[derive(Resource, Clone, ExtractResource)]
pub struct AudiolinkAudioData([[f32; 4]; SAMPLE_HISTORY]);

#[derive(Resource, Clone, ExtractResource, ShaderType)]
pub struct AudiolinkUniforms {
    gain: f32,
    bass: f32,
    trebble: f32,
    fade_length: f32,
}

#[derive(Resource)]
pub struct AudiolinkImageBindGroups([BindGroup; 2]);

#[derive(Resource)]
pub struct AudiolinkPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

pub enum AudiolinkState {
    Loading,
    Init,
    Update(usize),
}

pub struct AudiolinkNode {
    state: AudiolinkState,
}

impl Default for AudiolinkNode {
    fn default() -> Self {
        Self {
            state: AudiolinkState::Loading,
        }
    }
}

impl Plugin for AudiolinkComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<AudiolinkImages>::default(),
            ExtractResourcePlugin::<AudiolinkAudioData>::default(),
            ExtractResourcePlugin::<AudiolinkUniforms>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update);

        let audiolink_render_app = app.sub_app_mut(RenderApp);
        audiolink_render_app
            .add_systems(RenderStartup, init_audiolink_pipeline)
            .add_systems(
                Render,
                prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
            );

        let mut audiolink_render_graph = audiolink_render_app
            .world_mut()
            .resource_mut::<RenderGraph>();
        audiolink_render_graph.add_node(AudiolinkLabel, AudiolinkNode::default());
        audiolink_render_graph.add_node_edge(AudiolinkLabel, CameraDriverLabel);
    }
}

fn init_audiolink_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
) {
    let texture_bind_group_layout = render_device.create_bind_group_layout(
        "AudiolinkImages",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                storage_buffer_read_only::<[[f32; 4]; SAMPLE_HISTORY]>(false),
                uniform_buffer::<AudiolinkUniforms>(false),
            ),
        ),
    );

    let shader = asset_server.load(SHADER_ASSET_PATH);

    let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![texture_bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("init")),
        ..default()
    });

    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![texture_bind_group_layout.clone()],
        shader,
        entry_point: Some(Cow::from("update")),
        ..default()
    });

    commands.insert_resource(AudiolinkPipeline {
        texture_bind_group_layout,
        init_pipeline,
        update_pipeline,
    });
}

#[allow(clippy::too_many_arguments)]
fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<AudiolinkPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    audiolink_images: Res<AudiolinkImages>,
    audiolink_audio_data: Res<AudiolinkAudioData>,
    audiolink_uniforms: Res<AudiolinkUniforms>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let view_a = gpu_images.get(&audiolink_images.texture_a).unwrap();
    let view_b = gpu_images.get(&audiolink_images.texture_b).unwrap();

    let mut audio_buffer = StorageBuffer::from(audiolink_audio_data.0);

    audio_buffer.write_buffer(&render_device, &queue);

    let mut uniform_buffer = UniformBuffer::from(audiolink_uniforms.into_inner());

    uniform_buffer.write_buffer(&render_device, &queue);

    let bind_group_a_to_b = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::sequential((
            &view_a.texture_view,
            &view_b.texture_view,
            &audio_buffer,
            &uniform_buffer,
        )),
    );
    let bind_group_b_to_a = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::sequential((
            &view_b.texture_view,
            &view_a.texture_view,
            &audio_buffer,
            &uniform_buffer,
        )),
    );

    commands.insert_resource(AudiolinkImageBindGroups([
        bind_group_a_to_b,
        bind_group_b_to_a,
    ]));
}

pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_target_texture(
        AUDIOLINK_WIDTH,
        AUDIOLINK_HEIGHT,
        TextureFormat::Rgba32Float,
    );
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
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

    let image_a = images.add(image.clone());
    let image_b = images.add(image);

    commands.insert_resource(AudiolinkDataTexture(image_a.clone()));

    commands.insert_resource(AudiolinkImages {
        texture_a: image_a,
        texture_b: image_b,
    });

    commands.insert_resource(AudiolinkAudioData([[0.0; 4]; SAMPLE_HISTORY]));

    commands.insert_resource(AudiolinkUniforms {
        gain: 1.0,
        bass: 1.0,
        trebble: 1.0,
        fade_length: 0.8,
    });

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

#[allow(clippy::too_many_arguments)]
pub fn update(
    mut audiolink: Single<&mut Audiolink>,
    mut audiolink_audio_data: ResMut<AudiolinkAudioData>,
    mut audiolink_data_texture: ResMut<AudiolinkDataTexture>,
    time: Res<Time>,
    pipewire_input: NonSend<PipewireInput>,
    images: Res<AudiolinkImages>,
) {
    if audiolink_data_texture.0 == images.texture_a {
        audiolink_data_texture.0 = images.texture_b.clone();
    } else {
        audiolink_data_texture.0 = images.texture_a.clone();
    }

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

    let mut new_audiolink_data_audio_data: [[f32; 4]; SAMPLE_HISTORY] = [[0.0; 4]; SAMPLE_HISTORY];

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

    audiolink_audio_data.0 = new_audiolink_data_audio_data;

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

impl render_graph::Node for AudiolinkNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<AudiolinkPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            AudiolinkState::Loading => {
                match pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                    CachedPipelineState::Ok(_) => {
                        self.state = AudiolinkState::Init;
                    }
                    CachedPipelineState::Err(PipelineCacheError::ShaderNotLoaded(_)) => {}
                    CachedPipelineState::Err(err) => {
                        panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                    }
                    _ => {}
                }
            }
            AudiolinkState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = AudiolinkState::Update(1);
                }
            }
            AudiolinkState::Update(0) => {
                self.state = AudiolinkState::Update(1);
            }
            AudiolinkState::Update(1) => {
                self.state = AudiolinkState::Update(0);
            }
            AudiolinkState::Update(_) => unreachable!(),
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let bind_groups = &world.resource::<AudiolinkImageBindGroups>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<AudiolinkPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        match self.state {
            AudiolinkState::Loading => {}
            AudiolinkState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();

                pass.set_bind_group(0, &bind_groups[0], &[]);
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(
                    AUDIOLINK_WIDTH / WORKGROUP_SIZE,
                    AUDIOLINK_HEIGHT / WORKGROUP_SIZE,
                    1,
                );
            }
            AudiolinkState::Update(index) => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();

                pass.set_bind_group(0, &bind_groups[index], &[]);
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(
                    AUDIOLINK_WIDTH / WORKGROUP_SIZE,
                    AUDIOLINK_HEIGHT / WORKGROUP_SIZE,
                    1,
                );
            }
        }

        Ok(())
    }
}
