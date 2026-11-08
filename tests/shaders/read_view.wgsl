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
@group(1) @binding(0) var<storage, read_write> out: array<f32>;

@compute @workgroup_size(1)
fn main() {
    out[0] = view.position.x;
    out[1] = view.viewport.x;
    out[2] = view.view_proj[0][0];
    out[3] = view.inv_view[3][0];
}
