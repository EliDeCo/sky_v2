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
    num_view_ray_steps: i32,
    num_sun_ray_steps: i32,
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
    mie_beta_extinction: f32,
    mie_g: f32,
    mie_scale_height: f32,
    ozone_beta: Vec3,
    ozone_profile: Vec3,
}

pub fn update_settings(mut settings: ResMut<AtmosphereSettings>) {
    //all length units in km
    //irradiance in W/m^2
    settings.atmosphere_height = 100.0;
    settings.num_view_ray_steps = 10;
    settings.num_sun_ray_steps = 10;
    settings.planet_radius = 6378.0;
    settings.planet_color = Vec3::new(0.196, 0.459, 0.145);
    settings.rayleigh_scale_height = 8.0;
    settings.rayleigh_beta = Vec3::new(5.8e-3, 13.5e-3, 33.1e-3);
    settings.cos_sun_angular_radius = (atan(695700. / 149597870.7)).cos(); //solar_radius / distance_to_sun
    settings.atmosphere_radius = settings.planet_radius + settings.atmosphere_height; 
    settings.planet_center = Vec3::new(0.0, -settings.planet_radius, 0.0);
    settings.solar_irradiance = 1361. * 0.02; //solar_irradiance * brightness (where brightness is just a stylistic term to reduce blown out colors)
    let sun_solid_angle = std::f32::consts::TAU * (1. - settings.cos_sun_angular_radius);
    settings.solar_radiance = settings.solar_irradiance / sun_solid_angle;
    settings.mie_beta = 3e-3; //same for all wavelengths
    settings.mie_beta_extinction = 1.11; //constant for converting scattering to extention
    settings.mie_g = 0.76;
    settings.mie_scale_height = 1.2;
    settings.ozone_beta = Vec3::new(3.426, 8.298, 0.356) * 0.06 * 1e-2;
    settings.ozone_profile = Vec3::new(15., 22., 35.); //lower bound, highest concentration, upper bound
}


#[derive(Resource)]
pub struct SunPos(Vec3);

impl Default for SunPos {
    fn default() -> Self {
        SunPos(Vec3::new(0.0, 0.0, -1.0))
    }
}

pub fn update_sun(mut sun: ResMut<SunPos>, mut settings: ResMut<AtmosphereSettings>, time: Res<Time>) {
    // Axis of rotation lives mostly along x, tilted slightly toward y.
    let axis = Vec3::new(1.0, 0.3, 0.0).normalize();
    let angular_speed = 0.1; // radians per second
    let angle = time.elapsed_secs() * angular_speed;

    sun.0 = Quat::from_axis_angle(axis, angle) * Vec3::new(0.0, 0.0, -1.0);
    settings.sun_direction = sun.0;
}