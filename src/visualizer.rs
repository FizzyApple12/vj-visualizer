use bevy::{
    asset::{Asset, Assets},
    ecs::system::{Commands, ResMut},
    math::primitives::Plane3d,
    mesh::Mesh,
    pbr::{Material, MeshMaterial3d},
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    transform::components::Transform,
};

const SHADER_ASSET_PATH: &str = "visualizer.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

pub fn setup(
    mut _commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<CustomMaterial>>,
) {
    // commands.spawn((
    //     Mesh3d(meshes.add(Plane3d::default())),
    //     MeshMaterial3d(materials.add(CustomMaterial {})),
    //     Transform::from_xyz(0.0, 0.0, 0.0),
    // ));
}
