use scig::{
    app::{
        camera::{
            Camera,
            FreeCamera,
        },
        env_thread::{
            EnvRunner,
            RunMode,
        },
        scig_env::{
            FrameData,
            ScigEnv,
        },
        state::{
            FrameState,
            RenderInfoView,
        },
        window::ScigApp,
    },
    core::{
        pass::Pass,
        view::{
            View,
            ViewProjection,
        },
    },
    passes::points::{
        Point,
        PointBlend,
        PointPass,
        PointPassConfig,
    },
};
use std::f32;
use winit::{
    event_loop::EventLoop,
    keyboard::KeyCode,
};



struct PointEnv
{
    pass: Option<PointPass>,
    rotate_view: View,
    camera: FreeCamera,
    use_camera: bool,
}



impl PointEnv
{
    pub fn new() -> Self
    {
        Self {
            pass: None,
            rotate_view: View::new(
                glam::Affine3::from_translation([1.6, 0.0, 0.0].into()),
                ViewProjection::PerspectiveInfRev { fov_deg: 90.0 },
            ),
            camera: FreeCamera::default(),
            use_camera: false,
        }
    }
}



impl ScigEnv for PointEnv
{
    fn initialize(&mut self, render_info: RenderInfoView<'_>)
    {
        *render_info.clear = wgpu::Color {
            r: 0.05,
            g: 0.10,
            b: 0.01,
            a: 1.00,
        };

        let mut pass = PointPass::new(
            render_info.context,
            render_info.shaders,
            PointPassConfig::new(wgpu::TextureFormat::Rgba8Unorm)
                .blend(PointBlend::Alpha)
                .depth(true),
        );

        let res = 64;

        pass.set_points(
            render_info.context,
            &(0..res)
                .flat_map(|x| {
                    (-res..res).map(move |y| {
                        let a = x as f32 / res as f32;
                        let b = y as f32 / res as f32;
                        let (a_s, a_c) = (f32::consts::PI * (a - 0.5)).sin_cos();
                        let (b_s, b_c) = (f32::consts::PI * b).sin_cos();

                        Point {
                            pos: [0.5 + 1.5 * -a_c * b_s, 0.5 * a_s, 5.0 * -a_c * b_c].into(),
                            data: [0.02, 0.0, 0.0, 0.0],
                            color: [a, (b + 1.0) / 2.0, 0.5, 1.0],
                        }
                    })
                })
                .collect::<Vec<_>>(),
        );

        self.pass = Some(pass);
    }



    fn frame(&mut self, frame: FrameState)
    {
        let delta = frame.time.dt.as_secs_f32();

        if frame.input.key_just_pressed(&KeyCode::KeyC)
        {
            self.use_camera = !self.use_camera;
        }

        if self.use_camera
        {
            let (d_yaw, d_pitch) = frame.input.mouse_delta();
            self.camera
                .rotate_local((0.005 * d_pitch) as _, (0.005 * d_yaw) as _, 0.0);

            self.camera.translate_local(
                delta
                    * 2.0
                    * glam::vec3(
                        frame.input.axis(&KeyCode::KeyA, &KeyCode::KeyD) as f32,
                        frame.input.axis(&KeyCode::Space, &KeyCode::ShiftLeft) as f32,
                        frame.input.axis(&KeyCode::KeyW, &KeyCode::KeyS) as f32,
                    ),
            );
        }
        else
        {
            let delta = 0.05 * f32::consts::TAU * delta;
            self.rotate_view.view.translation =
                self.rotate_view.view.translation.rotate_y(0.05 * delta);

            self.rotate_view.view.matrix3 =
                glam::Mat3::from_euler(glam::EulerRot::default(), -0.025 * delta, 0.0, 0.0)
                    * self.rotate_view.view.matrix3;
        }
    }



    fn get_frame(&mut self) -> FrameData
    {
        let passes: Vec<Box<dyn Pass + Send + Sync>> = self
            .pass
            .iter()
            .map(|p| Box::new(p.clone()) as Box<dyn Pass + Send + Sync>)
            .collect();

        FrameData {
            views: vec![if self.use_camera
            {
                self.camera.view()
            }
            else
            {
                self.rotate_view
            }],
            passes,
        }
    }
}



fn main() -> anyhow::Result<()>
{
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = ScigApp::new(Box::new(|render_info| {
        let mut env = PointEnv::new();
        env.initialize(render_info);

        EnvRunner::new(
            env,
            RunMode::FrameLock {
                step_pre_frame: true,
            },
        )
    }));
    event_loop.run_app(&mut app)?;

    Ok(())
}
