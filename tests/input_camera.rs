#![cfg(feature = "winit")]



use scig::app::input::{
    InputEvent,
    InputState,
};
use winit::{
    event::{
        MouseButton,
        MouseScrollDelta,
    },
    keyboard::KeyCode,
};



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
