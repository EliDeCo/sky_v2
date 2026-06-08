use bevy::prelude::*;
use bevy_fragment_shader_plugin::prelude::*;

#[derive(Resource, ShaderType, Clone, Default)]
pub struct Uniform {
    resolution: Vec2,
    world_from_clip: Mat4,
}

pub fn update_uniform(
    mut uniform: ResMut<Uniform>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, transform) = (camera.0, camera.1);
    let clip_matrix = camera.clip_from_view();
    let transform_matrix = transform.to_matrix();
    let world_from_clip = transform_matrix * clip_matrix.inverse();

    uniform.resolution = Vec2::new(window.width(), window.height());
    uniform.world_from_clip = world_from_clip;
}
