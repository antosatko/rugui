use std::sync::Arc;

use nalgebra::Point2;
use wgpu::include_wgsl;

use crate::{
    styles::{LinearGradient, RadialGradient},
    texture::Texture, ElementTransform,
};

pub struct GpuBound {
    pub dimensions_buffer: wgpu::Buffer,
    pub dimensions_bind_group: wgpu::BindGroup,
    pub size: (u32, u32),
    pub pipelines: Pipelines,
}

#[derive(Debug)]
pub struct Pipelines {
    pub color_pipeline: wgpu::RenderPipeline,
    pub texture_pipeline: wgpu::RenderPipeline,
    pub radial_gradient_pipeline: wgpu::RenderPipeline,
    pub linear_gradient_pipeline: wgpu::RenderPipeline,
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

    pub fn new(queue: &wgpu::Queue, device: &wgpu::Device, size: (u32, u32)) -> Self {
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
        let radial_gradient_bind_group_layout =
            device.create_bind_group_layout(&RenderRadialGradient::LAYOUT);
        let linear_gradient_bind_group_layout =
            device.create_bind_group_layout(&RenderLinearGradient::LAYOUT);

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

        let radial_gradient_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Radial Gradient Pipeline Layout"),
                bind_group_layouts: &[
                    &dimensions_bind_group_layout,
                    &elements_bind_group_layout,
                    &radial_gradient_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let radial_gradient_shaders =
            device.create_shader_module(include_wgsl!("shaders/radial_grad.wgsl"));

        let radial_gradient_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Radial Gradient Pipeline"),
                layout: Some(&radial_gradient_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &radial_gradient_shaders,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &radial_gradient_shaders,
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

        let linear_gradient_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Linear Gradient Pipeline Layout"),
                bind_group_layouts: &[
                    &dimensions_bind_group_layout,
                    &elements_bind_group_layout,
                    &linear_gradient_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let linear_gradient_shaders =
            device.create_shader_module(include_wgsl!("shaders/linear_grad.wgsl"));

        let linear_gradient_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Linear Gradient Pipeline"),
                layout: Some(&linear_gradient_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &linear_gradient_shaders,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &linear_gradient_shaders,
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
            dimensions_buffer,
            dimensions_bind_group,
            size,
            pipelines: Pipelines {
                color_pipeline,
                texture_pipeline,
                radial_gradient_pipeline,
                linear_gradient_pipeline,
            },
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

pub struct RenderRadialGradient {
    pub center_color: Color,
    pub center: [f32; 2],
    pub radius: f32,
    pub outer_color: Color,
    pub bind_group: wgpu::BindGroup,
    pub center_color_buffer: wgpu::Buffer,
    pub center_buffer: wgpu::Buffer,
    pub radius_buffer: wgpu::Buffer,
    pub outer_color_buffer: wgpu::Buffer,
}

impl RenderRadialGradient {
    pub const LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        label: Some("Radial Gradient Bind Group Layout"),
        entries: &[
            // Center Color
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Center
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Radius
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Outer Color
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    };

    pub fn zeroed(device: &wgpu::Device) -> Self {
        let center_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Center Color Buffer"),
            size: std::mem::size_of::<Color>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let center_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Center Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let radius_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Radius Buffer"),
            size: std::mem::size_of::<[f32; 1]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let outer_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Outer Color Buffer"),
            size: std::mem::size_of::<Color>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&Self::LAYOUT);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radial Gradient Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &center_color_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &center_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &radius_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &outer_color_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        Self {
            center_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            center: [0.0, 0.0],
            radius: 0.0,
            outer_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            bind_group,
            center_color_buffer,
            center_buffer,
            radius_buffer,
            outer_color_buffer,
        }
    }

    pub fn set_center_color(&mut self, color: Color, queue: &wgpu::Queue) {
        self.center_color = color;

        queue.write_buffer(&self.center_color_buffer, 0, bytemuck::cast_slice(&[color]));
    }

    pub fn set_center(&mut self, center: [f32; 2], queue: &wgpu::Queue) {
        self.center = center;

        queue.write_buffer(&self.center_buffer, 0, bytemuck::cast_slice(&[center]));
    }

    pub fn set_radius(&mut self, radius: f32, queue: &wgpu::Queue) {
        self.radius = radius;

        queue.write_buffer(&self.radius_buffer, 0, bytemuck::cast_slice(&[radius]));
    }

    pub fn set_outer_color(&mut self, color: Color, queue: &wgpu::Queue) {
        self.outer_color = color;

        queue.write_buffer(&self.outer_color_buffer, 0, bytemuck::cast_slice(&[color]));
    }

    pub fn bind(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn write_all(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.center_color_buffer,
            0,
            bytemuck::cast_slice(&[self.center_color]),
        );

        queue.write_buffer(&self.center_buffer, 0, bytemuck::cast_slice(&[self.center]));

        queue.write_buffer(&self.radius_buffer, 0, bytemuck::cast_slice(&[self.radius]));
        queue.write_buffer(
            &self.outer_color_buffer,
            0,
            bytemuck::cast_slice(&[self.outer_color]),
        );
    }

    pub fn from_style(&mut self, style: &RadialGradient, transform: &ElementTransform) {
        self.center = style.center.position.normalized(transform.scale);
        self.center_color = style.center.color;
        self.radius = {
            let p1 = self.center;
            let p2 = style.radius.position.normalized(transform.scale);
            let a = p2[0] - p1[0];
            let b = p2[1] - p1[1];
            f32::sqrt(a * a + b * b)
        };
        self.outer_color = style.radius.color;
    }
}

pub struct RenderLinearGradient {
    pub start_color: Color,
    pub end_color: Color,
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub bind_group: wgpu::BindGroup,
    pub start_color_buffer: wgpu::Buffer,
    pub end_color_buffer: wgpu::Buffer,
    pub start_buffer: wgpu::Buffer,
    pub end_buffer: wgpu::Buffer,
}

impl RenderLinearGradient {
    pub const LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        label: Some("Linear Gradient Bind Group Layout"),
        entries: &[
            // Start Color
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // End Color
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Start
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // End
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    };

    pub fn zeroed(device: &wgpu::Device) -> Self {
        let start_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Start Color Buffer"),
            size: std::mem::size_of::<Color>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let end_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("End Color Buffer"),
            size: std::mem::size_of::<Color>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let start_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Start Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let end_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("End Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&Self::LAYOUT);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Linear Gradient Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &start_color_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &end_color_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &start_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &end_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        Self {
            start_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            end_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            start: [0.0, 0.0],
            end: [0.0, 0.0],
            bind_group,
            start_color_buffer,
            end_color_buffer,
            start_buffer,
            end_buffer,
        }
    }

    pub fn set_start_color(&mut self, color: Color, queue: &wgpu::Queue) {
        self.start_color = color;

        queue.write_buffer(&self.start_color_buffer, 0, bytemuck::cast_slice(&[color]));
    }

    pub fn set_end_color(&mut self, color: Color, queue: &wgpu::Queue) {
        self.end_color = color;

        queue.write_buffer(&self.end_color_buffer, 0, bytemuck::cast_slice(&[color]));
    }

    pub fn set_start(&mut self, start: [f32; 2], queue: &wgpu::Queue) {
        self.start = start;

        queue.write_buffer(&self.start_buffer, 0, bytemuck::cast_slice(&[start]));
    }

    pub fn set_end(&mut self, end: [f32; 2], queue: &wgpu::Queue) {
        self.end = end;

        queue.write_buffer(&self.end_buffer, 0, bytemuck::cast_slice(&[end]));
    }

    pub fn bind(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn write_all(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.start_color_buffer,
            0,
            bytemuck::cast_slice(&[self.start_color]),
        );
        queue.write_buffer(
            &self.end_color_buffer,
            0,
            bytemuck::cast_slice(&[self.end_color]),
        );

        queue.write_buffer(&self.start_buffer, 0, bytemuck::cast_slice(&[self.start]));

        queue.write_buffer(&self.end_buffer, 0, bytemuck::cast_slice(&[self.end]));
    }

    pub fn from_style(&mut self, style: &LinearGradient, transform: &ElementTransform) {
        self.start = style.p1.position.normalized(transform.scale);
        self.start_color = style.p1.color;
        self.end = style.p2.position.normalized(transform.scale);
        self.end_color = style.p2.color;
    }
}

#[derive(Debug, Clone, Copy, Default)]
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
    pub center_buffer: wgpu::Buffer,
    pub size_buffer: wgpu::Buffer,
    pub rotation_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub color: Option<RenderColor>,
    pub texture: Option<Arc<Texture>>,
    pub radial_gradient: Option<RenderRadialGradient>,
    pub linear_gradient: Option<RenderLinearGradient>,
    pub text: Option<Arc<Texture>>,
}

pub struct RenderColor {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl RenderColor {
    pub const BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Color Bind Group Layout"),
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

    pub fn uninit(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Render Color Buffer"),
            size: std::mem::size_of::<Color>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Color Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self { buffer, bind_group }
    }

    pub fn set_color(&self, color: Color, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[color]));
    }

    pub fn bind(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
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

    pub const ZEROED: Self = Self {
        center: [0.0, 0.0],
        size: [0.0, 0.0],
        rotation: 0.0,
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        },
    };

    pub fn from_transform(transform: &crate::ElementTransform) -> Self {
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

    pub fn update_transform(&mut self, transform: &crate::ElementTransform) {
        self.center = [transform.position.x, transform.position.y];
        self.size = [transform.scale.x, transform.scale.y];
        self.rotation = transform.rotation;
    }

    pub fn point_collision(&self, point: Point2<f32>) -> bool {
        let rotated_point = Point2::new(
            (point.x - self.center[0]) * self.rotation.cos()
                - (point.y - self.center[1]) * self.rotation.sin(),
            (point.x - self.center[0]) * self.rotation.sin()
                + (point.y - self.center[1]) * self.rotation.cos(),
        );

        let (width, height) = (self.size[0] / 2.0, self.size[1] / 2.0);

        rotated_point.x >= -width
            && rotated_point.x <= width
            && rotated_point.y >= -height
            && rotated_point.y <= height
    }
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

    pub fn zeroed(device: &wgpu::Device) -> Self {
        let center_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Center Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let size_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Size Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rotation_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rotation Buffer"),
            size: std::mem::size_of::<[f32; 1]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&Self::LAYOUT);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        Self {
            data: RenderElementData::ZEROED,
            center_buffer,
            size_buffer,
            rotation_buffer,
            bind_group,
            color: None,
            texture: None,
            radial_gradient: None,
            linear_gradient: None,
            text: None,
        }
    }

    pub fn set_transform(&mut self, transform: &crate::ElementTransform, queue: &wgpu::Queue) {
        self.data.update_transform(transform);
        self.write_all(queue);
    }

    pub fn set_color(&mut self, color: Color, queue: &wgpu::Queue, device: &wgpu::Device) {
        match &mut self.color {
            Some(render_color) => {
                render_color.set_color(color, queue);
            }
            None => {
                let render_color = RenderColor::uninit(device);
                render_color.set_color(color, queue);
                self.color = Some(render_color);
            }
        }
    }

    pub fn set_texture(&mut self, texture: Arc<Texture>) {
        self.texture = Some(texture);
    }

    pub fn set_text(&mut self, text: Arc<Texture>) {
        self.text = Some(text);
    }

    pub fn update(&mut self, data: RenderElementData, queue: &wgpu::Queue) {
        self.data = data;
        self.write_all(queue);
    }

    pub fn write_all(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.center_buffer,
            0,
            bytemuck::cast_slice(&self.data.center),
        );
        queue.write_buffer(&self.size_buffer, 0, bytemuck::cast_slice(&self.data.size));
        queue.write_buffer(
            &self.rotation_buffer,
            0,
            bytemuck::cast_slice(&[self.data.rotation]),
        );
    }

    pub fn bind(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn render(&self, pipelines: &Pipelines, pass: &mut wgpu::RenderPass) {
        if self.color.is_none()
            && self.texture.is_none()
            && self.radial_gradient.is_none()
            && self.linear_gradient.is_none()
        {
            return;
        } else {
            pass.set_bind_group(1, self.bind(), &[]);
        }
        if let Some(texture) = &self.texture {
            Self::draw_command(&pipelines.texture_pipeline, pass, &texture.bind_group);
        }
        if let Some(radial_gradient) = &self.radial_gradient {
            Self::draw_command(
                &pipelines.radial_gradient_pipeline,
                pass,
                &radial_gradient.bind_group,
            );
        }
        if let Some(linear_gradient) = &self.linear_gradient {
            Self::draw_command(
                &pipelines.linear_gradient_pipeline,
                pass,
                &linear_gradient.bind_group,
            );
        }
        if let Some(render_color) = &self.color {
            Self::draw_command(&pipelines.color_pipeline, pass, render_color.bind());
        }
        if let Some(texture) = &self.text {
            Self::draw_command(&pipelines.texture_pipeline, pass, &texture.bind_group);
        }
    }

    fn draw_command(
        pipeline: &wgpu::RenderPipeline,
        pass: &mut wgpu::RenderPass,
        bind_group: &wgpu::BindGroup,
    ) {
        pass.set_pipeline(pipeline);
        pass.set_bind_group(2, bind_group, &[]);
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

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const YELLOW: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const CYAN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const MAGENTA: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn with_alpha(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    pub fn with_red(&self, r: f32) -> Self {
        Self {
            r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }

    pub fn with_green(&self, g: f32) -> Self {
        Self {
            r: self.r,
            g,
            b: self.b,
            a: self.a,
        }
    }

    pub fn with_blue(&self, b: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b,
            a: self.a,
        }
    }
}

impl From<[f32; 4]> for Color {
    fn from(array: [f32; 4]) -> Self {
        Self {
            r: array[0],
            g: array[1],
            b: array[2],
            a: array[3],
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> [f32; 4] {
        [color.r, color.g, color.b, color.a]
    }
}
