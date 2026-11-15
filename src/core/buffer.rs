use crate::core::context::Context;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;



#[derive(Default)]
pub enum BufferAlignment
{
    #[default]
    None,
    Multiple(u64),
    CopyBufferAlignment,
    UniformAlignment,
}



impl BufferAlignment
{
    fn calculate_stride<T>(self, context: &Context) -> u64
    {
        let alignment = self
            .try_into()
            .unwrap_or(context.device.limits().min_uniform_buffer_offset_alignment as _);

        (size_of::<T>() as u64).next_multiple_of(alignment)
    }
}



impl TryInto<u64> for BufferAlignment
{
    type Error = ();

    fn try_into(self) -> Result<u64, Self::Error>
    {
        match self
        {
            BufferAlignment::None => Ok(1),
            BufferAlignment::Multiple(m) => Ok(m),
            BufferAlignment::CopyBufferAlignment => Ok(wgpu::COPY_BUFFER_ALIGNMENT),
            BufferAlignment::UniformAlignment => Err(()),
        }
    }
}



fn buffer_size(size: u64) -> u64
{
    size.max(1).next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT)
}



fn cast_slice_with_padding<T: bytemuck::Pod + bytemuck::NoUninit>(
    data: &[T],
    stride: u64,
) -> Vec<u8>
{
    let stride = stride as usize;
    let mut padded = vec![0; data.len() * stride];
    let data_size = size_of::<T>();

    for (i, d) in data.iter().enumerate()
    {
        let dst = &mut padded[i * stride..i * stride + data_size];
        let bytes = bytemuck::bytes_of(d);
        dst.copy_from_slice(bytes);
    }

    padded
}



#[derive(Debug, Clone)]
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



    pub fn stride(&self) -> u64
    {
        self.stride
    }
}



impl<T: bytemuck::Pod> Buffer<T>
{
    pub fn new<A: Into<BufferAlignment>>(
        context: &Context,
        usage: wgpu::BufferUsages,
        count: u64,
        alignment: A,
    ) -> Self
    {
        let stride = alignment.into().calculate_stride::<T>(context);

        Self::new_with_stride(context, usage, count, stride)
    }



    fn new_with_stride(
        context: &Context,
        usage: wgpu::BufferUsages,
        count: u64,
        stride: u64,
    ) -> Self
    {
        let handle = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer_size(stride * count),
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



    pub fn from_slice<A: Into<BufferAlignment>>(
        context: &Context,
        usage: wgpu::BufferUsages,
        data: &[T],
        alignment: A,
    ) -> Self
    {
        let stride = alignment.into().calculate_stride::<T>(context);

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



#[derive(Debug, Clone)]
pub struct GrowBuffer<T: bytemuck::Pod>
{
    buffer: Buffer<T>,
    count: u64,
    generation: u64,
}



impl<T: bytemuck::Pod> From<Buffer<T>> for GrowBuffer<T>
{
    fn from(buffer: Buffer<T>) -> Self
    {
        Self {
            count: buffer.count(),
            buffer,
            generation: 0,
        }
    }
}



impl<T: bytemuck::Pod> GrowBuffer<T>
{
    pub fn new<A: Into<BufferAlignment>>(
        context: &Context,
        usage: wgpu::BufferUsages,
        initial_capacity: u64,
        alignment: A,
    ) -> Self
    {
        Self {
            buffer: Buffer::new(
                context,
                usage,
                initial_capacity.next_power_of_two(),
                alignment,
            ),
            count: 0,
            generation: 0,
        }
    }



    pub fn from_buffer_with_count(buffer: Buffer<T>, count: u64) -> Self
    {
        Self {
            buffer,
            count,
            generation: 0,
        }
    }



    pub fn write(&mut self, context: &Context, data: &[T]) -> bool
    {
        let resize = data.len() as u64 > self.capacity();

        if resize
        {
            let new_capacity = (data.len() as u64).next_power_of_two();

            self.buffer = Buffer::new_with_stride(
                context,
                self.buffer().usage,
                new_capacity,
                self.buffer.stride(),
            );

            self.generation += 1;
        }

        self.buffer.write(context, 0, data);
        self.count = data.len() as _;

        resize
    }



    pub fn generation(&self) -> u64
    {
        self.generation
    }



    pub fn count(&self) -> u64
    {
        self.count
    }



    pub fn capacity(&self) -> u64
    {
        self.buffer.count()
    }



    pub fn buffer(&self) -> &Buffer<T>
    {
        &self.buffer
    }
}



pub struct PingPong<T: bytemuck::Pod>
{
    a: Buffer<T>,
    b: Buffer<T>,
    a_is_src: bool,
}



impl<T: bytemuck::Pod> PingPong<T>
{
    pub fn new(a: Buffer<T>, b: Buffer<T>) -> Self
    {
        Self {
            a,
            b,
            a_is_src: true,
        }
    }



    pub fn src(&self) -> &Buffer<T>
    {
        if self.a_is_src { &self.a } else { &self.b }
    }



    pub fn dst(&self) -> &Buffer<T>
    {
        if self.a_is_src { &self.b } else { &self.a }
    }



    pub fn flip(&mut self)
    {
        self.a_is_src = !self.a_is_src;
    }
}
