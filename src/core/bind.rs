use crate::core::{
    buffer::Buffer,
    context::Context,
};



// BindGroupBuilder, Uniform<T>, auto-layout helpers



#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VertexLayoutDesc
{
    pub array_stride: u64,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}



impl VertexLayoutDesc
{
    pub fn to_vertex_layout(&'_ self) -> wgpu::VertexBufferLayout<'_>
    {
        wgpu::VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes: &self.attributes,
        }
    }
}



#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LayoutId(u64);



pub struct BindGroupBuilder<'a>
{
    entries: Vec<wgpu::BindGroupEntry<'a>>,
}



impl<'a> BindGroupBuilder<'a>
{
    pub fn new() -> Self
    {
        Self { entries: vec![] }
    }


    pub fn add_entry<T>(mut self, buffer: &'a Buffer<T>) -> Self
    {
        self.entries.push(wgpu::BindGroupEntry {
            binding: self.entries.len() as _,
            resource: buffer.handle().as_entire_binding(),
        });

        self
    }



    pub fn build_compute(
        self,
        context: &Context,
        pipeline: &wgpu::ComputePipeline,
    ) -> wgpu::BindGroup
    {
        context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &self.entries,
            })
    }
}



impl<'a> Default for BindGroupBuilder<'a>
{
    fn default() -> Self
    {
        Self::new()
    }
}
