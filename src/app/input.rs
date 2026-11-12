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



#[derive(Default)]
struct MouseState
{
    pub pixel_scroll_delta: (f64, f64),
    pub line_scroll_delta: (f64, f64),
    pub movement_delta: (f64, f64),
    pub buttons: Vec<(MouseButton, bool)>,
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
}



#[derive(Default)]
struct FrameInput
{
    pub keyboard: Vec<(KeyCode, bool)>,
    pub mouse: MouseState,
}



impl FrameInput
{
    fn apply(&mut self, event: InputEvent)
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
            InputEvent::MouseButton { button, pressed } =>
            {
                if let Some(b) = self.mouse.buttons.iter_mut().find(|b| b.0 == button)
                {
                    b.1 = pressed;
                }
                else
                {
                    self.mouse.buttons.push((button, pressed))
                }
            }
            InputEvent::Keyboard { code, pressed } =>
            {
                if let Some(k) = self.keyboard.iter_mut().find(|k| k.0 == code)
                {
                    k.1 = pressed;
                }
                else
                {
                    self.keyboard.push((code, pressed))
                }
            }
        }
    }



    fn clear(&mut self)
    {
        self.keyboard.clear();
        self.mouse.clear();
    }



    fn key_value(&self, key: KeyCode) -> f64
    {
        if self.key_down(key) { 1.0 } else { 0.0 }
    }



    fn key_down(&self, key: KeyCode) -> bool
    {
        self.keyboard.iter().any(|e| e.1 && e.0 == key)
    }



    fn button_down(&self, button: MouseButton) -> bool
    {
        self.mouse.buttons.iter().any(|b| b.1 && b.0 == button)
    }



    fn mouse_delta(&self) -> (f64, f64)
    {
        self.mouse.movement_delta
    }



    fn scroll(&self) -> f64
    {
        self.mouse.scroll().1
    }



    fn axis(&self, pos: KeyCode, neg: KeyCode) -> f64
    {
        self.key_value(pos) - self.key_value(neg)
    }
}



pub struct InputState
{
    a_is_current: bool,
    frame_a: FrameInput,
    frame_b: FrameInput,
}



impl InputState
{
    pub fn new() -> Self
    {
        Self {
            a_is_current: true,
            frame_a: Default::default(),
            frame_b: Default::default(),
        }
    }



    pub fn apply(&mut self, event: InputEvent)
    {
        self.cur_frame_mut().apply(event)
    }



    pub fn is_down(&self, key: KeyCode) -> bool
    {
        self.cur_frame().key_down(key)
    }



    pub fn just_pressed(&self, key: KeyCode) -> bool
    {
        self.cur_frame().key_down(key) && !self.prev_frame().key_down(key)
    }



    pub fn just_released(&self, key: KeyCode) -> bool
    {
        !self.cur_frame().key_down(key) && self.prev_frame().key_down(key)
    }



    pub fn button_down(&self, button: MouseButton) -> bool
    {
        self.cur_frame().button_down(button)
    }



    pub fn mouse_delta(&self) -> (f64, f64)
    {
        self.cur_frame().mouse_delta()
    }



    pub fn scroll(&self) -> f64
    {
        self.cur_frame().scroll()
    }



    pub fn axis(&self, pos: KeyCode, neg: KeyCode) -> f64
    {
        self.cur_frame().axis(pos, neg)
    }



    pub fn end_frame(&mut self)
    {
        self.swap();
        self.clear_current();
    }



    pub fn focus_lost(&mut self)
    {
        self.frame_a.clear();
        self.frame_b.clear();
    }



    fn swap(&mut self)
    {
        self.a_is_current = !self.a_is_current;
    }



    fn clear_current(&mut self)
    {
        if self.a_is_current
        {
            self.frame_a.clear();
        }
        else
        {
            self.frame_b.clear();
        }
    }



    fn cur_frame(&self) -> &FrameInput
    {
        if self.a_is_current
        {
            &self.frame_a
        }
        else
        {
            &self.frame_b
        }
    }



    fn cur_frame_mut(&mut self) -> &mut FrameInput
    {
        if self.a_is_current
        {
            &mut self.frame_a
        }
        else
        {
            &mut self.frame_b
        }
    }



    fn prev_frame(&self) -> &FrameInput
    {
        if self.a_is_current
        {
            &self.frame_b
        }
        else
        {
            &self.frame_a
        }
    }
}



impl Default for InputState
{
    fn default() -> Self
    {
        Self::new()
    }
}
