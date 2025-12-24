use std::f32::consts::PI;

use bevy::{
    asset::{Asset, Assets, Handle},
    ecs::{
        component::Component,
        system::{Commands, Query, Res, ResMut, Single},
    },
    image::Image,
    math::{Quat, Vec3, primitives::Plane3d},
    mesh::{Mesh, Mesh3d},
    pbr::{Material, MeshMaterial3d},
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    transform::components::Transform,
    window::Window,
};
use bevy_svg::prelude::Origin;

use crate::audiolink::AudiolinkDataTexture;

const SHADER_ASSET_PATH: &str = "visualizer.wgsl";

#[derive(Component)]
pub struct Visualizer {
    pub material_handle: Handle<VisualizerMaterial>,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VisualizerMaterial {
    #[texture(0)]
    #[sampler(1)]
    color_texture: Option<Handle<Image>>,
}

impl Material for VisualizerMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VisualizerMaterial>>,
) {
    let visualizer_material = materials.add(VisualizerMaterial {
        color_texture: None,
    });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(visualizer_material.clone()),
        Origin::Center,
        Transform {
            translation: Vec3::new(0.0, 0.0, -905.0),
            scale: Vec3::new(750.0, 750.0, 750.0),
            rotation: Quat::from_rotation_x(PI * 0.5),
        },
        Visualizer {
            material_handle: visualizer_material,
        },
    ));
}

pub fn update(
    mut visualizer: Single<(&mut Visualizer, &mut Transform)>,
    audiolink_data_texture: Res<AudiolinkDataTexture>,
    mut materials: ResMut<Assets<VisualizerMaterial>>,
    windows: Query<&mut Window>,
) {
    if let Ok(window) = windows.single() {
        let aspect = window.resolution.width() / window.resolution.height();

        visualizer.1.scale = Vec3::new(750.0 * aspect, 750.0, 750.0);
    }

    if let Some(material_reference) = materials.get_mut(visualizer.0.material_handle.id()) {
        material_reference.color_texture = Some(audiolink_data_texture.0.clone());
    }
}
