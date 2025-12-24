use std::f32::consts::PI;

use bevy::{
    asset::{Asset, AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        system::{Commands, Res, ResMut, Single},
    },
    image::Image,
    math::{Quat, Vec3, primitives::Plane3d},
    mesh::{Mesh, Mesh3d},
    pbr::{Material, MeshMaterial3d},
    reflect::TypePath,
    render::{alpha::AlphaMode, render_resource::AsBindGroup},
    shader::ShaderRef,
    transform::components::Transform,
};
use bevy_svg::prelude::{Origin, Svg3d};

use crate::audiolink::AudiolinkDataTexture;

const SHADER_ASSET_PATH: &str = "logo.wgsl";

#[derive(Component)]
pub struct Logo {
    pub material_handle: Handle<LogoBackgroundMaterial>,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LogoBackgroundMaterial {
    #[texture(0)]
    #[sampler(1)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
}

impl Material for LogoBackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LogoBackgroundMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let logo = asset_server.load("logo-no-overlap.svg");

    let logo_background_material = materials.add(LogoBackgroundMaterial {
        color_texture: None,
        alpha_mode: AlphaMode::Opaque,
    });

    commands.spawn((
        Svg3d(logo),
        Origin::Center,
        Transform {
            translation: Vec3::new(0.0, 0.0, -700.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::default(),
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(logo_background_material.clone()),
        Origin::Center,
        Transform {
            translation: Vec3::new(0.0, 0.0, -701.0),
            scale: Vec3::new(228.0, 228.0, 228.0),
            rotation: Quat::from_rotation_x(PI * 0.5),
        },
    ));

    commands.spawn(Logo {
        material_handle: logo_background_material,
    });
}

pub fn update(
    visualizer: Single<&mut Logo>,
    audiolink_data_texture: Res<AudiolinkDataTexture>,
    mut materials: ResMut<Assets<LogoBackgroundMaterial>>,
) {
    if let Some(material_reference) = materials.get_mut(visualizer.material_handle.id()) {
        material_reference.color_texture = Some(audiolink_data_texture.0.clone());
    }
}
