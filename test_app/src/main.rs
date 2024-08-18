use std::sync::Arc;

use rugui::{
    render::{Color, RadialGradient}, styles::{Position, Rotation, Size}, texture::Texture, Children, Element, ElementKey, Gui, Section
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
    card: ElementKey,
    drawing: Drawing,
    window: Arc<winit::window::Window>,
    t: f32,
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
        styles.transfomr_mut().min_width = Size::Pixel(500.0);
        styles.transfomr_mut().max_width = Size::Pixel(1600.0);

        let mut row1 = Element::new(&gui).with_label("Row 1".to_string());
        let mut row2 = Element::new(&gui).with_label("Row 2".to_string());
        let mut row3 = Element::new(&gui).with_label("Row 3".to_string());

        let row1_styles = &mut row1.styles;
        row1_styles.set_bg_texture(Some(texture.clone()));
        row1_styles.transfomr_mut().position = Position::Left;
        row1_styles.transfomr_mut().align = Position::Left;
        row1_styles.transfomr_mut().width = Size::Pixel(200.0);

        let mut columns = Element::new(&gui).with_label("Columns".to_string());
        let mut column1 = Element::new(&gui).with_label("Column 1".to_string());
        let mut column2 = Element::new(&gui).with_label("Column 2".to_string());
        let mut column3 = Element::new(&gui).with_label("Column 3".to_string());

        let column1_styles = &mut column1.styles;
        column1_styles.set_bg_lin_gradient(Some(Arc::new(gui.linear_gradient(
                            (Position::Top, Color::RED.with_alpha(0.3)),
                            (Position::Center, Color::TRANSPARENT),
                        ))));
        column1_styles.set_bg_texture(Some(texture.clone()));
        column1_styles.transfomr_mut().margin = Size::Percent(-5.0);

        let column2_styles = &mut column2.styles;
        // experimental radial gradient
        // this is a subject to change
        let grad = gui.radial_gradient(
            (Position::Center, Color::YELLOW),
            (Position::Top, Color::WHITE.with_alpha(0.0)),
        );
        column2_styles.set_bg_rad_gradient(Some(Arc::new(grad)));
        column2_styles.transfomr_mut().margin = Size::Percent(50.0);
        column2_styles.transfomr_mut().rotation = Rotation::Deg(90.0);

        let column3_styles = &mut column3.styles;
        column3_styles.set_bg_color(Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 0.5,
        });
        column3_styles.transfomr_mut().margin = Size::Percent(5.0);

        columns.children = Children::Columns {
            children: vec![
                Section {
                    element: gui.add_element(column1),
                    size: Size::Percent(70.0),
                },
                Section {
                    element: gui.add_element(column2),
                    size: Size::None,
                },
                Section {
                    element: gui.add_element(column3),
                    size: Size::None,
                },
            ],
            spacing: Size::Fill,
        };

        let row3_styles = &mut row3.styles;
        row3_styles.set_bg_color(Color {
            r: 0.6,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        });
        row3_styles.transfomr_mut().min_height = Size::Pixel(200.0);
        row3_styles.set_bg_texture(Some(texture.clone()));
        row3_styles.transfomr_mut().position = Position::BottomRight;

        let columns_styles = &mut columns.styles;
        columns_styles.set_bg_color(Color {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 0.3,
        });
        columns_styles.transfomr_mut().rotation = Rotation::Deg(-10.0);

        row2.children = Children::Element(gui.add_element(columns));

        let card = gui.add_element(row2);

        rows.children = Children::Rows {
            children: vec![
                Section {
                    element: gui.add_element(row1),
                    size: Size::None,
                },
                Section {
                    element: card.clone(),
                    size: Size::None,
                },
                Section {
                    element: gui.add_element(row3),
                    size: Size::None,
                },
            ],
            spacing: Size::Fill,
        };
        let entry = gui.add_element(rows);
        gui.set_entry(Some(entry));

        window.set_visible(true);
        let this = Application {
            gui,
            card,
            drawing,
            window,
            t: 0.0,
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
                this.gui.resize(this.drawing.size, &this.drawing.queue);
                this.t += 1.0;
                let card = this.gui.get_element_mut(this.card).unwrap();
                card.styles.transfomr_mut().rotation = Rotation::Deg(this.t);
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
