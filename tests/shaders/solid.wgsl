// Fullscreen triangle at NDC depth 0.5. No vertex buffer.
@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(p[i], 0.5, 1.0);
}

@fragment
fn fs() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 1.0, 1.0, 1.0); }

@fragment
fn fs_half() -> @location(0) vec4<f32> { return vec4<f32>(0.5, 0.5, 0.5, 1.0); }

// Pixel (x, y) becomes [x, y, 0, 255] for x, y < 256 (Rgba8Unorm).
@fragment
fn fs_coords(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(floor(p.x) / 255.0, floor(p.y) / 255.0, 0.0, 1.0);
}

struct Mrt {
    @location(0) a: vec4<f32>,
    @location(1) b: vec4<f32>,
};

@fragment
fn fs_mrt() -> Mrt {
    var o: Mrt;
    o.a = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    o.b = vec4<f32>(0.0, 1.0, 0.0, 1.0);
    return o;
}
