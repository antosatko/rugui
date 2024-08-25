use std::sync::Arc;

use examples_common::Drawing;
use rugui::{
    render::Color,
    styles::{ColorPoint, LinearGradient, Position, RadialGradient, Size},
    texture::Texture,
    Element, Gui, Section,
};
use winit::application::ApplicationHandler;

extern crate examples_common;
extern crate pollster;
extern crate rugui;
extern crate wgpu;
extern crate winit;

fn main() {
    let mut app = App::Loading;

    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    event_loop.run_app(&mut app).unwrap();
}

pub enum App {
    Loading,
    App(Application),
}

pub struct Application {
    gui: Gui<()>,
    drawing: Drawing,
    window: Arc<winit::window::Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_title("Backgrounds example")
                        .with_visible(false),
                )
                .unwrap(),
        );

        let drawing = pollster::block_on(Drawing::new(window.clone()));
        window.set_visible(true);

        let size = window.inner_size();
        let mut gui = Gui::new(size.into(), drawing.device.clone(), drawing.queue.clone());

        let texture = Arc::new(rugui::texture::Texture::from_bytes(
            &drawing.device,
            &drawing.queue,
            include_bytes!("they.webp"),
            "they",
        ));

        let mut rows = Element::new().with_label("rows");
        let row1 = Element::new().with_label("row1");
        let mut column1 = Element::new().with_label("row1 column1");
        *column1.styles.bg_color_mut() = Color::RED;
        let mut column2 = Element::new().with_label("row1 column2");
        column2.styles.set_bg_texture(Some(texture.clone()));
        let row1 = row1.with_children(rugui::Children::Columns {
            children: vec![
                Section {
                    element: gui.add_element(column1),
                    size: Size::None,
                },
                Section {
                    element: gui.add_element(column2),
                    size: Size::None,
                },
            ],
            spacing: Size::None,
        });

        let row2 = Element::new().with_label("row2");
        let mut column1 = Element::new().with_label("row2 column1");
        column1.styles.set_bg_lin_gradient(Some(LinearGradient {
            p1: ColorPoint {
                position: Position::Left,
                color: Color::RED,
            },
            p2: ColorPoint {
                position: Position::Right,
                color: Color::GREEN,
            },
        }));
        let mut column2 = Element::new().with_label("row2 column2");
        column2.styles.set_bg_rad_gradient(Some(RadialGradient {
            p1: ColorPoint {
                position: Position::Center,
                color: Color::RED,
            },
            p2: ColorPoint {
                position: Position::Top,
                color: Color::GREEN,
            },
        }));
        let row2 = row2.with_children(rugui::Children::Columns {
            children: vec![
                Section {
                    element: gui.add_element(column1),
                    size: Size::None,
                },
                Section {
                    element: gui.add_element(column2),
                    size: Size::None,
                },
            ],
            spacing: Size::None,
        });

        rows.children = rugui::Children::Rows {
            children: vec![
                Section {
                    element: gui.add_element(row1),
                    size: Size::None,
                },
                Section {
                    element: gui.add_element(row2),
                    size: Size::None,
                },
            ],
            spacing: Size::None,
        };

        let entry = gui.add_element(rows);
        gui.set_entry(Some(entry));

        *self = App::App(Application {
            gui,
            drawing,
            window,
        });
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
                if size.width <= 0 || size.height <= 0 {
                    return;
                }
                examples_common::resize_event(&mut this.gui, &mut this.drawing, size.into());
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                this.gui.update();
                this.gui.prepare(&this.drawing.device, &this.drawing.queue);
                this.drawing.draw(&mut this.gui);
            }
            _ => {}
        }
    }
}
