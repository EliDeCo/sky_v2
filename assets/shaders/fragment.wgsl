const PI = 3.14159265359;

struct Uniform {
    resolution: vec2f,
    world_from_clip: mat4x4f,
}

struct AtmosphereSettings {
    atmosphere_height: f32,
    num_view_ray_steps: i32,
    num_sun_ray_steps: i32,
    planet_radius: f32,
    planet_color: vec3f,
    rayleigh_scale_height: f32,
    rayleigh_beta: vec3f,
    sun_direction: vec3f,
    cos_sun_angular_radius: f32,
    atmosphere_radius: f32,
    planet_center: vec3f,
    solar_irradiance: f32,
    solar_radiance: f32,
    mie_beta: f32,
    mie_beta_extinction: f32,
    mie_g: f32,
    mie_scale_height: f32,
    ozone_beta: vec3f,
    ozone_profile: vec3f,
    gauss_legendre_params: array<vec4f, 16>
}

@group(0) @binding(0) 
var<uniform> unif: Uniform;

@group(1) @binding(0)
var<uniform> info: AtmosphereSettings;

@fragment
fn frag_main(@builtin(position) frag_coords: vec4<f32>) -> @location(0) vec4<f32> {

    //Ray setup
    let uv = frag_coords.xy / unif.resolution;

    let ndc = vec2f(
        uv.x * 2.0 - 1.0,
        1.0 - uv.y * 2.0
    );

    let clip_near = vec4f(ndc, -1.0, 1.0);
    let clip_far  = vec4f(ndc,  1.0, 1.0);

    let world_near = unif.world_from_clip * clip_near;
    let world_far  = unif.world_from_clip * clip_far;

    let ray_origin = world_near.xyz / world_near.w;
    let far = world_far.xyz / world_far.w;

    let ray_dir = normalize(far - ray_origin);

    let planet = ray_sphere(ray_origin, ray_dir, info.planet_center, info.planet_radius);
   
   
   
    //sky
    let atmosphere_intersections = ray_sphere(ray_origin, ray_dir, info.planet_center, info.atmosphere_radius);
    // atmosphere is missed entirely, render space
    if all(atmosphere_intersections == vec2f(-1.0, -1.0)) {
        return vec4f(tonemap(vec3f(sun_disk(ray_dir))), 1.0);
    }

    var start_point = ray_origin;
    let outside_atmosphere = length(ray_origin - info.planet_center) > info.atmosphere_radius;
    //if ray starts outside atmosphere, move start point to entry point
    if outside_atmosphere {
        start_point = ray_origin + ray_dir * atmosphere_intersections.x;
    }
    var dist_through_atmosphere = atmosphere_intersections.y - max(atmosphere_intersections.x, 0.0);
    var background_color = vec3f(0);
    let planet_intersections = ray_sphere(ray_origin, ray_dir, info.planet_center, info.planet_radius);
    let hits_planet = planet_intersections.x > 0.0;
    if hits_planet {
        if outside_atmosphere { //if the ray starts outside the atmosphere and hits the planet, the distance through the atmosphere is the difference between the distances to the atmosphere and the planet surface
            dist_through_atmosphere = planet_intersections.x - atmosphere_intersections.x;
        } else { // if the ray starts inside the atmosphere, we need to check if it hits the planet before exiting the atmosphere
            dist_through_atmosphere = min(dist_through_atmosphere, planet_intersections.x);

        }
        let hit_point = ray_origin + ray_dir * planet_intersections.x;
        let normal    = normalize(hit_point - info.planet_center);
        let diffuse   = max(dot(normal, info.sun_direction), 0.0);
        background_color = info.planet_color * (0.1 + diffuse);  
    } else {
        background_color = vec3f(sun_disk(ray_dir));
    }

    let final_color = calculate_light(start_point, ray_dir, dist_through_atmosphere, background_color);

    return vec4f(tonemap(final_color), 1.0);
}

//returns the final color of the ray
fn calculate_light(ray_origin: vec3f, ray_dir: vec3f, dist_through_atmosphere: f32, background_color: vec3f) -> vec3f  {
    let step_size = dist_through_atmosphere / f32(info.num_view_ray_steps);
    var current_point = ray_origin;
    var view_optical_depth_rayleigh = 0.0;
    var view_optical_depth_ozone = 0.0;
    var view_optical_depth_mie = 0.0;
    var rayleigh_transmittance = vec3f(0);
    var mie_transmittance = vec3f(0);

    for (var i = 0; i < info.num_view_ray_steps; i += 1) {
        let height = length(current_point - info.planet_center) - info.planet_radius;

        let local_rayleigh_density = rayleigh_density(height);
        let local_mie_density = mie_density(height);
        let local_ozone_density = ozone_density(height);

        view_optical_depth_rayleigh += local_rayleigh_density * step_size;
        view_optical_depth_mie += local_mie_density * step_size;
        view_optical_depth_ozone += local_ozone_density * step_size;

        let sun_blocked = ray_sphere(current_point,info.sun_direction,info.planet_center,info.planet_radius).x > 0;

        if !sun_blocked {
            let sun_ray_length = ray_sphere(current_point, info.sun_direction, info.planet_center, info.atmosphere_radius).y;
            let sun_ray_optical_depth = optical_depth_gauss_legendre(current_point, info.sun_direction, sun_ray_length);

            let tau = info.rayleigh_beta * (view_optical_depth_rayleigh + sun_ray_optical_depth.x)
                + info.mie_beta_extinction * vec3f(info.mie_beta) * (view_optical_depth_mie + sun_ray_optical_depth.y)
                + info.ozone_beta * (view_optical_depth_ozone + sun_ray_optical_depth.z);
            
            let local_tansmittance = exp(-tau);

            rayleigh_transmittance += local_rayleigh_density * local_tansmittance * step_size;
            mie_transmittance += local_mie_density * local_tansmittance * step_size;
        }
        current_point += ray_dir * step_size;
    }

    let view_cos = dot(ray_dir,info.sun_direction);

    let scattering = info.solar_irradiance * (
        rayleigh_phase(view_cos) * info.rayleigh_beta * rayleigh_transmittance +
        mie_phase(view_cos) * info.mie_beta * mie_transmittance
    );

    let total_tau = info.rayleigh_beta * view_optical_depth_rayleigh
        + info.mie_beta_extinction * vec3f(info.mie_beta) * view_optical_depth_mie
        + info.ozone_beta * view_optical_depth_ozone;

    return background_color * exp(-total_tau) + scattering;
}
//returns a vec3f containing the rayleigh, mie, and ozone optical depth of the given ray
fn optical_depth(origin: vec3f, dir: vec3f, ray_length: f32) -> vec3f {
    var current_point = origin;
    let step_size = ray_length / f32(info.num_sun_ray_steps);
    let step = dir * step_size;
    var view_optical_depth_rayleigh = 0.0;
    var view_optical_depth_ozone = 0.0;
    var view_optical_depth_mie = 0.0;

    for (var i = 0; i < info.num_sun_ray_steps; i += 1) {
        let height = length(current_point - info.planet_center) - info.planet_radius;

        view_optical_depth_rayleigh += rayleigh_density(height);
        view_optical_depth_mie += mie_density(height);
        view_optical_depth_ozone += ozone_density(height);

        current_point += step;
    }

    return step_size * vec3f(view_optical_depth_rayleigh,view_optical_depth_mie,view_optical_depth_ozone);
}

fn optical_depth_gauss_legendre(origin: vec3f, dir: vec3f, ray_length: f32) -> vec3f {
    // Ozone: unchanged naive march
    var current_point = origin;
    let step_size = ray_length / f32(info.num_sun_ray_steps);
    let step = dir * step_size;
    var ozone_od = 0.0;
    for (var i = 0; i < info.num_sun_ray_steps; i += 1) {
        ozone_od += ozone_density(length(current_point - info.planet_center) - info.planet_radius);
        current_point += step;
    }

    let w = origin - info.planet_center;

    // Rayleigh
    let h_r   = info.rayleigh_scale_height;
    let bh_r  = 2.0 * dot(w, dir) / h_r;
    let c_r   = dot(w, w) / (h_r * h_r);
    let rh0_r = info.planet_radius / h_r;
    let L_r   = max(-0.5 * bh_r, 1.0);
    let ll_r  = log(2.0 * L_r);

    var ray_od = 0.0;
    for (var i = 0u; i < 16u; i += 1u) {
        let p = info.gauss_legendre_params[i];

        let x1  = L_r * (1.0 + p.x) / (1.0 - p.x);
        ray_od += p.y * exp(ll_r - 2.0 * log(1.0 - p.x) + rh0_r - sqrt(x1*x1 + bh_r*x1 + c_r));

        let x2  = L_r * (1.0 + p.z) / (1.0 - p.z);
        ray_od += p.w * exp(ll_r - 2.0 * log(1.0 - p.z) + rh0_r - sqrt(x2*x2 + bh_r*x2 + c_r));
    }
    ray_od *= h_r;

    // Mie
    let h_m   = info.mie_scale_height;
    let bh_m  = 2.0 * dot(w, dir) / h_m;
    let c_m   = dot(w, w) / (h_m * h_m);
    let rh0_m = info.planet_radius / h_m;
    let L_m   = max(-0.5 * bh_m, 1.0);
    let ll_m  = log(2.0 * L_m);

    var mie_od = 0.0;
    for (var i = 0u; i < 16u; i += 1u) {
        let p = info.gauss_legendre_params[i];

        let x1   = L_m * (1.0 + p.x) / (1.0 - p.x);
        mie_od  += p.y * exp(ll_m - 2.0 * log(1.0 - p.x) + rh0_m - sqrt(x1*x1 + bh_m*x1 + c_m));

        let x2   = L_m * (1.0 + p.z) / (1.0 - p.z);
        mie_od  += p.w * exp(ll_m - 2.0 * log(1.0 - p.z) + rh0_m - sqrt(x2*x2 + bh_m*x2 + c_m));
    }
    mie_od *= h_m;

    return vec3f(ray_od, mie_od, step_size * ozone_od);
}

// Returns vec2(t_enter, t_exit). On miss, both components are -1.0.
fn ray_sphere(origin: vec3f, dir: vec3f, center: vec3f, radius: f32) -> vec2f {
    let oc = origin - center;
    let b  = dot(oc, dir);
    let qc = oc - b * dir;
    var h  = radius * radius - dot(qc, qc);
    if h < 0.0 {
        return vec2f(-1.0, -1.0);
    }
    h = sqrt(h);
    return vec2f(-b - h, -b + h);
}

//calculates the density of the atmosphere at a given point for use in rayleigh scattering.
fn rayleigh_density(height: f32) -> f32 {
    return exp(-max(0.0, height) / info.rayleigh_scale_height);
}

fn rayleigh_phase(cos_theta: f32) -> f32 {
    return (3.0 / (16.0 * PI)) * (1.0 + cos_theta * cos_theta);
}

fn mie_density(height: f32) -> f32 {
    return exp(-max(0.0, height) / info.mie_scale_height);
}

fn mie_phase(cos_theta: f32) -> f32 {
    let denom = 1 + info.mie_g * info.mie_g - 2 * info.mie_g * cos_theta;
    return (1 - info.mie_g *  info.mie_g) / (4 * PI * pow(denom, 1.5));
}

fn ozone_density(height: f32) -> f32 {
    let lower = info.ozone_profile.x;
    let midpoint = info.ozone_profile.y;
    let upper = info.ozone_profile.z;

    if height < lower || height > upper {
        return 0;
    } else if height < midpoint {
        return (height - lower) / (midpoint - lower);
    } else {
        return (upper - height) / (upper - midpoint);
    }
}

fn tonemap(color: vec3f) -> vec3f {
    return vec3f(1) - exp(-color);
}

fn sun_disk(ray_dir: vec3f) -> f32 {
    let cos_theta = dot(ray_dir, info.sun_direction);

    let edge = fwidth(cos_theta);
    let disk = smoothstep(info.cos_sun_angular_radius - edge, info.cos_sun_angular_radius + edge, cos_theta);

    return disk * info.solar_radiance;
}