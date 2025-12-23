use bevy::{
    asset::{Asset, Handle},
    ecs::{component::Component, resource::Resource},
    image::Image,
    pbr::Material,
    reflect::TypePath,
    render::{
        extract_resource::ExtractResource, render_resource::AsBindGroup,
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
};

use crate::audiolink::SHADER_ASSET_PATH;

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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct AudiolinkMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub self_texture: Handle<Image>,

    #[uniform(2)]
    pub audiolink_data_gain: f32,
    #[uniform(3)]
    pub audiolink_data_bass: f32,
    #[uniform(4)]
    pub audiolink_data_trebble: f32,
    #[uniform(5)]
    pub audiolink_data_fade_length: f32,
    #[storage(6, read_only)]
    pub audiolink_data_lut: Handle<ShaderStorageBuffer>,
    #[storage(7, read_only)]
    pub audiolink_data_audio_data: Handle<ShaderStorageBuffer>,
}

impl Material for AudiolinkMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

#[derive(Resource)]
pub struct AudiolinkMaterialHandle(pub Handle<AudiolinkMaterial>);

#[derive(Clone, Resource)]
pub struct AudiolinkDataTexture(pub Handle<Image>);

impl ExtractResource for AudiolinkDataTexture {
    type Source = Self;

    fn extract_resource(source: &Self::Source) -> Self {
        (*source).clone()
    }
}
