use std::sync::Arc;

use examples_common::Drawing;
use rugui::{
    render::{Color, RenderRadialGradient},
    styles::{ColorPoint, LinearGradient, Position, RadialGradient, Rotation, Size},
    texture::Texture,
    Children, Element, ElementKey, Gui, Section,
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
    start: std::time::Instant,
    events: rugui_winit_events::State,
    rotation: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Idk,
    Card,
}

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

        let texture = Arc::new(rugui::texture::Texture::from_bytes(
            &drawing.device,
            &drawing.queue,
            include_bytes!("they.webp"),
            "they",
        ));

        let mut rows = Element::new().with_label("Hello");
        let styles = &mut rows.styles;
        styles.transfomr_mut().min_width = Size::Pixel(500.0);
        styles.transfomr_mut().max_width = Size::Pixel(1600.0);

        let mut row1 = Element::new().with_label("Row 1");
        let mut row2 = Element::new().with_label("Row 2");
        let mut row3 = Element::new().with_label("Row 3");

        let row1_styles = &mut row1.styles;
        row1_styles.set_bg_texture(Some(texture.clone()));
        row1_styles.transfomr_mut().position = Position::Left;
        row1_styles.transfomr_mut().align = Position::Left;
        row1_styles.transfomr_mut().width = Size::Pixel(200.0);

        let mut columns = Element::new().with_label("Columns");
        columns
            .event_listeners
            .insert(rugui::events::EventTypes::MouseEnter, Message::Card);
        columns
            .event_listeners
            .insert(rugui::events::EventTypes::MouseLeave, Message::Card);
        let mut column1 = Element::new().with_label("Column 1");
        let mut column2 = Element::new().with_label("Column 2");
        let mut column3 = Element::new().with_label("Column 3");

        let column1_styles = &mut column1.styles;
        column1_styles.transfomr_mut().rotation = Rotation::None;
        column1_styles.set_bg_lin_gradient(Some(LinearGradient {
            p1: ColorPoint {
                position: Position::Top,
                color: Color::RED.with_alpha(0.3),
            },
            p2: ColorPoint {
                position: Position::Center,
                color: Color::TRANSPARENT,
            },
        }));
        column1_styles.set_bg_texture(Some(texture.clone()));
        column1_styles.transfomr_mut().margin = Size::Percent(-5.0);

        let column2_styles = &mut column2.styles;
        column2_styles.transfomr_mut().rotation = Rotation::None;
        column2_styles.set_bg_rad_gradient(Some(RadialGradient {
            p1: ColorPoint {
                position: Position::Center,
                color: Color::YELLOW,
            },
            p2: ColorPoint {
                position: Position::Top,
                color: Color::WHITE.with_alpha(0.0),
            },
        }));
        column2_styles.transfomr_mut().margin = Size::Percent(50.0);

        let column3_styles = &mut column3.styles;
        column3_styles.transfomr_mut().rotation = Rotation::None;
        *column3_styles.bg_color_mut() = Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 0.5,
        };
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
        *row3_styles.bg_color_mut() = Color {
            r: 0.6,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        };
        row3_styles.transfomr_mut().min_height = Size::Pixel(200.0);
        row3_styles.set_bg_texture(Some(texture.clone()));
        row3_styles.transfomr_mut().position = Position::BottomRight;

        let columns_styles = &mut columns.styles;
        *columns_styles.bg_color_mut() = Color {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 0.3,
        };
        columns_styles.transfomr_mut().rotation = Rotation::None;

        let card = gui.add_element(columns);
        row2.children = Children::Element(card);

        gui.add_element(row2);

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
            start: std::time::Instant::now(),
            events: rugui_winit_events::State::new(),
            rotation: false,
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

        match this.events.event(&event) {
            Some(event) => {
                this.gui.event(event);
            }
            None => (),
        }

        while let Some(event) = this.gui.poll_event() {
            match event.msg {
                Message::Card => match event.event_type {
                    rugui::events::EventTypes::MouseEnter => {
                        this.rotation = false;
                    }
                    rugui::events::EventTypes::MouseLeave => {
                        this.rotation = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        match event {
            winit::event::WindowEvent::Resized(size) => {
                examples_common::resize_event(&mut this.gui, &mut this.drawing, size.into());
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let card = this.gui.get_element_mut(this.card).unwrap();
                if this.rotation {
                    this.t += 0.1;
                    card.styles.transfomr_mut().rotation = Rotation::Deg(this.t);
                }
                this.gui.update();
                this.gui.prepare(&this.drawing.device, &this.drawing.queue);
                this.drawing.draw(&mut this.gui);
            }
            _ => {}
        }
        this.window.request_redraw();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    }
}
