pub mod audiolink;
pub mod logo;
pub mod pipewire;
pub mod visualizer;

use bevy::prelude::*;

use crate::pipewire::PipewireInput;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipewire_input = PipewireInput::new()?;

    App::new()
        .add_plugins((
            DefaultPlugins,
            MaterialPlugin::<logo::CustomMaterial>::default(),
            MaterialPlugin::<audiolink::types::AudiolinkMaterial>::default(),
            MaterialPlugin::<visualizer::CustomMaterial>::default(),
            bevy_svg::prelude::SvgPlugin,
        ))
        .insert_non_send_resource(pipewire_input)
        .add_systems(
            Startup,
            (setup, logo::setup, audiolink::setup, visualizer::setup),
        )
        .add_systems(Update, audiolink::update)
        .run();

    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}
