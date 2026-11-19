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
pub struct Line
{
    pub start: glam::Vec2,
    pub end: glam::Vec2,
    pub start_color: [f32; 4],
    pub end_color: [f32; 4],
    pub data: [f32; 4],
}



#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy)]
pub struct LineUniform
{
    pub pos: [[f32; 2]; 2],
    pub start_color: [f32; 4],
    pub end_color: [f32; 4],
    pub data: [f32; 4],
} // 48 bytes



impl LineUniform
{
    pub fn new(line: &Line) -> Self
    {
        Self {
            pos: [line.start.to_array(), line.end.to_array()],
            start_color: line.start_color,
            end_color: line.end_color,
            data: line.data,
        }
    }



    pub fn vertex_desc() -> VertexLayoutDesc
    {
        VertexLayoutDesc {
            array_stride: size_of::<LineUniform>() as wgpu::BufferAddress,
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
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}



#[derive(Debug, Clone)]
pub struct LinePass
{
    buffer: GrowBuffer<LineUniform>,
    desc: RenderPipelineDesc,
}



impl LinePass
{
    pub fn new(context: &Context, shaders: &mut ShaderLib) -> Self
    {
        let shader =
            shaders.load_shader_source(context, include_str!("../../shaders/line_rendering.wgsl"));

        let mut desc = RenderPipelineDesc::new(shader.id(), "vs", Some("fs"))
            .color_target(
                wgpu::TextureFormat::Rgba8Unorm,
                Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            )
            .depth(DepthDesc::reversed_z())
            .vertex(0, LineUniform::vertex_desc());
        desc.cull = None;

        Self {
            buffer: GrowBuffer::new(
                context,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                1,
                BufferAlignment::None,
            ),
            desc,
        }
    }



    pub fn set_lines(&mut self, context: &Context, lines: &[Line])
    {
        // TODO: line rendering must support lines at all angles while maintaining
        // TODO: orthogonal thickness and seamless transitions.
        self.buffer.write(
            context,
            &lines.iter().map(LineUniform::new).collect::<Vec<_>>(),
        );
    }



    pub fn use_buffer<B: Into<GrowBuffer<LineUniform>>>(&mut self, buffer: B)
    {
        self.buffer = buffer.into();
    }



    pub fn use_buffer_with_count(&mut self, buffer: Buffer<LineUniform>, count: u64)
    {
        self.buffer = GrowBuffer::from_buffer_with_count(buffer, count);
    }



    pub fn count(&self) -> u64
    {
        self.buffer.count()
    }



    pub fn capacity(&self) -> u64
    {
        self.buffer.capacity()
    }
}



impl Pass for LinePass
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

        pass.set_vertex_buffer(0, self.buffer.buffer().handle().slice(..));
        pass.draw(0..6, 0..self.count() as u32);
    }
}
