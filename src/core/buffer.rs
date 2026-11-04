use crate::core::context::Context;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;



#[derive(Debug)]
pub struct Buffer<T>
{
    handle: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    len: usize,
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
        (self.len * size_of::<T>()) as _
    }



    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize
    {
        self.len
    }



    pub fn cast<U: bytemuck::Pod>(self) -> Buffer<U>
    {
        debug_assert!(self.size_bytes().is_multiple_of(size_of::<U>() as _));
        let len = (self.size_bytes() as usize) / size_of::<U>();
        Buffer::<U> {
            handle: self.handle,
            usage: self.usage,
            len,
            _phantom: PhantomData,
        }
    }
}



impl<T: bytemuck::Pod> Buffer<T>
{
    pub fn new(context: &Context, usage: wgpu::BufferUsages, len: usize) -> Self
    {
        let handle = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (size_of::<T>() * len) as _,
            usage,
            mapped_at_creation: false,
        });

        Self {
            handle,
            usage,
            len,
            _phantom: PhantomData,
        }
    }



    pub fn from_slice(context: &Context, usage: wgpu::BufferUsages, data: &[T]) -> Self
    {
        let contents = bytemuck::cast_slice(data);
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
            len: data.len(),
            _phantom: PhantomData,
        }
    }



    pub fn write(&self, context: &Context, offset: usize, data: &[T])
    {
        debug_assert!(offset + data.len() <= self.len);
        context.queue.write_buffer(
            &self.handle,
            (offset * size_of::<T>()) as _,
            bytemuck::cast_slice(data),
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
