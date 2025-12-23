use std::f32::consts::PI;

use bevy::{
    asset::{Asset, AssetServer, Assets},
    color::LinearRgba,
    ecs::system::{Commands, Res, ResMut},
    math::{Quat, Vec3, primitives::Plane3d},
    mesh::{Mesh, Mesh3d},
    pbr::{Material, MeshMaterial3d},
    reflect::TypePath,
    render::{alpha::AlphaMode, render_resource::AsBindGroup},
    shader::ShaderRef,
    transform::components::Transform,
};
use bevy_svg::prelude::{Origin, Svg3d};

const SHADER_ASSET_PATH: &str = "logo.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    color: LinearRgba,
    alpha_mode: AlphaMode,
}

impl Material for CustomMaterial {
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
    mut materials: ResMut<Assets<CustomMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let logo = asset_server.load("logo-no-overlap.svg");

    commands.spawn((
        Svg3d(logo),
        Origin::Center,
        Transform {
            translation: Vec3::new(0.0, 0.0, -500.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::default(),
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(CustomMaterial {
            color: LinearRgba::BLUE,
            alpha_mode: AlphaMode::Opaque,
        })),
        Origin::Center,
        Transform {
            translation: Vec3::new(0.0, 0.0, -501.0),
            scale: Vec3::new(228.0, 228.0, 228.0),
            rotation: Quat::from_rotation_x(PI * 0.5),
        },
    ));
}
