pub struct AmbientLight {
    pub color: [f32; 3],
    pub strength: f32,
}

impl AmbientLight {
    pub fn uniform(&self) -> [f32; 4] {
        return [self.color[0], self.color[1], self.color[2], self.strength];
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightUniform {
    color_strength: [f32; 4],
    direction: [f32; 3],
    _padding: u32,
}

pub struct DirectionalLight {
    pub color: [f32; 3],
    pub strength: f32,
    pub direction: cgmath::Vector3<f32>,
}

impl DirectionalLight {
    pub fn uniform(&self) -> DirectionalLightUniform {
        return DirectionalLightUniform {
            color_strength: [self.color[0], self.color[1], self.color[2], self.strength],
            direction: self.direction.into(),
            _padding: 0,
        };
    }
}
