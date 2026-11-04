use crate::core::{
    buffer::Buffer,
    context::Context,
};



pub trait Uniform
{
    fn uniform_size() -> u64;



    fn padded_uniform_size(context: &Context) -> u64;



    fn create_buffer(context: &Context) -> Buffer<Self>
    where
        Self: std::marker::Sized;



    fn create_bind_group(
        context: &Context,
        buffer: &Buffer<Self>,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout)
    where
        Self: std::marker::Sized;
}



#[repr(transparent)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct MatrixUniform<T, const N: usize, const M: usize>
{
    pub data: [[T; N]; M],
}



impl<T, const N: usize, const M: usize> MatrixUniform<T, N, M>
{
    pub const fn uniform_size() -> wgpu::BufferAddress
    {
        std::mem::size_of::<Self>() as wgpu::BufferAddress
    }



    pub fn padded_uniform_size(context: &Context) -> u64
    {
        let min_alignment: u64 = context.device.limits().min_uniform_buffer_offset_alignment as _;
        (Self::uniform_size() + min_alignment - 1) & !(min_alignment - 1) // ceil to alignment
    }
}



impl<T: bytemuck::Pod, const N: usize, const M: usize> MatrixUniform<T, N, M>
{
    pub fn create_buffer(context: &Context) -> Buffer<Self>
    {
        let max_models_per_frame = 64;

        let buffer_len = max_models_per_frame * Self::padded_uniform_size(context);
        Buffer::new(
            context,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            buffer_len as _,
        )
    }



    pub fn create_bind_group(
        context: &Context,
        buffer: &Buffer<Self>,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout)
    {
        let size = wgpu::BufferSize::new(Self::padded_uniform_size(context));

        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: size,
                    },
                    count: None,
                }],
            });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: buffer.handle(),
                        offset: 0,
                        size,
                    }),
                }],
            });

        (bind_group, layout)
    }
}
