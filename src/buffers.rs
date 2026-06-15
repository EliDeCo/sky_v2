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
    scale_height: f32,
    rayleigh_beta: Vec3,
    sun_direction: Vec3,
    cos_sun_angular_radius: f32,
    atmosphere_radius: f32,
    planet_center: Vec3,
    solar_irradiance: f32,
    solar_radiance: f32
}

pub fn update_settings(mut settings: ResMut<AtmosphereSettings>) {
    let mut reader = csv::Reader::from_path("configs.csv").unwrap();
    let values: Vec<String> = reader
        .records()
        .filter_map(|r| r.ok())
        .filter_map(|r| r.get(1).map(|s| s.to_string()))
        .collect();

    let parts: Vec<f32> = values[4]
        .split(",")
        .map(|x| x.trim().parse().unwrap())
        .collect();
    let planet_color = Vec3::new(parts[0], parts[1], parts[2]);

    let parts: Vec<f32> = values[6]
        .split(",")
        .map(|x| x.trim().parse().unwrap())
        .collect();
    let rayleigh_beta = Vec3::new(parts[0], parts[1], parts[2]);

    let parsed_values: Vec<f32> = values
        .into_iter()
        .enumerate()
        .map(|(i, x)| {
            if i == 4 || i == 6 {
                return 0.0;
            }
            return x.trim().parse().unwrap();
        })
        .collect();

    let brightness = parsed_values[10];

    settings.atmosphere_height = parsed_values[0];
    settings.num_rayleigh_steps = parsed_values[1] as i32;
    settings.num_optical_depth_steps = parsed_values[2] as i32;
    settings.planet_radius = parsed_values[3];
    settings.planet_color = planet_color;
    settings.scale_height = parsed_values[5];
    settings.rayleigh_beta = rayleigh_beta;
    settings.sun_direction = Vec3::new(0.0, 0.1, -1.0).normalize();
    settings.cos_sun_angular_radius = (atan(parsed_values[8] / parsed_values[7])).cos();
    settings.atmosphere_radius = settings.planet_radius + settings.atmosphere_height;
    settings.planet_center = Vec3::new(0.0, -settings.planet_radius, 0.0);
    settings.solar_irradiance = parsed_values[9] * brightness;
    let sun_solid_angle = std::f32::consts::TAU * (1. - settings.cos_sun_angular_radius);
    settings.solar_radiance = settings.solar_irradiance / sun_solid_angle;
}
