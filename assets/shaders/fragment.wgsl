struct Uniform {
    resolution: vec2f,
    world_from_clip: mat4x4f,
}

struct AtmosphereSettings {
    atmosphere_height: f32,
    num_rayleigh_steps: i32,
    num_optical_depth_steps: i32,
    planet_radius: f32,
    planet_color: vec3f,
    scale_height: f32,
    rayleigh_beta: vec3f,
    sun_direction: vec3f,
    sun_angular_diameter: f32,
    atmosphere_radius: f32,
    planet_center: vec3f,
    sun_intensity: f32
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
        return vec4f(0.0, 0.0, 0.0, 1.0);
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
    }

    let phase = rayleigh_phase(dot(ray_dir, -info.sun_direction));
    var inscattered_light = calculate_light(start_point, ray_dir, dist_through_atmosphere);
    inscattered_light *= info.rayleigh_beta * phase * info.sun_intensity;

    let view_ray_optical_depth = optical_depth(start_point, ray_dir, dist_through_atmosphere);
    let total_view_ray_transmittance = exp(-info.rayleigh_beta * view_ray_optical_depth);

    let final_color = background_color * total_view_ray_transmittance + inscattered_light;

    return vec4f(tonemap(final_color), 1.0);
}


fn calculate_light(ray_origin: vec3f, ray_dir: vec3f, dist_through_atmosphere: f32) -> vec3f {
    let step_size = dist_through_atmosphere / f32(info.num_rayleigh_steps);
    var scatter_point = ray_origin;
    var inscattered_light = vec3f(0);
    var view_ray_optical_depth = 0.0;

    //try to replace with integral
    for (var i = 0; i < info.num_rayleigh_steps; i += 1) {
        //if intersects planet, skip
        //if goes out into space, use infinite integration approximation (eventually)
        let sun_blocked = ray_sphere(scatter_point,info.sun_direction,info.planet_center,info.planet_radius).x > 0;
        let local_density = rayleigh_density(scatter_point);
        if !sun_blocked {
            let sun_ray_length = ray_sphere(scatter_point, info.sun_direction, info.planet_center, info.atmosphere_radius).y;
            let sun_ray_optical_depth = optical_depth(scatter_point, info.sun_direction, sun_ray_length);
            let transmittance = exp(-info.rayleigh_beta * (sun_ray_optical_depth + view_ray_optical_depth));
            inscattered_light += local_density * transmittance * step_size;
        }

        view_ray_optical_depth += local_density * step_size;
        scatter_point += ray_dir * step_size;
    }

    return inscattered_light;
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

//calculates the optical depth along a ray, essentially the average density across the ray
fn optical_depth(origin: vec3f, dir: vec3f, ray_length: f32) -> f32 {
    var sample_point = origin;
    let step_size = ray_length / f32(info.num_optical_depth_steps);
    var optical_depth = 0.0;

    //this can also likely be replaced with an integral
    for (var i = 0; i < info.num_optical_depth_steps; i += 1) {
        optical_depth += rayleigh_density(sample_point) * step_size;
        sample_point += dir * step_size;
    }

    return optical_depth;
}

//calculates the density of the atmosphere at a given point for use in rayleigh scattering.
fn rayleigh_density(point: vec3f) -> f32 {
    let height = length(point - info.planet_center) - info.planet_radius;
    return exp(-max(0.0, height) / info.scale_height);
}

fn rayleigh_phase(cos_theta: f32) -> f32 {
    return (3.0 / (16.0 * 3.14159)) * (1.0 + cos_theta * cos_theta);
}

fn tonemap(color: vec3f) -> vec3f {
    return vec3f(1) - exp(-color);
}