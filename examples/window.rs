mod common;



use common::*;
use scig::app::{
    env_thread::{
        EnvRunner,
        RunMode,
    },
    scig_env::ScigEnv,
};



fn main()
{
    show_window();
}



struct DummyEnv;



impl ScigEnv for DummyEnv {}



fn show_window()
{
    run_env(Box::new(|render_info| {
        *render_info.clear = wgpu::Color::BLUE;
        EnvRunner::new(DummyEnv, RunMode::None)
    }))
    .unwrap()
}
