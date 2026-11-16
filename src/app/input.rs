use std::collections::HashMap;
use winit::{
    dpi::PhysicalPosition,
    event::{
        DeviceEvent,
        KeyEvent,
        MouseButton,
        MouseScrollDelta,
        WindowEvent,
    },
    keyboard::{
        KeyCode,
        PhysicalKey,
    },
};



pub struct InputSettings
{
    pub sensitivity: f32,
    pub speed: f32,
}



pub trait InputHandler
{
    fn handle_key(&mut self, _key: KeyCode, _pressed: bool) -> bool
    {
        false
    }



    fn handle_mouse_movement<D: Into<(f64, f64)>>(&mut self, _delta: D) -> bool
    {
        false
    }



    fn handle_mouse_button(&mut self, _button: MouseButton, _pressed: bool) -> bool
    {
        false
    }



    fn handle_mouse_scroll(&mut self, _delta: &MouseScrollDelta) -> bool
    {
        false
    }



    fn handle_input(&mut self, _input: InputEvent) -> bool
    {
        false
    }
}



#[derive(Clone, Copy)]
pub enum InputEvent
{
    MouseWheel
    {
        delta: MouseScrollDelta
    },
    MouseMotion
    {
        delta: (f64, f64)
    },
    MouseButton
    {
        button: MouseButton, pressed: bool
    },
    Keyboard
    {
        code: KeyCode, pressed: bool
    },
}



impl TryFrom<WindowEvent> for InputEvent
{
    type Error = ();



    fn try_from(event: WindowEvent) -> Result<Self, Self::Error>
    {
        match event
        {
            WindowEvent::MouseInput { state, button, .. } => Ok(InputEvent::MouseButton {
                button,
                pressed: state.is_pressed(),
            }),
            WindowEvent::MouseWheel { delta, .. } => Ok(InputEvent::MouseWheel { delta }),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => Ok(InputEvent::Keyboard {
                code,
                pressed: state.is_pressed(),
            }),
            _ => Err(()),
        }
    }
}



impl TryFrom<DeviceEvent> for InputEvent
{
    type Error = ();



    fn try_from(event: DeviceEvent) -> Result<Self, Self::Error>
    {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event
        {
            Ok(InputEvent::MouseMotion { delta: (dx, dy) })
        }
        else
        {
            Err(())
        }
    }
}



#[derive(Debug, Default, Clone)]
struct MouseState
{
    pub pixel_scroll_delta: (f64, f64),
    pub line_scroll_delta: (f64, f64),
    pub movement_delta: (f64, f64),
    pub buttons: HashMap<MouseButton, KeyState>,
}



impl MouseState
{
    fn clear(&mut self)
    {
        self.pixel_scroll_delta = (0.0, 0.0);
        self.line_scroll_delta = (0.0, 0.0);
        self.movement_delta = (0.0, 0.0);
        self.buttons.clear();
    }



    fn scroll(&self) -> (f64, f64)
    {
        (
            self.line_scroll_delta.0 + self.pixel_scroll_delta.0,
            self.line_scroll_delta.1 + self.pixel_scroll_delta.1,
        )
    }



    fn button_state(&self, button: &MouseButton) -> &KeyState
    {
        self.buttons.get(button).unwrap_or(&KeyState {
            pressed: false,
            just_changed: false,
        })
    }



    fn end_frame(&mut self)
    {
        self.pixel_scroll_delta = (0.0, 0.0);
        self.line_scroll_delta = (0.0, 0.0);
        self.movement_delta = (0.0, 0.0);
        self.buttons
            .iter_mut()
            .for_each(|k| k.1.just_changed = false);
    }
}



#[derive(Debug, Default, Clone, Copy)]
struct KeyState
{
    pressed: bool,
    just_changed: bool,
}



#[derive(Debug, Default, Clone)]
pub struct InputState
{
    mouse: MouseState,
    keyboard: HashMap<KeyCode, KeyState>,
}



impl InputState
{
    pub fn new() -> Self
    {
        Default::default()
    }



    pub fn apply(&mut self, event: InputEvent)
    {
        match event
        {
            InputEvent::MouseWheel { delta } => match delta
            {
                MouseScrollDelta::LineDelta(x, y) =>
                {
                    self.mouse.line_scroll_delta.0 += x as f64;
                    self.mouse.line_scroll_delta.1 += y as f64;
                }
                MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) =>
                {
                    self.mouse.pixel_scroll_delta.0 += x;
                    self.mouse.pixel_scroll_delta.1 += y;
                }
            },
            InputEvent::MouseMotion { delta } =>
            {
                self.mouse.movement_delta.0 += delta.0;
                self.mouse.movement_delta.1 += delta.1;
            }
            InputEvent::MouseButton { button, pressed } => match self.mouse.buttons.entry(button)
            {
                std::collections::hash_map::Entry::Occupied(mut occupied_entry) =>
                {
                    let entry = occupied_entry.get_mut();
                    entry.just_changed = entry.pressed != pressed;
                    entry.pressed = pressed;
                }
                std::collections::hash_map::Entry::Vacant(vacant_entry) =>
                {
                    vacant_entry.insert(KeyState {
                        pressed,
                        just_changed: true,
                    });
                }
            },
            InputEvent::Keyboard { code, pressed } => match self.keyboard.entry(code)
            {
                std::collections::hash_map::Entry::Occupied(mut occupied_entry) =>
                {
                    let entry = occupied_entry.get_mut();
                    entry.just_changed = entry.pressed != pressed;
                    entry.pressed = pressed;
                }
                std::collections::hash_map::Entry::Vacant(vacant_entry) =>
                {
                    vacant_entry.insert(KeyState {
                        pressed,
                        just_changed: true,
                    });
                }
            },
        }
    }



    fn clear(&mut self)
    {
        self.keyboard.clear();
        self.mouse.clear();
    }



    pub fn key_value(&self, key: &KeyCode) -> f64
    {
        if self.key_down(key) { 1.0 } else { 0.0 }
    }



    pub fn key_down(&self, key: &KeyCode) -> bool
    {
        self.key_state(key).pressed
    }



    pub fn key_just_pressed(&self, key: &KeyCode) -> bool
    {
        matches!(
            self.key_state(key),
            KeyState {
                pressed: true,
                just_changed: true
            }
        )
    }



    pub fn key_just_released(&self, key: &KeyCode) -> bool
    {
        matches!(
            self.key_state(key),
            KeyState {
                pressed: false,
                just_changed: true
            }
        )
    }



    pub fn button_down(&self, button: &MouseButton) -> bool
    {
        self.mouse.button_state(button).pressed
    }



    fn key_state(&self, key: &KeyCode) -> &KeyState
    {
        self.keyboard.get(key).unwrap_or(&KeyState {
            pressed: false,
            just_changed: false,
        })
    }



    pub fn mouse_delta(&self) -> (f64, f64)
    {
        self.mouse.movement_delta
    }



    pub fn scroll(&self) -> f64
    {
        self.mouse.scroll().1
    }



    pub fn axis(&self, pos: &KeyCode, neg: &KeyCode) -> f64
    {
        self.key_value(pos) - self.key_value(neg)
    }



    pub fn focus_lost(&mut self)
    {
        self.clear();
    }



    pub fn end_frame(&mut self)
    {
        self.keyboard
            .iter_mut()
            .for_each(|k| k.1.just_changed = false);
        self.mouse.end_frame();
    }
}
