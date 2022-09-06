#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    ambient_light: [f32; 4],
}

pub trait Light {
    fn uniform(&self) -> LightUniform;
}

pub struct AmbientLight {
    pub color: [f32; 3],
    pub ambient_strength: f32,
}

impl AmbientLight {
    pub fn uniform(&self) -> LightUniform {
        return LightUniform {
            ambient_light: [self.color[0], self.color[1], self.color[2], self.ambient_strength]
        };
    }
}
