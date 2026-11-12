#![cfg(feature = "winit")]



mod common;



use common::*;
use scig::{
    app::{
        camera::{
            FreeCamera,
            OrbitCamera,
        },
        input::{
            InputEvent,
            InputState,
        },
    },
    core::{
        math,
        view::{
            View,
            ViewUniform,
        },
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



// ---------- FlyCamera ----------

/*/
#[test]
fn fly_camera_defaults_to_looking_down_negative_z()
{
    let c = FlyCamera::new([0.0, 0.0, 5.0], 0.0, 0.0);
    assert!(vec_approx(c.forward(), [0.0, 0.0, -1.0], EPS));
    assert!(vec_approx(
        math::transform(&c.view(), [0.0, 0.0, 0.0, 1.0]),
        [0.0, 0.0, -5.0, 1.0],
        EPS
    ));
}



#[test]
fn positive_yaw_turns_right_and_positive_pitch_looks_up()
{
    assert!(vec_approx(
        FlyCamera::new([0.0; 3], FRAC_PI_2, 0.0).forward(),
        [1.0, 0.0, 0.0],
        EPS
    ));
    let up = FlyCamera::new([0.0; 3], 0.0, FRAC_PI_4).forward();
    assert!(up[1] > 0.7 && up[2] < 0.0);
}



#[test]
fn pitch_is_clamped_below_the_poles()
{
    let mut c = FlyCamera::new([0.0; 3], 0.0, 0.0);
    c.look(0.0, 100.0);
    assert!(c.forward()[1] < 1.0);
    c.look(0.0, -200.0);
    assert!(c.forward()[1] > -1.0);
    assert!(math::inverse(&c.view()).is_some());
    assert!(c.view().iter().flatten().all(|v| v.is_finite()));
}



#[test]
fn translate_local_follows_yaw()
{
    let mut c = FlyCamera::new([0.0; 3], 0.0, 0.0);
    c.translate_local([0.0, 0.0, 2.0]);
    assert!(vec_approx(c.position(), [0.0, 0.0, -2.0], EPS));
    c.translate_local([1.0, 0.0, 0.0]);
    assert!(vec_approx(c.position(), [1.0, 0.0, -2.0], EPS));

    let mut t = FlyCamera::new([0.0; 3], FRAC_PI_2, 0.0);
    t.translate_local([0.0, 0.0, 2.0]);
    assert!(vec_approx(t.position(), [2.0, 0.0, 0.0], EPS));
    t.translate_local([1.0, 0.0, 0.0]);
    assert!(
        vec_approx(t.position(), [2.0, 0.0, 1.0], EPS),
        "right of +x is +z"
    );
}



#[test]
fn view_is_consistent_with_forward()
{
    let c = FreeCamera::new([1.0, 2.0, 3.0], 0.7, 0.3);
    let f = c.forward();
    let p = [1.0 + 3.0 * f[0], 2.0 + 3.0 * f[1], 3.0 + 3.0 * f[2], 1.0];
    assert!(vec_approx(
        math::transform(&c.view(), p),
        [0.0, 0.0, -3.0, 1.0],
        1.0e-3
    ));
}

*/

// ---------- OrbitCamera ----------

/*
#[test]
fn orbit_position_is_the_eye_not_the_target()
{
    // Regression for the old OrbitCamera::get_position, which returned the target.
    let c = OrbitCamera::new([1.0, 2.0, 3.0], 10.0, 0.0, 0.0, proj90());
    assert!(vec_approx(c.target(), [1.0, 2.0, 3.0], EPS));
    assert!(vec_approx(c.position(), [1.0, 2.0, 13.0], EPS));
}



#[test]
fn orbit_view_puts_the_target_straight_ahead()
{
    let c = OrbitCamera::new([4.0, -1.0, 2.0], 7.0, 0.9, 0.4, proj90());
    let p = math::transform(&c.view(), [4.0, -1.0, 2.0, 1.0]);
    assert!(vec_approx(p, [0.0, 0.0, -7.0, 1.0], 1.0e-3));
}



#[test]
fn view_uniform_reports_the_orbit_eye()
{
    let c = OrbitCamera::new([0.0, 0.0, 0.0], 10.0, 0.5, 0.2, proj90());
    let u = ViewUniform::new(&c.view(), [64, 64]);
    let e = c.position();
    assert!(vec_approx(
        [u.position[0], u.position[1], u.position[2]],
        e,
        1.0e-3
    ));
    assert!(e != c.target());
}



#[test]
fn orbit_zoom_is_clamped()
{
    let mut c = OrbitCamera::new([0.0; 3], 10.0, 0.0, 0.0, proj90());
    c.set_distance_limits(1.0, 20.0);
    c.zoom(-1000.0);
    assert!(approx(c.distance(), 1.0, EPS));
    c.zoom(1000.0);
    assert!(approx(c.distance(), 20.0, EPS));
}



#[test]
fn orbit_pitch_is_clamped()
{
    let mut c = OrbitCamera::new([0.0; 3], 5.0, 0.0, 0.0, proj90());
    c.orbit(0.0, 100.0);
    assert!(c.view().iter().flatten().all(|v| v.is_finite()));
    assert!(math::inverse(&c.view()).is_some());
}
*/
