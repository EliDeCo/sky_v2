struct Uniform {
    resolution: vec2f,
    world_from_clip: mat4x4f,
}

@group(0) @binding(0) 
var<uniform> unif: Uniform;

@fragment
fn frag_main(@builtin(position) frag_coords: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = frag_coords.xy / unif.resolution;

    //normalized device coordinates (flip y)
    let ndc = vec2f(
        uv.x * 2.0 - 1.0,
        1.0 - uv.y * 2.0
    );

    let clip_near = vec4f(ndc, -1.0, 1.0);
    let clip_far  = vec4f(ndc,  1.0, 1.0);

    let world_near = unif.world_from_clip * clip_near;
    let world_far  = unif.world_from_clip * clip_far;

    let origin = world_near.xyz / world_near.w;
    let far = world_far.xyz / world_far.w;

    let direction = normalize(far - origin);

    let light_dir = normalize(vec3f(1.0, 2.0, 1.0));
    let sky_color   = vec3f(0.529, 0.808, 0.922);
    let grass_color = vec3f(0.196, 0.459, 0.145);
    let ambient     = 0.1;

    //ground
    let sphere_center = vec3f(0.0, -6378000.0, 0.0);
    let sphere_radius = 6378000.0;

    let t = ray_sphere_intersect(origin, direction, sphere_center, sphere_radius);
    if t > 0.0 {
        let hit_point = origin + direction * t;
        let normal    = normalize(hit_point - sphere_center);
        let diffuse   = max(dot(normal, light_dir), 0.0);
        let lit       = grass_color * (ambient + diffuse);
        return vec4f(lit, 1.0);
    }
        
    //sky
    return vec4f(sky_color, 1.0);

}

// Returns the distance to the nearest positive hit, or -1.0 on miss.
fn ray_sphere_intersect(origin: vec3f, dir: vec3f, center: vec3f, radius: f32) -> f32 {
    let oc = origin - center;
    let a  = dot(dir, dir);
    let b  = 2.0 * dot(oc, dir);
    let c  = dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return -1.0;
    }
    let sqrt_d = sqrt(discriminant);
    let t0 = (-b - sqrt_d) / (2.0 * a);
    let t1 = (-b + sqrt_d) / (2.0 * a);
    if t0 > 0.0 { return t0; }
    if t1 > 0.0 { return t1; }
    return -1.0;
}