use cgmath::{Angle, Deg};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct LightBuffer {
    pub ambient_uniforms: [[f32; 4]; 1],
    pub dir_uniforms: [DirectionalLightUniform; 10],
    pub point_uniforms: [PointLightUniform; 10],
    pub spot_uniforms: [SpotLightUniform; 10],
    pub uniform_lens: [u32; 4],
}

impl Default for LightBuffer {
    fn default() -> Self {
        Self {
            ambient_uniforms: [[0.0; 4]; 1],
            dir_uniforms: [DirectionalLightUniform::default(); 10],
            point_uniforms: [PointLightUniform::default(); 10],
            spot_uniforms: [SpotLightUniform::default(); 10],
            uniform_lens: [0; 4],
        }
    }
}

pub struct SceneLights {
    pub ambient_lights: Vec<BaseLight>,
    pub directional_lights: Vec<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
    pub spot_lights: Vec<SpotLight>,
    light_buffer: wgpu::Buffer,
    pub light_bind_group: wgpu::BindGroup,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
}

impl SceneLights {
    fn create_buffer(device: &wgpu::Device, label: &str, data: &[u8]) -> wgpu::Buffer {
        return device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
    }

    pub fn new(device: &wgpu::Device) -> Self {
        let ambient_lights = vec![BaseLight::new([1.0, 1.0, 1.0], 0.1)];
        let directional_lights = vec![DirectionalLight::new(
            [1.0, 0.5, 0.0],
            0.05,
            [0.0, 0.0, 1.0],
        )];
        let point_lights = vec![PointLight::new(
            [0.0, 1.0, 0.0],
            [2.0, 2.0, 2.0],
            1.0,
            1.0,
            1.0,
        )];
        let spot_lights = vec![SpotLight::new(
            [1.0, 0.0, 0.0],
            [6.0, 2.0, 6.0],
            [1.0, -1.0, 1.0],
            Deg(40.0),
            0.5,
            0.5,
            0.0,
        ),
        SpotLight::new(
            [0.0, 0.0, 1.0],
            [6.0, 2.0, 6.0],
            [-1.0, -1.0, -1.0],
            Deg(40.0),
            0.5,
            0.5,
            0.0,
        )];

        let mut light_buffer_data = LightBuffer::default();
        light_buffer_data.uniform_lens = [
            ambient_lights.len() as _,
            directional_lights.len() as _,
            point_lights.len() as _,
            spot_lights.len() as _,
        ];
        for i in 0..ambient_lights.len() {
            light_buffer_data.ambient_uniforms[i] = ambient_lights[i].uniform();
        }
        for i in 0..directional_lights.len() {
            light_buffer_data.dir_uniforms[i] = directional_lights[i].uniform();
        }
        for i in 0..point_lights.len() {
            light_buffer_data.point_uniforms[i] = point_lights[i].uniform();
        }
        for i in 0..spot_lights.len() {
            light_buffer_data.spot_uniforms[i] = spot_lights[i].uniform();
        }
        let light_buffer = SceneLights::create_buffer(
            device,
            "Light Buffer",
            bytemuck::cast_slice(&[light_buffer_data]),
        );

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("light_bind_group_layout"),
            });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("light_bind_group"),
        });
        Self {
            ambient_lights,
            directional_lights,
            point_lights,
            spot_lights,
            light_buffer,
            light_bind_group,
            light_bind_group_layout,
        }
    }
}

pub struct BaseLight {
    pub color: [f32; 3],
    pub strength: f32,
}

impl BaseLight {
    pub fn new<C>(color: C, strength: f32) -> Self
    where
        C: Into<[f32; 3]>,
    {
        Self {
            color: color.into(),
            strength,
        }
    }

    pub fn uniform(&self) -> [f32; 4] {
        return [self.color[0], self.color[1], self.color[2], self.strength];
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightUniform {
    base: [f32; 4],
    direction: [f32; 3],
    _padding: u32,
}

pub struct DirectionalLight {
    pub base: BaseLight,
    pub direction: cgmath::Vector3<f32>,
}

impl DirectionalLight {
    pub fn new<C, D>(color: C, strength: f32, direction: D) -> Self
    where
        C: Into<[f32; 3]>,
        D: Into<cgmath::Vector3<f32>>,
    {
        Self {
            base: BaseLight::new(color, strength),
            direction: direction.into(),
        }
    }

    pub fn uniform(&self) -> DirectionalLightUniform {
        return DirectionalLightUniform {
            base: self.base.uniform(),
            direction: self.direction.into(),
            _padding: 0,
        };
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniform {
    color: [f32; 3],
    _padding1: u32,
    attenuation: [f32; 3],
    _padding2: u32,
    position: [f32; 3],
    _padding3: u32,
}

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
    pub fn new<C, P>(color: C, position: P, c_att: f32, l_att: f32, e_att: f32) -> Self
    where
        C: Into<[f32; 3]>,
        P: Into<cgmath::Vector3<f32>>,
    {
        Self {
            color: color.into(),
            attenuation: Attenuation {
                constant: c_att,
                linear: l_att,
                exp: e_att,
            },
            position: position.into(),
        }
    }

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

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotLightUniform {
    base_uniform: PointLightUniform,
    direction_cutoffcos: [f32; 4],
}

pub struct SpotLight {
    pub base: PointLight,
    pub direction: cgmath::Vector3<f32>,
    pub cutoff: cgmath::Rad<f32>,
}

impl SpotLight {
    pub fn new<C, P, D, A>(
        color: C,
        position: P,
        direction: D,
        cutoff: A,
        c_att: f32,
        l_att: f32,
        e_att: f32,
    ) -> Self
    where
        C: Into<[f32; 3]>,
        P: Into<cgmath::Vector3<f32>>,
        D: Into<cgmath::Vector3<f32>>,
        A: Into<cgmath::Rad<f32>>,
    {
        Self {
            base: PointLight::new(color, position, c_att, l_att, e_att),
            direction: direction.into(),
            cutoff: cutoff.into(),
        }
    }

    pub fn uniform(&self) -> SpotLightUniform {
        return SpotLightUniform {
            base_uniform: self.base.uniform(),
            direction_cutoffcos: [
                self.direction.x,
                self.direction.y,
                self.direction.z,
                self.cutoff.cos(),
            ],
        };
    }
}
