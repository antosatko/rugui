use std::sync::Arc;

use gui::{render::Color, styles::Size, texture::Texture, Element, Gui};
use winit::{application::ApplicationHandler, window};

fn main() {
    App::run();
}

enum App {
    Loading,
    App(Application),
}

impl App {
    fn run() {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let mut app = App::Loading;
        event_loop.run_app(&mut app).unwrap();
    }
}

struct Application {
    gui: Gui<Message>,
    drawing: Drawing,
    window: Arc<winit::window::Window>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    window::WindowAttributes::default()
                        .with_title("GUI Test")
                        .with_visible(false),
                )
                .unwrap(),
        );
        let drawing = pollster::block_on(Drawing::new(window.clone()));
        let mut gui: Gui<Message> = Gui::new((800, 600), drawing.device.clone(), drawing.queue.clone());

        let texture = gui.texture_from_bytes(include_bytes!("they.webp"), "sdf");

        let mut elem = Element::new(&gui).with_label("Hello".to_string());
        let styles = &mut elem.styles;
        styles.background.color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.6,
            a: 0.5,
        };
        styles.min_width = Size::Pixel(500.0);
        styles.max_width = Size::Pixel(1200.0);
        styles.background.texture = Some(Arc::new(texture));
        let entry = gui.add_element(elem);
        gui.set_entry(Some(entry));


        window.set_visible(true);
        let this = Application {
            gui,
            drawing,
            window,
        };
        *self = App::App(this);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let this = match self {
            App::App(this) => this,
            App::Loading => return,
        };

        match event {
            winit::event::WindowEvent::Resized(size) => {
                this.gui
                    .resize((size.width, size.height), &this.drawing.queue);

                this.drawing.config.width = size.width;
                this.drawing.config.height = size.height;
                this.drawing.size = (size.width, size.height);
                this.drawing
                    .surface
                    .configure(&this.drawing.device, &this.drawing.config);
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                this.drawing.draw(&mut this.gui);
            }
            _ => {}
        }
        this.window.request_redraw();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    }
}

struct Drawing {
    config: wgpu::SurfaceConfiguration,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    window: Arc<winit::window::Window>,
    size: (u32, u32),
}

impl Drawing {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
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
                    memory_hints: wgpu::MemoryHints::Performance,
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

    fn draw(&mut self, gui: &mut Gui<Message>) {
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
