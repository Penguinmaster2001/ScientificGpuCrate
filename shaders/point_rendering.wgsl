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



struct Point {
    @location(0) pos: vec4<f32>,
    @location(1) data: vec4<f32>,
    @location(2) color: vec4<f32>,
};



struct PointOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) data: vec4<f32>,
    @location(3) color: vec4<f32>,
}




@group(0) @binding(0) var<uniform> view: ViewUniform;



@vertex
fn vs(@builtin(vertex_index) i: u32, point: Point) -> PointOutput
{
    const quad_pos = array<vec4<f32>, 6>(
        vec4<f32>(-0.5, -0.5, 0.0, 1.0),
        vec4<f32>( 0.5, -0.5, 0.0, 1.0),
        vec4<f32>( 0.5,  0.5, 0.0, 1.0),
        
        vec4<f32>(-0.5,  0.5, 0.0, 1.0),
        vec4<f32>(-0.5, -0.5, 0.0, 1.0),
        vec4<f32>( 0.5,  0.5, 0.0, 1.0),
    );

    const quad_uv = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );

    var out: PointOutput;

    let view_right = view.inv_view[0];
    let view_up = view.inv_view[1];

    let pos = vec4<f32>((quad_pos[i].x * view_right + quad_pos[i].y * view_up).xyz, 1.0);

    out.clip_position = view.view_proj * (point.pos + (point.data[0] * pos));
    out.uv = quad_uv[i];
    out.world_position = point.pos;
    out.data = point.data;
    out.color = point.color;

    return out;
}



@fragment
fn fs(in: PointOutput) -> @location(0) vec4<f32>
{
    var centered_uv = 2.0 * (in.uv - vec2<f32>(0.5));
    let dist = dot(centered_uv, centered_uv);

    if dist <= 1.0
    {
        return in.color;
    }

    discard;
}
