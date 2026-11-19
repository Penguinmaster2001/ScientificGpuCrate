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



struct Line {
    @location(0) pos: vec4<f32>,
    @location(1) start_color: vec4<f32>,
    @location(2) end_color: vec4<f32>,
    @location(3) data: vec4<f32>,
};



struct PointOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) start_color: vec4<f32>,
    @location(3) end_color: vec4<f32>,
    @location(4) data: vec4<f32>,
}




@group(0) @binding(0) var<uniform> view: ViewUniform;



@vertex
fn vs(@builtin(vertex_index) i: u32, line: Line) -> PointOutput
{
    const quad_offsets = array<vec4<f32>, 6>(
        vec4<f32>(0.0, -0.5, 0.0, 0.0),
        vec4<f32>(0.0, -0.5, 0.0, 0.0),
        vec4<f32>(0.0,  0.5, 0.0, 0.0),
        
        vec4<f32>(0.0,  0.5, 0.0, 0.0),
        vec4<f32>(0.0, -0.5, 0.0, 0.0),
        vec4<f32>(0.0,  0.5, 0.0, 0.0),
    );

    const quad_uv = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );

    var start = vec4<f32>(line.pos.xy, 0.0, 1.0);
    var end = vec4<f32>(line.pos.zw, 0.0, 1.0);

    if start.x > end.x
    {
        let temp = start;
        start = end;
        end = start;
    }

    var out: PointOutput;

    var pos = vec4<f32>();

    if i == 0 || i == 3 || i == 4
    {
        pos = start + (line.data.x * quad_offsets[i]);
    }
    else
    {
        pos = end + (line.data.y * quad_offsets[i]);
    }

    out.clip_position = view.view_proj * pos;
    out.uv = quad_uv[i];
    out.world_position = pos;
    out.data = line.data;
    out.start_color = line.start_color;
    out.end_color = line.end_color;

    return out;
}



@fragment
fn fs(in: PointOutput) -> @location(0) vec4<f32>
{
    return mix(in.start_color, in.end_color, in.uv.x);

    // var centered_uv = 2.0 * (in.uv - vec2<f32>(0.5));
    // let dist = dot(centered_uv, centered_uv);

    // if dist <= 1.0
    // {
    //     return in.color;
    // }

    // discard;
}
