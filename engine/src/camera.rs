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
    pub fn new<E: Into<cgmath::Point3<f32>>, T: Into<cgmath::Point3<f32>>>(
        eye: E,
        target: T,
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
            view_proj: (proj * view).into(),
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

const SAFE_FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2 - 0.0001;

pub struct FPSCamera {
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    
    pub position: cgmath::Point3<f32>,
    pub projection: Projection,
    pub speed: f32,
    pub sensitivity: f32,
}

impl FPSCamera {
    pub fn new<V: Into<cgmath::Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
        projection: Projection,
        speed: f32,
        sensitivity: f32
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            projection,
            speed,
            sensitivity
        }
    }
}

impl Camera for FPSCamera {
    fn view_proj(&self) -> CameraUniform {
        let view = Matrix4::look_to_rh(
            self.position,
            cgmath::Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.0.sin()).normalize(),
            cgmath::Vector3::unit_y(),
        );
        let proj = self.projection.calc_matrix();

        return CameraUniform {
            view_position: self.position.to_homogeneous().into(),
            view_proj: (proj * view).into(),
        };
    }

    fn projection(&self) -> &Projection {
        return &self.projection;
    }

    fn projection_mut(&mut self) -> &mut Projection {
        return &mut self.projection;
    }
}

impl Controller for FPSCamera {
    fn input(&mut self, event: ControllerEvent) {
        match event {
            ControllerEvent::KeyboardInput(state, key) => {
                let amount = if state == ElementState::Pressed {1.0} else {0.0};
                match key {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.amount_forward = amount;
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.amount_backward = amount;
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.amount_left = amount;
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.amount_right = amount;
                    }
                    VirtualKeyCode::Space => {
                        self.amount_up = amount;
                    }
                    VirtualKeyCode::LShift => {
                        self.amount_down = amount;
                    },
                    _ => {}
                }
            },
            ControllerEvent::MouseMove((dx, dy)) => {
                self.rotate_horizontal = dx as f32;
                self.rotate_vertical = dy as f32;
            },
            ControllerEvent::MouseScroll(scroll) => {
                self.scroll -= scroll;
            },
            _ => {}
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let forward = cgmath::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = cgmath::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        self.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = self.pitch.0.sin_cos();
        let scrollward = cgmath::Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        self.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        self.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        self.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}
