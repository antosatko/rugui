use std::sync::Arc;

use rugui::{
    render::{Color, RadialGradient},
    styles::{Position, Rotation, Size},
    texture::Texture,
    Children, Element, Gui, Spacing,
};
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
        let mut gui: Gui<Message> =
            Gui::new((800, 600), drawing.device.clone(), drawing.queue.clone());
        gui.debug = true;

        let texture = gui.texture_from_bytes(include_bytes!("they.webp"), "sdf");

        let mut rows = Element::new(&gui).with_label("Hello".to_string());
        let styles = &mut rows.styles;
        styles.min_width = Size::Pixel(500.0);
        styles.max_width = Size::Pixel(1600.0);

        let mut row1 = Element::new(&gui).with_label("Row 1".to_string());
        let mut row2 = Element::new(&gui).with_label("Row 2".to_string());
        let mut row3 = Element::new(&gui).with_label("Row 3".to_string());

        let row1_styles = &mut row1.styles;
        row1_styles.background.texture = Some(texture.clone());
        row1_styles.position = Position::Left;
        row1_styles.align = Position::Left;
        row1_styles.width = Size::Pixel(200.0);

        let row2_styles = &mut row2.styles;
        row2_styles.background.color = Color {
            r: 0.0,
            g: 0.6,
            b: 0.0,
            a: 0.5,
        };

        let mut columns = Element::new(&gui).with_label("Columns".to_string());
        let mut column1 = Element::new(&gui).with_label("Column 1".to_string());
        let mut column2 = Element::new(&gui).with_label("Column 2".to_string());
        let mut column3 = Element::new(&gui).with_label("Column 3".to_string());

        let column1_styles = &mut column1.styles;
        column1_styles.background.lin_gradient = Some(Arc::new(gui.linear_gradient(
            (Position::Top, Color::RED.with_alpha(0.3)),
            (Position::Center, Color::TRANSPARENT),
        )));
        column1_styles.background.texture = Some(texture.clone());
        column1_styles.margin = Size::Percent(-5.0);

        let column2_styles = &mut column2.styles;
        // experimental radial gradient
        // this is a subject to change
        let grad = gui.radial_gradient(
            (Position::Center, Color::BLUE),
            (Position::TopLeft, Color::MAGENTA),
        );
        column2_styles.background.rad_gradient = Some(Arc::new(grad));
        column2_styles.margin = Size::Percent(50.0);

        let column3_styles = &mut column3.styles;
        column3_styles.background.color = Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 0.5,
        };
        column3_styles.margin = Size::Percent(5.0);

        columns.children = Children::Columns {
            children: vec![
                Spacing {
                    element: gui.add_element(column1),
                    spacing: Size::Percent(70.0),
                },
                Spacing {
                    element: gui.add_element(column2),
                    spacing: Size::None,
                },
                Spacing {
                    element: gui.add_element(column3),
                    spacing: Size::None,
                },
            ],
            spacing: Size::Fill,
        };

        let row3_styles = &mut row3.styles;
        row3_styles.background.color = Color {
            r: 0.6,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        };
        row3_styles.min_height = Size::Pixel(200.0);
        row3_styles.background.texture = Some(texture.clone());
        row3_styles.position = Position::BottomRight;

        row2.children = Children::Element(gui.add_element(columns));

        rows.children = Children::Rows {
            children: vec![
                Spacing {
                    element: gui.add_element(row1),
                    spacing: Size::None,
                },
                Spacing {
                    element: gui.add_element(row2),
                    spacing: Size::None,
                },
                Spacing {
                    element: gui.add_element(row3),
                    spacing: Size::None,
                },
            ],
            spacing: Size::Fill,
        };
        let entry = gui.add_element(rows);
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
