use bevy::{prelude::*, window::PresentMode};
use bevy_flycam::prelude::*;
use bevy_fps_counter::FpsCounterPlugin;
use bevy_fragment_shader_plugin::prelude::*;

mod buffers;
use buffers::*;

fn main() {
    App::new()
        //camera
        .add_plugins((DefaultPlugins, NoCameraPlayerPlugin))
        .insert_resource(MovementSettings {
            sensitivity: 0.00007,
            speed: 10000.0,
        })
        .insert_resource(KeyBindings {
            move_ascend: KeyCode::KeyE,
            move_descend: KeyCode::KeyQ,
            ..Default::default()
        })
        .add_plugins(FpsCounterPlugin)
        //shader
        .add_plugins(FullscreenFragmentPlugin::new("shaders/fragment.wgsl"))
        .register_uniform_buffer::<Uniform>(0, 0)
        .init_resource::<Uniform>()
        //run
        .add_systems(Startup, setup)
        .add_systems(Update, (update_uniform, toggle_vsync))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        FlyCam,
        Transform::from_xyz(0.0, 10.0, 0.0),
    ));
}

fn toggle_vsync(
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        window.present_mode = match window.present_mode {
            PresentMode::AutoVsync => PresentMode::AutoNoVsync,
            PresentMode::AutoNoVsync => PresentMode::AutoVsync,
            _ => PresentMode::AutoVsync,
        }
    }
}
