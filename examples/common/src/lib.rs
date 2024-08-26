use std::sync::Arc;

use rugui::Gui;


pub struct Drawing {
    pub config: wgpu::SurfaceConfiguration,
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub window: Arc<winit::window::Window>,
    pub size: (u32, u32),
}

impl Drawing {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 1,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            config,
            instance,
            surface,
            device: Arc::new(device),
            queue: Arc::new(queue),
            window,
            size: (size.width, size.height),
        }
    }

    pub fn draw<Message: Clone>(&mut self, gui: &mut Gui<Message>) {
        if self.size.0 == 0 || self.size.1 == 0 {
            return;
        }
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            gui.render(&mut pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

pub fn resize_event<Msg: Clone>(gui: &mut Gui<Msg>, drawing: &mut Drawing, size: (u32, u32)) {
    drawing.config.width = size.0;
    drawing.config.height = size.1;
    drawing.size = (size.0, size.1);
    if size.0 == 0 || size.1 == 0 {
        return;
    }
    gui.resize(size, &drawing.queue);
    drawing
        .surface
        .configure(&drawing.device, &drawing.config);
}