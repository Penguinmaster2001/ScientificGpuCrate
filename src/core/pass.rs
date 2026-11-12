use crate::core::{
    context::Context,
    pipeline::PipelineCache,
    shader::ShaderLib,
    target::Frame,
    view::{
        View,
        ViewBuffer,
        ViewUniform,
    },
};
use std::time::Duration;



pub trait Pass
{
    fn prepare(&mut self, _context: &mut FrameContext<'_>) {}
    fn draw(&mut self, _context: &mut FrameContext<'_>, _view: usize) {}
}



pub struct FrameContext<'a>
{
    pub context: &'a Context,
    pub shaders: &'a ShaderLib,
    pub pipelines: &'a mut PipelineCache,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub frame: &'a Frame,
    pub views: &'a [View],
    pub view_buffer: &'a ViewBuffer,
    pub time: FrameTime,
}



impl<'a> FrameContext<'a>
{
    pub fn views_bind_group(&'a self) -> &'a wgpu::BindGroup
    {
        self.view_buffer.bind_group()
    }



    pub fn view_offset(&self, i: u64) -> wgpu::DynamicOffset
    {
        (i * self.view_buffer.stride()) as _
    }
}



#[derive(Default, Clone, Copy)]
pub struct FrameTime
{
    pub t: Duration,
    pub dt: Duration,
    pub frame: u64,
}



pub struct PassOps
{
    ops: wgpu::Operations<wgpu::Color>,
    depth_ops: Option<wgpu::Operations<f32>>,
}



impl PassOps
{
    pub fn clear(color: wgpu::Color, depth_clear: f32) -> Self
    {
        Self {
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color),
                store: wgpu::StoreOp::Store,
            },
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(depth_clear),
                store: wgpu::StoreOp::Store,
            }),
        }
    }



    pub fn load() -> Self
    {
        Self {
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
        }
    }



    pub fn load_color_clear_depth(depth_clear: f32) -> Self
    {
        Self {
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(depth_clear),
                store: wgpu::StoreOp::Store,
            }),
        }
    }
}



pub fn begin_render_pass<'e>(
    encoder: &'e mut wgpu::CommandEncoder,
    frame: &Frame,
    pass_ops: PassOps,
) -> wgpu::RenderPass<'e>
{
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &frame.color,
            resolve_target: None,
            ops: pass_ops.ops,
            depth_slice: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &frame.depth,
            depth_ops: pass_ops.depth_ops,
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    })
}



#[expect(clippy::too_many_arguments)]
pub fn encode_frame(
    context: &Context,
    shaders: &mut ShaderLib,
    pipelines: &mut PipelineCache,
    view_buffer: &mut ViewBuffer,
    encoder: &mut wgpu::CommandEncoder,
    frame: &Frame,
    views: &[View],
    clear: wgpu::Color,
    passes: &mut [&mut dyn Pass],
    frame_time: FrameTime,
)
{
    view_buffer.upload(
        context,
        &views
            .iter()
            .map(|v| ViewUniform::new(v, frame.size))
            .collect::<Vec<_>>(),
    );

    let mut frame_context = FrameContext {
        context,
        shaders,
        pipelines,
        encoder,
        frame,
        views,
        view_buffer,
        time: frame_time,
    };

    for pass in passes.iter_mut()
    {
        pass.prepare(&mut frame_context);
    }

    // Color clear pass
    {
        begin_render_pass(
            frame_context.encoder,
            frame_context.frame,
            PassOps::clear(clear, 0.0),
        );
    }

    for view in 0..views.len()
    {
        // Clear pass
        {
            begin_render_pass(
                frame_context.encoder,
                frame_context.frame,
                PassOps::load_color_clear_depth(0.0),
            );
        }

        for pass in passes.iter_mut()
        {
            pass.draw(&mut frame_context, view);
        }
    }
}
