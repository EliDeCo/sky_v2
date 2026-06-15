use bevy::{math::ops::atan, prelude::*};
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

#[derive(Resource, ShaderType, Clone, Default)]
pub struct AtmosphereSettings {
    atmosphere_height: f32,
    num_rayleigh_steps: i32,
    num_optical_depth_steps: i32,
    planet_radius: f32,
    planet_color: Vec3,
    rayleigh_scale_height: f32,
    rayleigh_beta: Vec3,
    sun_direction: Vec3,
    cos_sun_angular_radius: f32,
    atmosphere_radius: f32,
    planet_center: Vec3,
    solar_irradiance: f32,
    solar_radiance: f32,
    mie_beta: f32,
    mie_scale_height: f32,
    ozone_beta: Vec3,
    ozone_profile: Vec3, 
}

pub fn update_settings(mut settings: ResMut<AtmosphereSettings>) {
    settings.atmosphere_height = 100.0;
    settings.num_rayleigh_steps = 16;
    settings.num_optical_depth_steps = 16;
    settings.planet_radius = 6378.0;
    settings.planet_color = Vec3::new(0.196, 0.459, 0.145);
    settings.rayleigh_scale_height = 8.0;
    settings.rayleigh_beta = Vec3::new(5.8e-3, 13.5e-3, 33.1e-3);
    settings.sun_direction = Vec3::new(0.0, 0.025, -1.0).normalize();
    settings.cos_sun_angular_radius = (atan(695700. / 149597870.7)).cos(); //solar_radius / distance_to_sun
    settings.atmosphere_radius = settings.planet_radius + settings.atmosphere_height; 
    settings.planet_center = Vec3::new(0.0, -settings.planet_radius, 0.0);
    settings.solar_irradiance = 1361. * 0.01; //solar_irradiance * brightness
    let sun_solid_angle = std::f32::consts::TAU * (1. - settings.cos_sun_angular_radius);
    settings.solar_radiance = settings.solar_irradiance / sun_solid_angle;
    settings.mie_beta = 21e-3;
    settings.mie_scale_height = 1.2;
    settings.ozone_beta = Vec3::new(3.426, 8.298, 0.356) * 0.06 * 1e-2;
    settings.ozone_profile = Vec3::new(15., 22., 35.); //lower bound, highest concentration, upper bound (km)
}
