use crate::app::env_thread::EnvRunner;
use crate::app::input::{
    InputEvent,
    InputState,
};
use crate::app::state::{
    FrameState,
    RenderInfo,
    RenderInfoView,
    State,
};
use crate::app::window;
use crate::core::context::{
    Context,
    ContextBuilder,
};
use crate::core::pass::FrameTime;
use crate::core::pipeline::PipelineCache;
use crate::core::shader::ShaderLib;
use crate::core::target::Target;
use crate::core::texture::texture_data_to_image;
use crate::core::view::ViewBuffer;
use pollster::FutureExt;
use std::{
    sync::Arc,
    time::Instant,
};
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;
use winit::window::Window;
use winit::{
    application::ApplicationHandler,
    event::{
        DeviceEvent,
        DeviceId,
    },
    event_loop::ActiveEventLoop,
};



pub type EnvFactory = Box<dyn FnOnce(RenderInfoView) -> EnvRunner>;



#[expect(clippy::large_enum_variant)]
enum AppState
{
    PreInit(EnvFactory),
    Running(State),
    Error,
}



pub struct ScigApp
{
    state: AppState,
    last_time: Instant,
    start_time: Instant,
    frame: u64,
    input: InputState,
    window: Option<Arc<window::Window>>,
}



impl ScigApp
{
    pub fn new(factory: EnvFactory) -> Self
    {
        Self {
            state: AppState::PreInit(factory),
            last_time: Instant::now(),
            start_time: Instant::now(),
            frame: 0,
            input: InputState::new(),
            window: None,
        }
    }
}



impl ApplicationHandler<State> for ScigApp
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop)
    {
        let window_attributes = Window::default_attributes();

        let window = match event_loop.create_window(window_attributes)
        {
            Ok(window) => Arc::new(window),
            Err(error) =>
            {
                eprintln!("Failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window.clone());

        if let AppState::PreInit(factory) = std::mem::replace(&mut self.state, AppState::Error)
        {
            let (context, surface) =
                Context::with_surface(ContextBuilder::default(), window.clone())
                    .block_on()
                    .unwrap();
            let shaders = ShaderLib::new();
            let pipelines = PipelineCache::new();
            let clear = wgpu::Color::BLACK;
            let view_buffer = ViewBuffer::new(&context);
            let target = Target::window(&context, surface, window.inner_size()).unwrap();

            let mut render_info = RenderInfo {
                target,
                context,
                shaders,
                pipelines,
                view_buffer,
                clear,
            };

            let env = factory(render_info.borrow());

            self.state = AppState::Running(State::new(render_info, env));
        }
    }



    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: State)
    {
        self.state = AppState::Running(event);
    }



    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    )
    {
        if let Ok(input_event) = InputEvent::try_from(event)
        {
            self.input.apply(input_event);
        }
    }



    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop)
    {
        if let Some(window) = self.window.clone()
        {
            window.request_redraw();
        }
    }



    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    )
    {
        let state = match &mut self.state
        {
            AppState::Running(canvas) => canvas,
            _ =>
            {
                println!("incorrect state");
                return;
            }
        };

        match event
        {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested =>
            {
                let now = Instant::now();
                let delta_time = now - self.last_time;
                let run_time = now - self.start_time;

                let frame_time = FrameTime {
                    t: run_time,
                    dt: delta_time,
                    frame: self.frame,
                };

                self.frame += 1;
                self.last_time = now;

                match state.render(
                    FrameState {
                        time: frame_time,
                        input: self.input.clone(),
                    },
                    Some(|| {
                        if let Some(window) = self.window.clone()
                        {
                            window.pre_present_notify();
                        }
                    }),
                )
                {
                    Ok(_) =>
                    {
                        if self
                            .input
                            .key_combo_just_pressed([&KeyCode::AltLeft, &KeyCode::KeyP])
                        {
                            let size = (3840, 2160);
                            let data = state
                                .render_to_texture(size, wgpu::TextureFormat::Rgba8Unorm)
                                .unwrap();
                            texture_data_to_image(&data, size)
                                .save("image.png")
                                .unwrap();
                        }

                        self.input.end_frame();
                    }

                    // Reconfigure the surface if it's lost or outdated
                    // Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) =>
                    // {
                    //     let size = state.renderer.window.inner_size();
                    //     state.resize(size.width, size.height);
                    // }
                    Err(e) =>
                    {
                        println!("Unable to render {}", e);
                    }
                }
            }

            event =>
            {
                if let Ok(input_event) = InputEvent::try_from(event)
                {
                    self.input.apply(input_event);
                }
            }
        }
    }
}
