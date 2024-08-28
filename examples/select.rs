

use std::sync::Arc;

use examples_common::Drawing;
use rugui::{styles::{Size, Color}, Children, Element, Gui, Section};
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

fn create_select(label: &str) -> Element<()> {
    let mut element = Element::new().with_label(label);
    element
        .events
        .listen(rugui::events::EventTypes::Select, ());
    element.events.listen(rugui::events::EventTypes::Input, ());
    element.styles.selectable = true;

    element
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_title("Select example")
                        .with_visible(false),
                )
                .unwrap(),
        );

        let drawing = pollster::block_on(Drawing::new(window.clone()));
        window.set_visible(true);

        let size = window.inner_size();
        let mut gui = Gui::new(size.into(), &drawing.device, &drawing.queue);

        let rows = Element::new().with_label("rows");
        let mut row1 = create_select("row1");
        row1.text_str("Good job!");
        row1.styles.resize_text(50.0);
        let mut row2 = Element::new().with_label("row2");
        row2.text_str("Try pressing Tab..");
        row2.styles.text_mut().color = Color::WHITE;
        row2.styles.resize_text(50.0);
        let mut row3 = create_select("row3");
        row3.text_str("Write something: ");
        row3.styles.resize_text(50.0);
        let row4 = create_select("row4");
        let mut row5 = Element::new().with_label("row5");
        {
            let children = Vec::from([
                Section {
                    element: gui.add_element(create_select("column1")),
                    size: Size::None,
                },
                Section {
                    element: gui.add_element(create_select("column2")),
                    size: Size::None,
                },
                Section {
                    element: gui.add_element(create_select("column3")),
                    size: Size::None,
                }
            ]);
            row5.children = Children::Columns { children, spacing: Size::None }
        }

        let children = Vec::from([
            Section {
                size: Size::None,
                element: gui.add_element(row1),
            },
            Section {
                size: Size::None,
                element: gui.add_element(row2),
            },
            Section {
                size: Size::None,
                element: gui.add_element(row3),
            },
            Section {
                size: Size::None,
                element: gui.add_element(row4),
            },
            Section {
                size: Size::None,
                element: gui.add_element(row5),
            },
        ]);
        let key = gui.add_element(rows.with_children(rugui::Children::Rows {
            children,
            spacing: Size::None,
        }));
        gui.set_entry(Some(key));

        *self = App::App(Application {
            gui,
            drawing,
            window,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let this = match self {
            App::App(this) => this,
            App::Loading => return,
        };

        this.gui.update();
        rugui::winit::event(&mut this.gui, &event);


        while let Some(event) = this.gui.poll_event() {
            match event.element_event {
                rugui::events::ElementEvent::Select => {
                    if let Some(element) = this.gui.get_element_mut(event.key) {
                        *element.styles.bg_color_mut() = Color::RED;
                    }
                    this.window.request_redraw();
                }
                rugui::events::ElementEvent::Unselect => {
                    if let Some(element) = this.gui.get_element_mut(event.key) {
                        *element.styles.bg_color_mut() = Color::BLACK;
                        element.text_str("")
                    }
                    this.window.request_redraw();
                }
                rugui::events::ElementEvent::Input { text } => {
                    if let Some(element) = this.gui.get_element_mut(event.key) {
                        let mut field = element.text().clone().unwrap_or(&"".to_string()).to_string();
                        field.push_str(&text);
                        element.text_string(field);
                        this.window.request_redraw();
                    }
                }
                _ => {}
            }
        }

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
