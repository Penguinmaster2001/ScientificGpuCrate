use crate::core::{
    buffer::Buffer,
    context::Context,
};



pub trait Uniform
{
    fn uniform_size() -> u64
    where
        Self: std::marker::Sized,
    {
        std::mem::size_of::<Self>() as wgpu::BufferAddress
    }



    fn padded_uniform_size(context: &Context) -> u64
    where
        Self: std::marker::Sized,
    {
        let min_alignment: u64 = context.device.limits().min_uniform_buffer_offset_alignment as _;
        (Self::uniform_size() + min_alignment - 1) & !(min_alignment - 1) // ceil to alignment
    }
}



#[repr(transparent)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct MatrixUniform<T, const N: usize, const M: usize>
{
    pub data: [[T; N]; M],
}



impl<T, const N: usize, const M: usize> MatrixUniform<T, N, M>
{
    pub fn create_bind_group(
        context: &Context,
        buffer: &Buffer<Self>,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout)
    where
        Self: std::marker::Sized,
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



impl<T, const N: usize, const M: usize> Uniform for MatrixUniform<T, N, M> {}



impl Uniform for u8 {}
impl Uniform for u16 {}
impl Uniform for u32 {}
impl Uniform for u64 {}
impl Uniform for u128 {}
impl Uniform for usize {}
impl Uniform for f32 {}
impl Uniform for f64 {}
impl<const N: usize> Uniform for [u8; N] {}
impl<const N: usize> Uniform for [u16; N] {}
impl<const N: usize> Uniform for [u32; N] {}
impl<const N: usize> Uniform for [u64; N] {}
impl<const N: usize> Uniform for [u128; N] {}
impl<const N: usize> Uniform for [usize; N] {}
impl<const N: usize> Uniform for [f32; N] {}
impl<const N: usize> Uniform for [f64; N] {}
