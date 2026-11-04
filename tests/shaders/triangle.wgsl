@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var p = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0));
    return vec4<f32>(p[i], 0.5, 1.0);
}
@fragment fn fs() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
@fragment fn fs_half() -> @location(0) vec4<f32> { return vec4<f32>(0.5, 0.5, 0.5, 1.0); }
