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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniform {
    color: [f32; 3],
    _padding1: u32,
    attenuation: [f32; 3],
    _padding2: u32,
    position: [f32; 3],
    _padding3: u32,
}

#[derive(Debug)]
pub struct Attenuation {
    pub constant: f32,
    pub linear: f32,
    pub exp: f32,
}

pub struct PointLight {
    pub color: [f32; 3],
    pub attenuation: Attenuation,
    pub position: cgmath::Vector3<f32>,
}

impl PointLight {
    pub fn uniform(&self) -> PointLightUniform {
        return PointLightUniform {
            color: self.color,
            _padding1: 0,
            attenuation: [
                self.attenuation.constant,
                self.attenuation.linear,
                self.attenuation.exp,
            ],
            _padding2: 0,
            position: self.position.into(),
            _padding3: 0,
        };
    }
}
