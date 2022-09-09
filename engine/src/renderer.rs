use cgmath::{prelude::*, Deg};
use itertools::Itertools;
use wgpu::util::DeviceExt;
use winit::{event::Event, window::Window};

use crate::{
    camera::{Camera, FPSCamera, Projection},
    controller::Controller,
    light::{BaseLight, LightBufferManager, LightKind, PointLight, SpotLight},
    model::{DrawMesh, Instance, InstanceRaw, Material, Mesh, ModelVertex, Vertex},
    texture::Texture,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
}

pub struct Renderer {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,

    instance_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,

    depth_texture: Texture,

    camera_bind_group: wgpu::BindGroup,

    render_pipeline: wgpu::RenderPipeline,
    //light_render_pipeline: wgpu::RenderPipeline,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub instances: Vec<Instance>,
    pub camera: FPSCamera,
    material: Material,
    plane_mesh: Mesh,
    spot_light: SpotLight,
    point_light: PointLight,
    light_manager: LightBufferManager,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .expect("Failed to create device and/or queue");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        // ====================== Create lights ======================
        let mut light_manager = LightBufferManager::new(&device);
        light_manager.ambient_count += 1;
        light_manager.update_light_buffer(
            &queue,
            LightKind::Ambient,
            0,
            &BaseLight::new([1.0, 1.0, 1.0], 0.01),
        );
        let spot_light = SpotLight::new(
            [1.0, 0.0, 0.0],
            [2.0, 2.0, 2.0],
            [1.0, -1.0, 1.0],
            Deg(30.0),
            0.5,
            0.5,
            0.5,
        );
        light_manager.update_light_buffer(&queue, LightKind::Spot, 0, &spot_light);
        light_manager.spot_count += 1;
        let point_light = PointLight::new([0.0, 1.0, 1.0], [2.0, 2.0, 2.0], 0.5, 0.5, 0.5);
        light_manager.update_light_buffer(&queue, LightKind::Point, 0, &point_light);
        light_manager.point_count += 1;
        light_manager.update_light_counts(&queue);
        // ===========================================================

        // ====================== Create Instances ======================
        //const NUM_INSTANCES_PER_ROW: u32 = 20;
        //const SPACE_BETWEEN: f32 = 2.0;
        //let instances = (0..NUM_INSTANCES_PER_ROW)
        //    .flat_map(|z| {
        //        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        //            let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        //            let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

        //            let position = cgmath::Vector3 { x, y: 0.0, z };

        //            //let rotation = if position.is_zero() {
        //            //    cgmath::Quaternion::from_axis_angle(
        //            //        cgmath::Vector3::unit_z(),
        //            //        cgmath::Deg(0.0),
        //            //    )
        //            //} else {
        //            //    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
        //            //};
        //            let rotation = cgmath::Quaternion::from_axis_angle(
        //                cgmath::Vector3::unit_z(),
        //                cgmath::Deg(0.0),
        //            );

        //            Instance { position, rotation }
        //        })
        //    })
        //    .collect::<Vec<_>>();
        let instances = vec![Instance {
            position: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
        }];
        let instance_data = instances.iter().map(Instance::to_raw).collect_vec();
        // ==============================================================

        // ====================== Create Camera ======================
        let camera = FPSCamera::new(
            (0.0, 10.0, 20.0),
            Deg(-90.0),
            Deg(-20.0),
            Projection::new(config.width, config.height, Deg(45.0), 0.1, 100.0),
            4.0,
            0.4,
        );
        // ==========================================================

        // Create textures
        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        // Create buffers
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera.uniform()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind groups
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera_bind_group_layout =
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
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // ====================== Create Geometry ======================
        let material = Material::from_files(
            "Happy-Tree",
            &device,
            &queue,
            &texture_bind_group_layout,
            "cube-diffuse.jpg",
            Some("cube-normal.png")
        )
        .await;
        let plane_mesh = Mesh::plane(&device, 10.0, 10.0, 10, 10);
        // =============================================================

        // Create pipelines
        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Basic Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("basic.wgsl").into()),
            };
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_manager.light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            create_render_pipeline(
                "Render Pipeline",
                &device,
                &layout,
                config.format,
                Some(Texture::DEPTH_FORMAT),
                &[ModelVertex::desc(), InstanceRaw::desc()],
                shader,
            )
        };

        //let light_render_pipeline = {
        //    let shader = wgpu::ShaderModuleDescriptor {
        //        label: Some("Light Shader"),
        //        source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
        //    };
        //    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //        label: Some("Light Render Pipeline Layout"),
        //        bind_group_layouts: &[&camera_bind_group_layout, &scene_light.light_bind_group_layout],
        //        push_constant_ranges: &[],
        //    });
        //    create_render_pipeline(
        //        "Light Render Pipeline",
        //        &device,
        //        &layout,
        //        config.format,
        //        Some(Texture::DEPTH_FORMAT),
        //        &[ModelVertex::desc()],
        //        shader,
        //    )
        //};

        return Self {
            surface,
            config,
            device,
            queue,
            depth_texture,
            instance_buffer,
            camera_buffer,
            camera_bind_group,
            render_pipeline,
            //light_render_pipeline,
            size,
            instances,
            camera,
            material,
            plane_mesh,
            spot_light,
            point_light,
            light_manager,
        };
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.camera
                .projection_mut()
                .resize(new_size.width, new_size.height);
        }
    }

    // True if event was fully processed
    pub fn input(&mut self, _: &Event<()>) -> bool {
        return false;
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        // Update camera
        self.camera.update(dt);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera.uniform()]),
        );

        // Update lights
        self.spot_light.direction =
            cgmath::Quaternion::from_angle_y(Deg(1.0)).rotate_vector(self.spot_light.direction);
        self.light_manager
            .update_light_buffer(&self.queue, LightKind::Spot, 0, &self.spot_light);
        self.point_light.position =
            cgmath::Quaternion::from_angle_y(Deg(-1.0)).rotate_point(self.point_light.position);
        self.light_manager
            .update_light_buffer(&self.queue, LightKind::Point, 0, &self.point_light);
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // Render light (for debbuging)
            //render_pass.set_pipeline(&self.light_render_pipeline);
            //render_pass.draw_light_model(
            //    &self.obj_model,
            //    &self.camera_bind_group,
            //    &self.light_bind_group,
            //);

            // Render models
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_mesh(
                &self.plane_mesh,
                &self.material,
                &self.camera_bind_group,
                &self.light_manager.light_bind_group,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn create_render_pipeline(
    label: &str,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });
}
