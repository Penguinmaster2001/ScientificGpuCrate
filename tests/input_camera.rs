#![cfg(feature = "winit")]



mod common;



use common::*;
use scig::app::{
    camera::{
        Camera,
        FreeCamera,
    },
    input::{
        InputEvent,
        InputState,
    },
};
use std::f32::consts::{
    FRAC_PI_2,
    FRAC_PI_4,
};
use winit::{
    event::{
        MouseButton,
        MouseScrollDelta,
    },
    keyboard::KeyCode,
};



const EPS: f32 = 1.0e-4;



fn key(code: KeyCode, pressed: bool) -> InputEvent
{
    InputEvent::Keyboard { code, pressed }
}



// ---------- InputState ----------

#[test]
fn key_state_edges()
{
    let mut s = InputState::new();
    s.apply(key(KeyCode::KeyW, true));
    assert!(s.key_down(&KeyCode::KeyW) && s.key_just_pressed(&KeyCode::KeyW));
    s.end_frame();
    assert!(s.key_down(&KeyCode::KeyW) && !s.key_just_pressed(&KeyCode::KeyW));
    s.apply(key(KeyCode::KeyW, false));
    assert!(!s.key_down(&KeyCode::KeyW) && s.key_just_released(&KeyCode::KeyW));
    s.end_frame();
    assert!(!s.key_just_released(&KeyCode::KeyW));
}



#[test]
fn auto_repeat_does_not_retrigger_just_pressed()
{
    let mut s = InputState::new();
    s.apply(key(KeyCode::Space, true));
    s.end_frame();
    s.apply(key(KeyCode::Space, true));
    assert!(s.key_down(&KeyCode::Space));
    assert!(!s.key_just_pressed(&KeyCode::Space));
}



#[test]
fn press_and_release_inside_one_frame_is_still_seen()
{
    let mut s = InputState::new();
    s.apply(key(KeyCode::KeyE, true));
    assert!(s.key_just_pressed(&KeyCode::KeyE));
    s.apply(key(KeyCode::KeyE, false));
    assert!(s.key_just_released(&KeyCode::KeyE));
    assert!(!s.key_down(&KeyCode::KeyE));
}



#[test]
fn mouse_motion_accumulates_until_end_of_frame()
{
    let mut s = InputState::new();
    s.apply(InputEvent::MouseMotion { delta: (1.0, 2.0) });
    s.apply(InputEvent::MouseMotion { delta: (3.0, 4.0) });
    assert_eq!(s.mouse_delta(), (4.0, 6.0));
    s.end_frame();
    assert_eq!(s.mouse_delta(), (0.0, 0.0));
}



#[test]
fn scroll_accumulates_lines()
{
    let mut s = InputState::new();
    s.apply(InputEvent::MouseWheel {
        delta: MouseScrollDelta::LineDelta(0.0, 1.0),
    });
    s.apply(InputEvent::MouseWheel {
        delta: MouseScrollDelta::LineDelta(0.0, 2.0),
    });
    assert_eq!(s.scroll(), 3.0);
    s.end_frame();
    assert_eq!(s.scroll(), 0.0);
}



#[test]
fn mouse_buttons_track_state()
{
    let mut s = InputState::new();
    s.apply(InputEvent::MouseButton {
        button: MouseButton::Left,
        pressed: true,
    });
    assert!(s.button_down(&MouseButton::Left) && !s.button_down(&MouseButton::Right));
}



#[test]
fn axis_combines_two_keys()
{
    let mut s = InputState::new();
    assert_eq!(s.axis(&KeyCode::KeyD, &KeyCode::KeyA), 0.0);
    s.apply(key(KeyCode::KeyD, true));
    assert_eq!(s.axis(&KeyCode::KeyD, &KeyCode::KeyA), 1.0);
    s.apply(key(KeyCode::KeyA, true));
    assert_eq!(s.axis(&KeyCode::KeyD, &KeyCode::KeyA), 0.0);
    s.apply(key(KeyCode::KeyD, false));
    assert_eq!(s.axis(&KeyCode::KeyD, &KeyCode::KeyA), -1.0);
}



#[test]
fn focus_loss_releases_everything()
{
    // Without this, a key released while another window has focus stays down forever.
    let mut s = InputState::new();
    s.apply(key(KeyCode::KeyW, true));
    s.apply(InputEvent::MouseButton {
        button: MouseButton::Left,
        pressed: true,
    });
    s.focus_lost();
    assert!(!s.key_down(&KeyCode::KeyW));
    assert!(!s.button_down(&MouseButton::Left));
}



// ---------- FreeCamera ----------

#[test]
fn free_camera_defaults_to_looking_down_negative_z()
{
    let c = FreeCamera::default();
    assert!(
        vec_approx((*c.forward()).into(), [0.0, 0.0, -1.0], EPS),
        "{:?}",
        c.forward()
    );
}



#[test]
fn positive_yaw_turns_right_and_positive_pitch_looks_up()
{
    let fwd = (*FreeCamera::from_position_attitude([0.0; 3], FRAC_PI_2, 0.0, 0.0, proj90())
        .forward())
    .into();
    assert!(vec_approx(fwd, [0.0, 1.0, 0.0], EPS), "fwd: {fwd:?}");
    let up = *FreeCamera::from_position_attitude([0.0; 3], 0.0, FRAC_PI_4, 0.0, proj90()).forward();
    assert!(
        vec_approx(up.into(), [-0.707, 0.0, -0.707], 0.001),
        "up: {up}"
    );
}



#[test]
fn translate_local_follows_yaw()
{
    let mut c = FreeCamera::default();
    c.translate_local([0.0, 0.0, 2.0]);
    assert!(vec_approx((*c.position()).into(), [0.0, 0.0, -2.0], EPS));
    c.translate_local([1.0, 0.0, 0.0]);
    assert!(vec_approx((*c.position()).into(), [1.0, 0.0, -2.0], EPS));

    let mut t = FreeCamera::from_position_attitude([0.0; 3], FRAC_PI_2, 0.0, 0.0, proj90());
    assert!(
        vec_approx((*t.position()).into(), [0.0, 0.0, 0.0], EPS),
        "origin {}",
        t.position()
    );
    t.translate_local([0.0, 0.0, 2.0]);
    assert!(
        vec_approx((*t.position()).into(), [0.0, 2.0, 0.0], EPS),
        "1st translate {}",
        t.position()
    );
    t.translate_local([1.0, 0.0, 0.0]);
    assert!(
        vec_approx((*t.position()).into(), [1.0, 2.0, 0.0], EPS),
        "right of +x is +z {}",
        t.position()
    );
}



#[test]
fn view_is_consistent_with_forward()
{
    let c = FreeCamera::from_position_attitude([1.0, 2.0, 3.0], 0.7, 0.3, 0.0, proj90());
    let f = c.forward();
    let p = [1.0 + 3.0 * f[0], 2.0 + 3.0 * f[1], 3.0 + 3.0 * f[2]];
    assert!(
        vec_approx(
            c.view().view.inverse().transform_point3(p.into()).into(),
            [0.0, 0.0, 3.0],
            1.0e-3
        ),
        "{}",
        c.view().view.inverse().transform_point3(p.into())
    );
}
