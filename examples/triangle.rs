mod common;



use common::*;
use scig::{
    app::{
        env_thread::{
            EnvRunner,
            RunMode,
        },
        scig_env::{
            FrameData,
            ScigEnv,
        },
        state::RenderInfoView,
    },
    core::{
        pass::Pass,
        pipeline::{
            DepthDesc,
            RenderPipelineDesc,
        },
        view::View,
    },
};



fn main()
{
    show_window();
}



struct TriEnv
{
    pass: Option<TrianglePass>,
    views: Vec<View>,
}



impl TriEnv
{
    pub fn new() -> Self
    {
        Self {
            pass: None,
            views: vec![],
        }
    }
}



impl ScigEnv for TriEnv
{
    fn initialize(&mut self, render_info: RenderInfoView<'_>)
    {
        *render_info.clear = wgpu::Color {
            r: 0.2,
            g: 0.3,
            b: 0.5,
            a: 1.0,
        };

        let shader = render_info
            .shaders
            .load_shader_source(
                render_info.context,
                include_str!("../shaders/world_tri.wgsl"),
            )
            .id();

        let mut pass = TrianglePass::new(
            RenderPipelineDesc::new(shader, "vs", Some("fs"))
                .color_target(RGBA8, None)
                .depth(DepthDesc::reversed_z())
                .layout(render_info.view_buffer.layout().clone()),
        );
        pass.uses_view = true;

        self.pass = Some(pass);

        self.views = vec![
            View::new(eye_view(-0.5), proj90()).mask(wgpu::ColorWrites::RED),
            View::new(eye_view(0.5), proj90())
                .mask(wgpu::ColorWrites::GREEN | wgpu::ColorWrites::BLUE),
        ];
    }



    fn get_frame(&mut self) -> FrameData
    {
        let passes: Vec<Box<dyn Pass + Send + Sync>> = self
            .pass
            .iter()
            .map(|p| Box::new(p.clone()) as Box<dyn Pass + Send + Sync>)
            .collect();

        FrameData {
            views: self.views.to_vec(),
            passes,
        }
    }
}



fn show_window()
{
    run_env(Box::new(|render_info| {
        let mut env = TriEnv::new();
        env.initialize(render_info);

        EnvRunner::new(env, RunMode::Free)
    }))
    .unwrap()
}
