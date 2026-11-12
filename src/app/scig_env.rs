use crate::core::{
    pass::{
        FrameTime,
        Pass,
    },
    view::View,
};



#[derive(Default)]
pub struct FrameData<'a>
{
    pub views: &'a [View],
    pub passes: &'a mut [&'a mut dyn Pass],
}



pub trait ScigEnv
{
    fn get_frame(&mut self, frame_time: FrameTime) -> FrameData<'_>;
}
