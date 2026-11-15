struct ViewUniform {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    position: vec4<f32>,
    viewport: vec4<f32>,
    near_far: vec4<f32>,
};

@group(0) @binding(0) var<uniform> view: ViewUniform;

// World-space triangle at z = -3, counter-clockwise seen from +z.
@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var p = array<vec3<f32>, 3>(
        vec3<f32>(-1.0, -1.0, -3.0),
        vec3<f32>(1.0, -1.0, -3.0),
        vec3<f32>(0.0, 1.0, -3.0),
    );
    return view.view_proj * vec4<f32>(p[i], 1.0);
}

@fragment
fn fs() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 1.0, 1.0, 1.0); }
