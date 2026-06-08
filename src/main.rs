use bevy::prelude::*;
use bevy_flycam::prelude::*;
use bevy_fragment_shader_plugin::prelude::*;

mod buffers;
use buffers::*;

fn main() {
    App::new()

        //camera
        .add_plugins((DefaultPlugins, NoCameraPlayerPlugin))
        .insert_resource(MovementSettings {
            sensitivity: 0.00007,
            speed: 10.0,
        })
        .insert_resource(KeyBindings {
            move_ascend: KeyCode::KeyE,
            move_descend: KeyCode::KeyQ,
            ..Default::default()
        })


        //shader
        .add_plugins(FullscreenFragmentPlugin::new("shaders/fragment.wgsl"))
        .register_uniform_buffer::<Uniform>(0, 0)
        .init_resource::<Uniform>()


        //run
        .add_systems(Startup, setup)
        .add_systems(Update, update_uniform)
        .run();
}


fn setup(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Msaa::Off, FlyCam, Transform::from_xyz(0.0, 10.0, 0.0)));
}
