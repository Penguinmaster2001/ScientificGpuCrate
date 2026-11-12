use crate::app::scig_env::ScigEnv;
use crate::core::context::{
    Context,
    ContextBuilder,
};
use crate::core::pass::{
    FrameTime,
    encode_frame,
};
use crate::core::pipeline::PipelineCache;
use crate::core::shader::ShaderLib;
use crate::core::target::Target;
use crate::core::view::ViewBuffer;
use std::sync::Arc;
use winit::window::Window;



pub struct State
{
    pub target: Target,
    pub context: Context,
    pub shaders: ShaderLib,
    pub pipelines: PipelineCache,
    pub view_buffer: ViewBuffer,
    pub clear: wgpu::Color,
    pub env: Option<Box<dyn ScigEnv>>,
}



impl State
{
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self>
    {
        let (context, surface) =
            Context::with_surface(ContextBuilder::default(), window.clone()).await?;
        let view_buffer = ViewBuffer::new(&context);
        let target = Target::window(&context, surface, window.inner_size())?;

        Ok(Self {
            target,
            context,
            shaders: ShaderLib::new(),
            pipelines: PipelineCache::new(),
            view_buffer,
            clear: wgpu::Color::BLACK,
            env: None,
        })
    }



    pub fn resize(&mut self, width: u32, height: u32)
    {
        self.target.resize(&self.context, (width, height));
    }



    pub fn render(&mut self, frame_time: FrameTime) -> anyhow::Result<()>
    {
        let frame_data = self
            .env
            .as_mut()
            .map_or_default(|e| e.get_frame(frame_time));

        let frame = self.target.acquire(&self.context)?;
        let mut encoder = self
            .context
            .device
            .create_command_encoder(&Default::default());

        encode_frame(
            &self.context,
            &mut self.shaders,
            &mut self.pipelines,
            &mut self.view_buffer,
            &mut encoder,
            &frame,
            frame_data.views,
            self.clear,
            frame_data.passes,
            frame_time,
        );

        self.context.queue.submit([encoder.finish()]);
        frame.present(&self.context);

        Ok(())
    }
}
