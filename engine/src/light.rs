use cgmath::{Angle, Deg};
use wgpu::util::DeviceExt;

pub struct SceneLights {
    pub ambient_light: BaseLight,
    pub directional_light: DirectionalLight,
    pub point_light: PointLight,
    pub spot_light: SpotLight,
    ambient_buffer: wgpu::Buffer,
    directional_buffer: wgpu::Buffer,
    point_buffer: wgpu::Buffer,
    spot_buffer: wgpu::Buffer,
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
        let ambient_light = BaseLight::new([1.0, 1.0, 1.0], 0.01);
        let directional_light = DirectionalLight::new([1.0, 0.5, 0.0], 0.05, [0.0, 0.0, 1.0]);
        let point_light = PointLight::new([0.0, 1.0, 0.0], [2.0, 2.0, 2.0], 1.0, 1.0, 1.0);
        let spot_light = SpotLight::new(
            [1.0, 0.0, 0.0],
            [6.0, 2.0, 6.0],
            [5.0, -1.0, 5.0],
            Deg(40.0),
            0.5,
            0.5,
            0.0,
        );

        let ambient_buffer = SceneLights::create_buffer(
            device,
            "ambient_buffer",
            bytemuck::cast_slice(&[ambient_light.uniform()]),
        );
        let directional_buffer = SceneLights::create_buffer(
            device,
            "directional_buffer",
            bytemuck::cast_slice(&[directional_light.uniform()]),
        );
        let point_buffer = SceneLights::create_buffer(
            device,
            "point_buffer",
            bytemuck::cast_slice(&[point_light.uniform()]),
        );
        let spot_buffer = SceneLights::create_buffer(
            device,
            "spot_buffer",
            bytemuck::cast_slice(&[spot_light.uniform()]),
        );

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("light_bind_group_layout"),
            });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ambient_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: directional_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: point_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: spot_buffer.as_entire_binding(),
                },
            ],
            label: Some("light_bind_group"),
        });
        Self {
            ambient_light,
            directional_light,
            point_light,
            spot_light,
            ambient_buffer,
            directional_buffer,
            point_buffer,
            spot_buffer,
            light_bind_group,
            light_bind_group_layout
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
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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
