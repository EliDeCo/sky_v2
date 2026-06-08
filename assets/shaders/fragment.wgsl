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

    //ground
    let ground_normal = vec3f(0.0, 1.0, 0.0);
    if dot(direction, ground_normal) < -0.00001 {
        let t = -origin.y / direction.y;
        let hit_pos = origin + direction * t;

        let green = sin(hit_pos.x * 5) * cos(hit_pos.z * 5) * 0.1 + 0.5;

        return vec4f(0.1, green, 0.1, 1.0);
    }

    return vec4f(0.1, 0.1, 5.0, 1.0);


}