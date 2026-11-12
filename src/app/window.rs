use crate::app::state::State;
use crate::core::pass::FrameTime;
use std::{
    sync::Arc,
    time::Instant,
};
use winit::event::WindowEvent;
use winit::{
    application::ApplicationHandler,
    event::{
        DeviceEvent,
        DeviceId,
    },
    event_loop::ActiveEventLoop,
    window::Window,
};



pub struct ScigApp
{
    state: Option<State>,
    last_time: Instant,
    start_time: Instant,
    frame: u64,
}



impl ScigApp
{
    pub fn new() -> Self
    {
        Self {
            state: None,
            last_time: Instant::now(),
            start_time: Instant::now(),
            frame: 0,
        }
    }
}



impl Default for ScigApp
{
    fn default() -> Self
    {
        Self::new()
    }
}



impl ApplicationHandler<State> for ScigApp
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop)
    {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

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

        self.state = match pollster::block_on(State::new(window))
        {
            Ok(state) => Some(state),
            Err(error) =>
            {
                eprintln!("Failed to create state: {error}");
                event_loop.exit();
                return;
            }
        };
    }



    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State)
    {
        self.state = Some(event);
    }



    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _event: DeviceEvent,
    )
    {
        // let Some(state) = &mut self.state
        // else
        // {
        //     return;
        // };

        // if let Ok(event) = InputEvent::try_from(event)
        // {
        //     state.handle_input(event_loop, event);
        // }
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
            Some(canvas) => canvas,
            None => return,
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

                match state.render(frame_time)
                {
                    Ok(_) => (),

                    // Reconfigure the surface if it's lost or outdated
                    // Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) =>
                    // {
                    //     let size = state.renderer.window.inner_size();
                    //     state.resize(size.width, size.height);
                    // }
                    Err(e) =>
                    {
                        eprintln!("Unable to render {}", e);
                    }
                }
            }

            event =>
            {
                println!("{:?}", event);
                // if let Ok(event) = InputEvent::try_from(event)
                // {
                //     state.handle_input(event_loop, event);
                // }
            }
        }
    }
}
