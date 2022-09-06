use winit::event::{ElementState, MouseButton, VirtualKeyCode};

pub enum ControllerEvent {
    MouseMove((f64, f64)),
    MouseScroll(f32),
    MouseInput(ElementState, MouseButton),
    KeyboardInput(ElementState, VirtualKeyCode),
}

pub trait Controller {
    fn input(&mut self, event: ControllerEvent);
    fn update(&mut self, dt: std::time::Duration);
}
