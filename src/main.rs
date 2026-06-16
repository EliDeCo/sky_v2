use bevy::prelude::*;
use bevy_flycam::prelude::*;
use bevy_fps_counter::FpsCounterPlugin;
use bevy_fragment_shader_plugin::prelude::*;

mod buffers;
use buffers::*;
mod ui;
use ui::PresentModeDropdownPlugin;

fn main() {
    App::new()
        //camera
        .add_plugins((DefaultPlugins, NoCameraPlayerPlugin))
        .insert_resource(MovementSettings {
            sensitivity: 0.00007,
            speed: 100.0,
        })
        .insert_resource(KeyBindings {
            move_ascend: KeyCode::KeyE,
            move_descend: KeyCode::KeyQ,
            ..Default::default()
        })
        .add_plugins(FpsCounterPlugin)
        .add_plugins(PresentModeDropdownPlugin)
        //shader
        .add_plugins(FullscreenFragmentPlugin::new("shaders/fragment.wgsl"))
        .register_uniform_buffer::<Uniform>(0, 0)
        .init_resource::<Uniform>()
        .register_uniform_buffer::<AtmosphereSettings>(1, 0)
        .init_resource::<AtmosphereSettings>()
        .init_resource::<SunPos>()
        //run
        .add_systems(Startup, (setup, update_settings))
        .add_systems(Update, (update_sun, update_uniform))
        .run();
}

fn setup(mut commands: Commands) {
    //camera
    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        FlyCam,
        Transform::from_xyz(0.0, 0.001, 0.0),
    ));
}