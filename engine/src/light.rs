use std::mem::size_of;

use cgmath::Angle;
use wgpu::util::DeviceExt;

pub enum LightKind {
    Ambient,
    Directional,
    Point,
    Spot,
}

pub const MAX_AMBIENT_LIGHTS: usize = 1;
pub const MAX_DIRECTIONAL_LIGHTS: usize = 10;
pub const MAX_POINT_LIGHTS: usize = 256;
pub const MAX_SPOT_LIGHTS: usize = 256;
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct LightBuffer {
    pub ambient_uniforms: [[f32; 4]; MAX_AMBIENT_LIGHTS],
    pub dir_uniforms: [DirectionalLightUniform; MAX_DIRECTIONAL_LIGHTS],
    pub point_uniforms: [PointLightUniform; MAX_POINT_LIGHTS],
    pub spot_uniforms: [SpotLightUniform; MAX_SPOT_LIGHTS],
    pub uniform_lens: [u32; 4],
}

impl Default for LightBuffer {
    fn default() -> Self {
        Self {
            ambient_uniforms: [[0.0; 4]; MAX_AMBIENT_LIGHTS],
            dir_uniforms: [DirectionalLightUniform::default(); MAX_DIRECTIONAL_LIGHTS],
            point_uniforms: [PointLightUniform::default(); MAX_POINT_LIGHTS],
            spot_uniforms: [SpotLightUniform::default(); MAX_SPOT_LIGHTS],
            uniform_lens: [0; 4],
        }
    }
}

pub struct LightBufferManager {
    light_buffer: wgpu::Buffer,
    pub ambient_count: u32,
    pub directional_count: u32,
    pub point_count: u32,
    pub spot_count: u32,
    pub light_bind_group: wgpu::BindGroup,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
}

impl LightBufferManager {
    fn create_buffer(device: &wgpu::Device, label: &str, data: &[u8]) -> wgpu::Buffer {
        return device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
    }

    pub fn new(device: &wgpu::Device) -> Self {
        let light_buffer_data = LightBuffer::default();
        let light_buffer = LightBufferManager::create_buffer(
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
            ambient_count: 0,
            directional_count: 0,
            point_count: 0,
            spot_count: 0,
            light_buffer,
            light_bind_group,
            light_bind_group_layout,
        }
    }

    const fn calculate_buffer_offset(&self, kind: &LightKind, index: usize) -> usize {
        return match kind {
            LightKind::Ambient => size_of::<[f32; 4]>() * index,
            LightKind::Directional => {
                size_of::<[[f32; 4]; MAX_AMBIENT_LIGHTS]>()
                    + size_of::<DirectionalLightUniform>() * index
            }
            LightKind::Point => {
                size_of::<[[f32; 4]; MAX_AMBIENT_LIGHTS]>()
                    + size_of::<[DirectionalLightUniform; MAX_DIRECTIONAL_LIGHTS]>()
                    + size_of::<PointLightUniform>() * index
            }
            LightKind::Spot => {
                size_of::<[[f32; 4]; MAX_AMBIENT_LIGHTS]>()
                    + size_of::<[DirectionalLightUniform; MAX_DIRECTIONAL_LIGHTS]>()
                    + size_of::<[PointLightUniform; MAX_POINT_LIGHTS]>()
                    + size_of::<SpotLightUniform>() * index
            }
        };
    }

    pub fn update_light_buffer<L>(
        &self,
        queue: &wgpu::Queue,
        kind: LightKind,
        index: usize,
        light: &L,
    ) where
        L: Light,
    {
        let offset = self.calculate_buffer_offset(&kind, index);
        queue.write_buffer(&self.light_buffer, offset as _, &light.buffer_data());
    }

    pub fn update_light_counts(&self, queue: &wgpu::Queue)
    {
        let offset: usize = self.calculate_buffer_offset(&LightKind::Spot, MAX_SPOT_LIGHTS);
        queue.write_buffer(
            &self.light_buffer,
            offset as _,
            bytemuck::cast_slice(&[
                self.ambient_count,
                self.directional_count,
                self.point_count,
                self.spot_count,
            ]),
        );
    }
}

pub trait Light {
    fn buffer_data(&self) -> Vec<u8>;
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

    fn uniform(&self) -> [f32; 4] {
        return [self.color[0], self.color[1], self.color[2], self.strength];
    }
}

impl Light for BaseLight {
    fn buffer_data(&self) -> Vec<u8> {
        return bytemuck::cast_slice(&[self.uniform()]).to_vec();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct DirectionalLightUniform {
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

    fn uniform(&self) -> DirectionalLightUniform {
        return DirectionalLightUniform {
            base: self.base.uniform(),
            direction: self.direction.into(),
            _padding: 0,
        };
    }
}

impl Light for DirectionalLight {
    fn buffer_data(&self) -> Vec<u8> {
        return bytemuck::cast_slice(&[self.uniform()]).to_vec();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLightUniform {
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

    fn uniform(&self) -> PointLightUniform {
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

impl Light for PointLight {
    fn buffer_data(&self) -> Vec<u8> {
        return bytemuck::cast_slice(&[self.uniform()]).to_vec();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct SpotLightUniform {
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

    fn uniform(&self) -> SpotLightUniform {
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

impl Light for SpotLight {
    fn buffer_data(&self) -> Vec<u8> {
        return bytemuck::cast_slice(&[self.uniform()]).to_vec();
    }
}
