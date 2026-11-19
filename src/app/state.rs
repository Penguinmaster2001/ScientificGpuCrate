use crate::app::env_thread::EnvRunner;
use crate::app::input::InputState;
use crate::core::context::Context;
use crate::core::pass::{
    FrameTime,
    Pass,
    encode_frame,
};
use crate::core::pipeline::PipelineCache;
use crate::core::readback::{
    EncoderToken,
    TextureReadBack,
};
use crate::core::shader::ShaderLib;
use crate::core::target::Target;
use crate::core::view::ViewBuffer;



pub struct RenderInfo
{
    pub target: Target,
    pub context: Context,
    pub shaders: ShaderLib,
    pub pipelines: PipelineCache,
    pub view_buffer: ViewBuffer,
    pub clear: wgpu::Color,
}



pub struct RenderInfoView<'a>
{
    pub target: &'a Target,
    pub context: &'a Context,
    pub shaders: &'a mut ShaderLib,
    pub pipelines: &'a mut PipelineCache,
    pub view_buffer: &'a ViewBuffer,
    pub clear: &'a mut wgpu::Color,
}



impl RenderInfo
{
    pub fn borrow<'a>(&'a mut self) -> RenderInfoView<'a>
    {
        RenderInfoView {
            target: &self.target,
            context: &self.context,
            shaders: &mut self.shaders,
            pipelines: &mut self.pipelines,
            view_buffer: &self.view_buffer,
            clear: &mut self.clear,
        }
    }
}



#[derive(Debug, Default, Clone)]
pub struct FrameState
{
    pub time: FrameTime,
    pub input: InputState,
}



impl FrameState
{
    pub fn from_time(time: FrameTime) -> Self
    {
        Self {
            time,
            ..Default::default()
        }
    }
}



pub struct State
{
    pub render_info: RenderInfo,
    pub env: EnvRunner,
}



impl State
{
    pub fn new(render_info: RenderInfo, env: EnvRunner) -> Self
    {
        Self { render_info, env }
    }



    pub fn resize(&mut self, width: u32, height: u32)
    {
        self.render_info
            .target
            .resize(&self.render_info.context, (width, height));
    }



    pub fn render(
        &mut self,
        frame_state: FrameState,
        pre_present: Option<impl FnOnce()>,
    ) -> anyhow::Result<()>
    {
        let frame_data = self.env.get_frame(frame_state.clone())?;

        let frame = self.render_info.target.acquire(&self.render_info.context)?;
        let mut encoder = self
            .render_info
            .context
            .device
            .create_command_encoder(&Default::default());

        encode_frame(
            &self.render_info.context,
            &mut self.render_info.shaders,
            &mut self.render_info.pipelines,
            &mut self.render_info.view_buffer,
            &mut encoder,
            &frame,
            &frame_data.views,
            self.render_info.clear,
            &frame_data
                .passes
                .iter()
                .map(|p| p.as_ref() as &dyn Pass)
                .collect::<Vec<_>>(),
            frame_state.time,
        );

        self.render_info.context.queue.submit([encoder.finish()]);

        if let Some(pre_present) = pre_present
        {
            pre_present();
        }

        frame.present(&self.render_info.context);

        Ok(())
    }



    pub fn render_to_texture(
        &mut self,
        size: impl Into<(u32, u32)>,
        format: wgpu::TextureFormat,
    ) -> anyhow::Result<Vec<u8>>
    {
        let frame_data = self.env.get_frame_immediate()?;

        let mut target = Target::offscreen(&self.render_info.context, size, format);
        let frame = target.acquire(&self.render_info.context).unwrap();
        let mut encoder = self
            .render_info
            .context
            .device
            .create_command_encoder(&Default::default());

        encode_frame(
            &self.render_info.context,
            &mut self.render_info.shaders,
            &mut self.render_info.pipelines,
            &mut self.render_info.view_buffer,
            &mut encoder,
            &frame,
            &frame_data.views,
            self.render_info.clear,
            &frame_data
                .passes
                .iter()
                .map(|p| p.as_ref() as &dyn Pass)
                .collect::<Vec<_>>(),
            FrameTime::default(),
        );

        let rb = TextureReadBack::encode_texture(
            &self.render_info.context,
            &mut encoder,
            target
                .color()
                .expect("Offscreen target should have color texture"),
        );
        self.render_info.context.queue.submit([encoder.finish()]);
        frame.present(&self.render_info.context);
        rb.map_texture(&self.render_info.context, EncoderToken { id: 0 })
    }
}
