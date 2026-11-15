use crate::core::{
    bind::VertexLayoutDesc,
    buffer::{
        Buffer,
        BufferAlignment,
        GrowBuffer,
    },
    context::Context,
    pass::{
        FrameContext,
        Pass,
        PassOps,
        begin_render_pass,
    },
    pipeline::{
        DepthDesc,
        RenderPipelineDesc,
    },
    shader::ShaderLib,
};



#[derive(Clone, Copy)]
pub struct Point
{
    pub pos: glam::Vec3,
    pub data: [f32; 4],
    pub color: [f32; 4],
}



#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy)]
pub struct PointUniform
{
    pub pos: [f32; 4],
    pub data: [f32; 4],
    pub color: [f32; 4],
} // 48 bytes



impl PointUniform
{
    pub fn new(point: &Point) -> Self
    {
        Self {
            pos: point.pos.extend(1.0).to_array(),
            data: point.data,
            color: point.color,
        }
    }



    pub fn vertex_desc() -> VertexLayoutDesc
    {
        VertexLayoutDesc {
            array_stride: size_of::<PointUniform>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: vec![
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}



pub enum PointBlend
{
    Opaque,
    Alpha,
    Additive,
}



impl From<PointBlend> for wgpu::BlendState
{
    fn from(val: PointBlend) -> Self
    {
        match val
        {
            PointBlend::Opaque => wgpu::BlendState::REPLACE,
            PointBlend::Alpha => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            PointBlend::Additive => wgpu::BlendState::ADDITIVE,
        }
    }
}



pub struct PointPassConfig
{
    pub color_format: wgpu::TextureFormat,
    pub blend: PointBlend,
    pub depth: bool,
    pub capacity: usize,
}



impl PointPassConfig
{
    pub fn new(color_format: wgpu::TextureFormat) -> Self
    {
        Self {
            color_format,
            blend: PointBlend::Additive,
            depth: true,
            capacity: 0,
        }
    }



    pub fn blend(mut self, blend: PointBlend) -> Self
    {
        self.blend = blend;

        self
    }



    pub fn depth(mut self, depth: bool) -> Self
    {
        self.depth = depth;

        self
    }
}



#[derive(Debug, Clone)]
pub struct PointPass
{
    point_buffer: GrowBuffer<PointUniform>,
    desc: RenderPipelineDesc,
}



impl PointPass
{
    pub fn new(context: &Context, shaders: &mut ShaderLib, config: PointPassConfig) -> Self
    {
        let shader =
            shaders.load_shader_source(context, include_str!("../../shaders/point_rendering.wgsl"));

        let mut desc = RenderPipelineDesc::new(shader.id(), "vs", Some("fs"))
            .color_target(config.color_format, Some(config.blend.into()))
            .depth(
                if config.depth
                {
                    DepthDesc::reversed_z()
                }
                else
                {
                    DepthDesc::always()
                },
            )
            .vertex(0, PointUniform::vertex_desc());
        desc.cull = None;

        Self {
            point_buffer: GrowBuffer::new(
                context,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                config.capacity as _,
                BufferAlignment::None,
            ),
            desc,
        }
    }



    pub fn set_points(&mut self, context: &Context, points: &[Point])
    {
        self.point_buffer.write(
            context,
            &points.iter().map(PointUniform::new).collect::<Vec<_>>(),
        );
    }



    pub fn use_buffer<B: Into<GrowBuffer<PointUniform>>>(&mut self, buffer: B)
    {
        self.point_buffer = buffer.into();
    }



    pub fn use_buffer_with_count(&mut self, buffer: Buffer<PointUniform>, count: u64)
    {
        self.point_buffer = GrowBuffer::from_buffer_with_count(buffer, count);
    }



    pub fn count(&self) -> u64
    {
        self.point_buffer.count()
    }



    pub fn capacity(&self) -> u64
    {
        self.point_buffer.capacity()
    }
}



impl Pass for PointPass
{
    fn prepare(&self, _context: &mut FrameContext<'_>) {}



    fn draw(&self, context: &mut FrameContext<'_>, view: usize)
    {
        let desc = self
            .desc
            .clone()
            .color_mask(context.views[view].color_mask)
            .layout(context.view_buffer.layout().clone());
        let pipeline = context
            .pipelines
            .render(context.context, context.shaders, &desc)
            .unwrap();

        let mut pass = begin_render_pass(context.encoder, context.frame, PassOps::load());
        pass.set_pipeline(&pipeline);

        pass.set_bind_group(
            0,
            context.view_buffer.bind_group(),
            &[context.view_buffer.offset(view)],
        );

        pass.set_vertex_buffer(0, self.point_buffer.buffer().handle().slice(..));
        pass.draw(0..6, 0..self.count() as u32);
    }
}
