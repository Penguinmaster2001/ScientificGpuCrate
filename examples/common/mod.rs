use scig::{
    app::window::{
        EnvFactory,
        ScigApp,
    },
    core::{
        math,
        pass::{
            FrameContext,
            Pass,
            PassOps,
            begin_render_pass,
        },
        pipeline::RenderPipelineDesc,
        view::ViewProjection,
    },
};
use winit::{
    event_loop::EventLoop,
    platform::wayland::EventLoopBuilderExtWayland,
};



pub const RGBA8: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;



pub fn eye_view(x: f32) -> glam::Affine3
{
    math::look_at([x, 0.0, 0.0], [x, 0.0, -1.0], [0.0, 1.0, 0.0])
}



pub fn proj90() -> ViewProjection
{
    ViewProjection::PerspectiveInfRev { fov_deg: 90.0 }
}



#[derive(Clone)]
pub struct TrianglePass
{
    pub desc: RenderPipelineDesc,
    pub instances: u32,
    pub uses_view: bool,
}



impl TrianglePass
{
    pub fn new(desc: RenderPipelineDesc) -> Self
    {
        Self {
            desc,
            instances: 1,
            uses_view: false,
        }
    }
}



impl Pass for TrianglePass
{
    fn draw(&self, context: &mut FrameContext<'_>, view: usize)
    {
        let desc = self.desc.clone().color_mask(context.views[view].color_mask);
        let pipeline = context
            .pipelines
            .render(context.context, context.shaders, &desc)
            .unwrap();

        let mut pass = begin_render_pass(context.encoder, context.frame, PassOps::load());
        pass.set_pipeline(&pipeline);

        if self.uses_view
        {
            pass.set_bind_group(
                0,
                context.view_buffer.bind_group(),
                &[context.view_buffer.offset(view)],
            );
        }
        pass.draw(0..3, 0..self.instances);
    }
}



pub fn run_env(factory: EnvFactory) -> anyhow::Result<()>
{
    let event_loop = EventLoop::with_user_event().with_any_thread(true).build()?;
    let mut app = ScigApp::new(factory);
    event_loop.run_app(&mut app)?;

    Ok(())
}
