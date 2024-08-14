use std::sync::Arc;

use wgpu::include_wgsl;

use crate::texture::Texture;

pub struct GpuBound {
    pub proxy: GpuProxy,
    pub dimensions_buffer: wgpu::Buffer,
    pub dimensions_bind_group: wgpu::BindGroup,
    pub size: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct GpuProxy {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub pipelines: Arc<Pipelines>,
}

impl GpuProxy {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, pipelines: Pipelines) -> Self {
        Self {
            device: device,
            queue: queue,
            pipelines: Arc::new(pipelines),
        }
    }
}

#[derive(Debug)]
pub struct Pipelines {
    pub color_pipeline: wgpu::RenderPipeline,
    pub texture_pipeline: wgpu::RenderPipeline,
}

impl GpuBound {
    pub const DIMENSIONS_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Dimensions Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    pub fn new(queue: Arc<wgpu::Queue>, device: Arc<wgpu::Device>, size: (u32, u32)) -> Self {
        let dimensions_bind_group_layout =
            device.create_bind_group_layout(&Self::DIMENSIONS_LAYOUT);

        let dimensions_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dimensions Buffer"),
            size: std::mem::size_of::<(u32, u32)>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let dimensions_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Dimensions Bind Group"),
            layout: &dimensions_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &dimensions_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        queue.write_buffer(
            &dimensions_buffer,
            0,
            bytemuck::cast_slice(&[size.0 as f32, size.1 as f32]),
        );

        let elements_bind_group_layout = device.create_bind_group_layout(&RenderElement::LAYOUT);
        let color_bind_group_layout = device.create_bind_group_layout(&Color::BIND_GROUP_LAYOUT);
        let texture_bind_group_layout =
            device.create_bind_group_layout(&Texture::BIND_GROUP_LAYOUT);

        let color_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &dimensions_bind_group_layout,
                    &elements_bind_group_layout,
                    &color_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let color_shaders = device.create_shader_module(include_wgsl!("shaders/color.wgsl"));

        let color_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&color_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &color_shaders,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &color_shaders,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Texture Pipeline Layout"),
                bind_group_layouts: &[
                    &dimensions_bind_group_layout,
                    &elements_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let texture_shaders = device.create_shader_module(include_wgsl!("shaders/texture.wgsl"));

        let texture_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Texture Pipeline"),
            layout: Some(&texture_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &texture_shaders,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &texture_shaders,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            proxy: GpuProxy::new(
                device,
                queue,
                Pipelines {
                    color_pipeline,
                    texture_pipeline,
                },
            ),
            dimensions_buffer,
            dimensions_bind_group,
            size,
        }
    }

    pub fn resize(&mut self, size: (u32, u32), queue: &wgpu::Queue) {
        self.size = size;

        queue.write_buffer(
            &self.dimensions_buffer,
            0,
            bytemuck::cast_slice(&[size.0 as f32, size.1 as f32]),
        );
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub struct RenderElement {
    pub data: RenderElementData,
    center_buffer: wgpu::Buffer,
    size_buffer: wgpu::Buffer,
    rotation_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    color: RenderColor,
    texture: Option<Arc<Texture>>,
}

pub struct RenderColor {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderElementData {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub color: Color,
}

impl RenderElementData {
    pub fn new(center: [f32; 2], size: [f32; 2], rotation: f32, color: Color) -> Self {
        Self {
            center,
            size,
            rotation,
            color,
        }
    }

    pub fn zeroed() -> Self {
        Self {
            center: [0.0, 0.0],
            size: [0.0, 0.0],
            rotation: 0.0,
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
        }
    }

    pub fn from_transform(transform: &crate::NodeTransform) -> Self {
        Self {
            center: [transform.position.x, transform.position.y],
            size: [transform.scale.x, transform.scale.y],
            rotation: transform.rotation,
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
        }
    }

    pub fn update_transform(&mut self, transform: &crate::NodeTransform) {
        self.center = [transform.position.x, transform.position.y];
        self.size = [transform.scale.x, transform.scale.y];
        self.rotation = transform.rotation;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushType {
    Color,
    Texture,
}

impl RenderElement {
    pub const LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        label: Some("Render Element Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    };

    pub fn zeroed(proxy: &GpuProxy) -> Self {
        let center_buffer = proxy.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Center Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let size_buffer = proxy.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Size Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rotation_buffer = proxy.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rotation Buffer"),
            size: std::mem::size_of::<[f32; 1]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = proxy.device.create_bind_group_layout(&Self::LAYOUT);

        let bind_group = proxy.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Element Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &center_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &size_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &rotation_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let color_buffer = proxy.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Color Buffer"),
            size: std::mem::size_of::<Color>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let color_bind_group_layout = proxy
            .device
            .create_bind_group_layout(&Color::BIND_GROUP_LAYOUT);

        let color_bind_group = proxy.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Bind Group"),
            layout: &color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &color_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            data: RenderElementData::zeroed(),
            center_buffer,
            size_buffer,
            rotation_buffer,
            bind_group,
            color: RenderColor {
                buffer: color_buffer,
                bind_group: color_bind_group,
            },
            texture: None,
        }
    }

    pub fn set_transform(&mut self, transform: &crate::NodeTransform, proxy: &GpuProxy) {
        self.data.update_transform(transform);
        self.write_all(proxy);
    }

    pub fn set_color(&mut self, color: Color, proxy: &GpuProxy) {
        self.data.color = color;
        proxy.queue.write_buffer(
            &self.color.buffer,
            0,
            bytemuck::cast_slice(&[color]),
        );
    }

    pub fn set_texture(&mut self, texture: Arc<Texture>) {
        self.texture = Some(texture);
    }

    pub fn update(&mut self, data: RenderElementData, proxy: &GpuProxy) {
        self.data = data;
        self.write_all(proxy);
    }

    pub fn write_all(&self, proxy: &GpuProxy) {
        proxy.queue.write_buffer(
            &self.center_buffer,
            0,
            bytemuck::cast_slice(&self.data.center),
        );
        proxy
            .queue
            .write_buffer(&self.size_buffer, 0, bytemuck::cast_slice(&self.data.size));
        proxy.queue.write_buffer(
            &self.rotation_buffer,
            0,
            bytemuck::cast_slice(&[self.data.rotation]),
        );
        
        proxy.queue.write_buffer(
            &self.color.buffer,
            0,
            bytemuck::cast_slice(&[self.data.color]),
        );
    }

    pub fn bind(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn render(
        &self,
        pipelines: &Pipelines,
        pass: &mut wgpu::RenderPass,
    ) {
        if let Some(texture) = &self.texture {
            pass.set_pipeline(&pipelines.texture_pipeline);
            pass.set_bind_group(1, self.bind(), &[]);
            pass.set_bind_group(2, &texture.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
        pass.set_pipeline(&pipelines.color_pipeline);
        pass.set_bind_group(1, self.bind(), &[]);
        pass.set_bind_group(2, &self.color.bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}

impl Color {
    pub const BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };
}
