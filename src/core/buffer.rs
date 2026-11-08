use crate::core::{
    context::Context,
    uniform::Uniform,
};
use std::marker::PhantomData;
use wgpu::util::DeviceExt;



fn cast_slice_with_padding<T: bytemuck::Pod + bytemuck::NoUninit>(
    data: &[T],
    stride: u64,
) -> Vec<u8>
{
    let mut padded = vec![0; data.len() * stride as usize];
    let data_size = size_of::<T>();

    for (i, d) in data.iter().enumerate()
    {
        let dst = &mut padded[i * stride as usize..i * stride as usize + data_size];
        let bytes = bytemuck::bytes_of(d);
        dst.copy_from_slice(bytes);
    }

    padded
}



#[derive(Debug)]
pub struct Buffer<T>
{
    handle: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    count: u64,
    stride: u64,
    _phantom: PhantomData<T>,
}



impl<T> Buffer<T>
{
    pub fn handle(&self) -> &wgpu::Buffer
    {
        &self.handle
    }



    pub fn size_bytes(&self) -> u64
    {
        self.count * self.stride
    }



    pub fn count(&self) -> u64
    {
        self.count
    }



    pub fn cast<U: bytemuck::Pod>(self) -> Buffer<U>
    {
        debug_assert!(self.size_bytes().is_multiple_of(size_of::<U>() as _));
        let len = self.size_bytes() / size_of::<U>() as u64;
        Buffer::<U> {
            handle: self.handle,
            usage: self.usage,
            count: len,
            stride: self.stride,
            _phantom: PhantomData,
        }
    }



    pub fn stride(&self) -> u64
    {
        self.stride
    }
}



impl<T: bytemuck::Pod + Uniform> Buffer<T>
{
    pub fn new(
        context: &Context,
        usage: wgpu::BufferUsages,
        count: u64,
        dynamic_offset: bool,
    ) -> Self
    {
        let stride = if dynamic_offset
        {
            T::padded_uniform_size(context)
        }
        else
        {
            size_of::<T>() as _
        };

        let handle = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (stride * count) as _,
            usage,
            mapped_at_creation: false,
        });

        Self {
            handle,
            usage,
            count,
            stride,
            _phantom: PhantomData,
        }
    }



    pub fn from_slice(
        context: &Context,
        usage: wgpu::BufferUsages,
        data: &[T],
        dynamic_offset: bool,
    ) -> Self
    {
        let stride = if dynamic_offset
        {
            T::padded_uniform_size(context)
        }
        else
        {
            size_of::<T>() as _
        };

        let contents = &cast_slice_with_padding(data, stride);
        let handle = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage,
            });

        Self {
            handle,
            usage,
            count: data.len() as _,
            stride,
            _phantom: PhantomData,
        }
    }



    pub fn write(&self, context: &Context, offset: u64, data: &[T])
    {
        debug_assert!(offset + data.len() as u64 <= self.count);
        context.queue.write_buffer(
            &self.handle,
            offset * self.stride,
            bytemuck::cast_slice(&cast_slice_with_padding(data, self.stride)),
        );
    }
}



#[expect(unused)]
pub struct GrowBuffer<T: bytemuck::Pod>
{
    buffer: Buffer<T>,
    generation: u64,
}



impl<T: bytemuck::Pod> GrowBuffer<T>
{
    pub fn generation(&self) -> u64
    {
        self.generation
    }
}



#[expect(unused)]
pub struct PingPong<T: bytemuck::Pod>
{
    a: Buffer<T>,
    b: Buffer<T>,
}
