@fragment
fn frag_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4(0.5,0.2,0.1, 1.0);
}