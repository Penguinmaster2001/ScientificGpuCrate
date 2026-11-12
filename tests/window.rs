use scig::app::window::ScigApp;
use winit::{
    event_loop::EventLoop,
    platform::wayland::EventLoopBuilderExtWayland,
};



fn run_env() -> anyhow::Result<()>
{
    let event_loop = EventLoop::with_user_event().with_any_thread(true).build()?;
    let mut app = ScigApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}



#[test]
fn show_window()
{
    run_env().unwrap()
}
