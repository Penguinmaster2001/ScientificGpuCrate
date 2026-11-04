use crate::core::{
    context::Context,
    pipeline::PipelineCache,
    shader::ShaderLib,
    target::Frame,
    view::View,
};



pub trait Pass
{
    fn record(&mut self, cx: &mut FrameContext<'_>);
}



pub struct FrameContext<'a>
{
    pub context: &'a Context,
    pub shaders: ShaderLib,
    pub pipelines: PipelineCache,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub frame: &'a Frame,
    pub views: &'a [View],
    pub time: FrameTime,
}



impl<'a> FrameContext<'a>
{
    pub fn views_bind_group(&'a self) -> Option<wgpu::BindGroup>
    {
        todo!()
    }



    pub fn view_offset(&self, _i: usize) -> wgpu::DynamicOffset
    {
        todo!()
    }
}



pub struct FrameTime
{
    pub t: f64,
    pub dt: f32,
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



// struct TrianglePass
// {
//     desc: RenderPipelineDesc,
// }



// impl Pass for TrianglePass
// {
//     fn record(&mut self, context: &mut FrameContext<'_>)
//     {
//         for (i, view) in context.views.iter().enumerate()
//         {
//             let desc = self.desc.clone().color_mask(view.color_mask);
//             let pipeline = context
//                 .pipelines
//                 .render(context.context, &context.shaders, &desc)
//                 .unwrap();
//             let ops = if i == 0
//             {
//                 PassOps::clear(wgpu::Color::BLACK, 0.0)
//             }
//             else
//             {
//                 PassOps::load()
//             };

//             let views_bind_group = context.views_bind_group();
//             let view_offset = context.view_offset(i);

//             let mut pass = begin_render_pass(context.encoder, context.frame, ops);
//             pass.set_pipeline(&pipeline);
//             pass.set_bind_group(0, &views_bind_group, &[view_offset]);
//             pass.draw(0..3, 0..1);
//         }
//     }
// }



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
