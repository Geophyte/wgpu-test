use winit::event::{ElementState, VirtualKeyCode};

use crate::controller::{Controller, ControllerEvent};

pub trait Camera {
    fn view_projection_matrix(&self) -> cgmath::Matrix4<f32>;
    fn view_proj(&self) -> [[f32; 4]; 4] {
        return self.view_projection_matrix().into();
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct PerspectiveCamera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    pub speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl PerspectiveCamera {
    pub fn new(aspect: f32, speed: f32) -> Self {
        return Self {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        };
    }
}

impl Camera for PerspectiveCamera {
    fn view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

impl Controller for PerspectiveCamera {
    fn input(&mut self, event: ControllerEvent) {
        match event {
            ControllerEvent::KeyboardInput(state, key) => {
                let is_pressed = state == ElementState::Pressed;
                match key {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                    }
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                    }
                    VirtualKeyCode::LControl => {
                        self.is_down_pressed = is_pressed;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        use cgmath::InnerSpace;
        let dt = dt.as_secs_f32();

        let forward = self.target - self.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.is_forward_pressed && forward_mag > 1.0 {
            self.eye += forward_norm * self.speed * dt;
        }
        if self.is_backward_pressed {
            self.eye -= forward_norm * self.speed * dt;
        }

        let right = forward_norm.cross(self.up);

        let forward = self.target - self.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            self.eye = self.target - (forward + right * self.speed * dt).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            self.eye = self.target - (forward - right * self.speed * dt).normalize() * forward_mag;
        }

        let up = forward_norm - self.up.normalize();
        if self.is_up_pressed {
            self.eye = self.target - (forward + up * self.speed * dt).normalize() * forward_mag;
        }
        if self.is_down_pressed {
            self.eye = self.target - (forward - up * self.speed * dt).normalize() * forward_mag;
        }
    }
}
