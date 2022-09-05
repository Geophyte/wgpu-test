use winit::event::{ElementState, MouseButton, VirtualKeyCode};

trait Controller {
    fn on_keyboard_input(&mut self, state: ElementState, key: VirtualKeyCode);
    fn on_mouse_move(&mut self, position: (f32, f32));
    fn on_mouse_wheel(&mut self, delta: f32);
    fn on_mouse_input(&mut self, state: ElementState, button: MouseButton);
    fn update(&mut self, dt: std::time::Duration);
}
