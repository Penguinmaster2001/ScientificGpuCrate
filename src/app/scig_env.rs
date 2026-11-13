use crate::{
    app::{
        input::InputEvent,
        state::{
            FrameState,
            RenderInfoView,
        },
    },
    core::{
        pass::Pass,
        view::View,
    },
};
use std::time::Duration;



#[derive(Default)]
pub struct FrameData
{
    pub views: Vec<View>,
    pub passes: Vec<Box<dyn Pass + Send + Sync + 'static>>,
}



pub trait ScigEnv: 'static + Send + Sync
{
    fn initialize(&mut self, _render_info: RenderInfoView<'_>) {}



    fn start(&mut self) {}



    fn stop(&mut self) {}



    fn input(&mut self, _input: InputEvent) {}



    fn step(&mut self, _delta: Duration) {}



    fn frame(&mut self, _frame: FrameState) {}



    fn get_frame(&mut self) -> FrameData
    {
        Default::default()
    }
}
