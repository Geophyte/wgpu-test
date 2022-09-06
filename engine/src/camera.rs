use cgmath::{perspective, InnerSpace, Matrix4, Rad};
use winit::event::{ElementState, VirtualKeyCode};

use crate::controller::{Controller, ControllerEvent};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        return Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        };
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        return OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar);
    }
}

pub trait Camera {
    fn view_proj(&self) -> CameraUniform;
    fn projection(&self) -> &Projection;
    fn projection_mut(&mut self) -> &mut Projection;
}

pub struct PerspectiveCamera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    pub projection: Projection,
    pub speed: f32,
}

impl PerspectiveCamera {
    pub fn new(
        eye: (f32, f32, f32),
        target: (f32, f32, f32),
        projection: Projection,
        speed: f32,
    ) -> Self {
        Self {
            eye: eye.into(),
            target: target.into(),
            up: cgmath::Vector3::unit_y(),
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            projection,
            speed,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn view_proj(&self) -> CameraUniform {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = self.projection.calc_matrix();

        return CameraUniform {
            view_position: self.eye.to_homogeneous().into(),
            view_proj: (OPENGL_TO_WGPU_MATRIX * proj * view).into()
        };
    }

    fn projection(&self) -> &Projection {
        return &self.projection;
    }

    fn projection_mut(&mut self) -> &mut Projection {
        return &mut self.projection;
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

                    VirtualKeyCode::R => {
                        self.eye = (0.0, 5.0, 10.0).into();
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