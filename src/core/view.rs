use crate::core::{
    buffer::Buffer,
    context::Context,
};



pub struct View
{
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub color_mask: wgpu::ColorWrites,
    pub clear_color: bool, // true for the first view only
}



pub struct ViewUniform
{
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}



impl Default for ViewUniform
{
    fn default() -> Self
    {
        Self {
            view: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}



impl ViewUniform
{
    pub fn new<R: Into<[u32; 2]>>(view: &View, _resolution: R) -> Self
    {
        Self {
            view: view.view,
            proj: view.proj,
        }
    }



    // pub fn create_camera_buffer(&self, renderer: &Renderer) -> wgpu::Buffer
    // {
    //     renderer
    //         .device
    //         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some("Camera Buffer"),
    //             contents: bytemuck::cast_slice(&[*self]),
    //             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    //         })
    // }



    // pub fn create_camera_bind_group(
    //     camera_buffer: &Buffer,
    //     renderer: &Renderer,
    // ) -> (wgpu::BindGroup, wgpu::BindGroupLayout)
    // {
    //     let camera_bind_group_layout =
    //         renderer
    //             .device
    //             .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //                 entries: &[wgpu::BindGroupLayoutEntry {
    //                     binding: 0,
    //                     visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
    //                     ty: wgpu::BindingType::Buffer {
    //                         ty: wgpu::BufferBindingType::Uniform,
    //                         has_dynamic_offset: false,
    //                         min_binding_size: None,
    //                     },
    //                     count: None,
    //                 }],
    //                 label: Some("camera_bind_group_layout"),
    //             });

    //     let camera_bind_group = renderer
    //         .device
    //         .create_bind_group(&wgpu::BindGroupDescriptor {
    //             layout: &camera_bind_group_layout,
    //             entries: &[wgpu::BindGroupEntry {
    //                 binding: 0,
    //                 resource: camera_buffer.as_entire_binding(),
    //             }],
    //             label: Some("camera_bind_group"),
    //         });

    //     (camera_bind_group, camera_bind_group_layout)
    // }
}



pub struct ViewBuffer
{
    buffer: Buffer<ViewUniform>,
    stride: u64,
}



impl ViewBuffer
{
    pub fn upload(&mut self, _context: &Context, _views: &[ViewUniform])
    {
        todo!()
    }



    pub fn offset(&self, i: usize) -> u32
    {
        (i as u64 * self.stride) as u32
    }



    pub fn bind_group(&self) -> &wgpu::BindGroup
    {
        todo!()
    }



    pub fn layout(&self) -> &wgpu::BindGroupLayout
    {
        todo!()
    }
}
