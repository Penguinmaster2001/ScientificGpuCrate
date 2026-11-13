// Driver semantics of `encode_frame`: prepare once per frame, then view-major draws.
mod common;
use common::*;
use scig::core::pass::{
    FrameContext,
    Pass,
};
use std::{
    cell::RefCell,
    rc::Rc,
};



struct RecordingPass
{
    name: &'static str,
    log: Rc<RefCell<Vec<String>>>,
}



impl Pass for RecordingPass
{
    fn prepare(&self, _context: &mut FrameContext<'_>)
    {
        self.log.borrow_mut().push(format!("{}.prepare", self.name));
    }



    fn draw(&self, _context: &mut FrameContext<'_>, view: usize)
    {
        self.log
            .borrow_mut()
            .push(format!("{}.draw{}", self.name, view));
    }
}



fn recording_passes() -> (Rc<RefCell<Vec<String>>>, RecordingPass, RecordingPass)
{
    let log = Rc::new(RefCell::new(vec![]));
    let a = RecordingPass {
        name: "A",
        log: log.clone(),
    };
    let b = RecordingPass {
        name: "B",
        log: log.clone(),
    };
    (log, a, b)
}



#[tokio::test]
async fn prepare_once_then_draws_are_view_major()
{
    let mut h = Harness::new().await;
    let (log, a, b) = recording_passes();
    h.render_rgba8((4, 4), &[cam_view(), cam_view()], &[&a, &b]);
    // View-major: both passes finish view 0 before either starts view 1.
    // Pass-major would break depth/clear handling for multi-pass scenes.
    assert_eq!(
        *log.borrow(),
        [
            "A.prepare",
            "B.prepare",
            "A.draw0",
            "B.draw0",
            "A.draw1",
            "B.draw1"
        ]
    );
}



#[tokio::test]
async fn prepare_runs_even_without_views_and_draw_does_not()
{
    let mut h = Harness::new().await;
    let (log, a, b) = recording_passes();
    h.render_rgba8((4, 4), &[], &[&a, &b]);
    assert_eq!(*log.borrow(), ["A.prepare", "B.prepare"]);
}



#[tokio::test]
async fn zero_views_still_clears_to_the_clear_color()
{
    let mut h = Harness::new().await;
    let out = h.render(
        (4, 3),
        RGBA8,
        wgpu::Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
        &[],
        &[],
    );
    assert_eq!(out.len(), 4 * 3 * 4);
    assert!(pixels(&out, 4).all(|(_, _, p)| p == [0, 255, 0, 255]));
}



#[tokio::test]
async fn no_passes_one_view_is_just_the_clear()
{
    let mut h = Harness::new().await;
    let out = h.render_rgba8((4, 4), &[cam_view()], &[]);
    assert!(pixels(&out, 4).all(|(_, _, p)| p == BLACK));
}



#[tokio::test]
async fn driver_can_run_many_frames_with_one_harness()
{
    let mut h = Harness::new().await;
    let (log, a, _b) = recording_passes();
    for _ in 0..5
    {
        h.render_rgba8((4, 4), &[cam_view()], &[&a]);
    }
    assert_eq!(log.borrow().len(), 10);
}
