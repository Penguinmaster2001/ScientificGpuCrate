use crate::core::{
    buffer::Buffer,
    context::Context,
};



// BindGroupBuilder, Uniform<T>, auto-layout helpers



#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VertexLayoutDesc;



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
