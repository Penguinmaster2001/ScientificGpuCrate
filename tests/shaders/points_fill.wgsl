struct Point {
    pos: vec4<f32>,
    data: vec4<f32>,
    color: vec4<f32>,
};




@group(0) @binding(0) var<storage, read_write> points: array<Point>;




@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) id: vec3<u32>)
{
    if (id.x >= arrayLength(&points))
    {
        return;
    }
    
    points[id.x] = Point(
        vec4<f32>(f32(id.x) * 2.0 - 2.0, 0.0, -5.0, 1.0),
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(1.0, 1.0, 1.0, 1.0),
    );
}
