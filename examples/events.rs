use std::sync::Arc;

use examples_common::Drawing;
use rugui::{render::Color, styles::Size, Element, Gui};
use winit::application::ApplicationHandler;

extern crate examples_common;
extern crate pollster;
extern crate rugui;
extern crate rugui_winit_events;
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
    gui: Gui<Messages>,
    drawing: Drawing,
    window: Arc<winit::window::Window>,
    events: rugui_winit_events::State,
}

#[derive(Clone)]
pub enum Messages {
    BoxHover,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_title("Events example")
                        .with_visible(false),
                )
                .unwrap(),
        );

        let drawing = pollster::block_on(Drawing::new(window.clone()));
        window.set_visible(true);

        let size = window.inner_size();
        let mut gui = Gui::new(size.into(), &drawing.device, &drawing.queue);

        let mut element = Element::new().with_label("hello element");
        element
            .event_listeners
            .insert(rugui::events::EventTypes::MouseEnter, Messages::BoxHover);
        element
            .event_listeners
            .insert(rugui::events::EventTypes::MouseLeave, Messages::BoxHover);
        let styles = &mut element.styles;
        styles.transfomr_mut().max_width = Size::Percent(50.0);
        styles.transfomr_mut().max_height = Size::Percent(80.0);
        *styles.bg_color_mut() = Color::CYAN;
        let element_key = gui.add_element(element);
        gui.set_entry(Some(element_key));

        let events = rugui_winit_events::State::new();

        *self = App::App(Application {
            gui,
            drawing,
            window,
            events,
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

        if let Some(event) = this.events.event(&event) {
            this.gui.event(event);
        }

        while let Some(message) = this.gui.poll_event() {
            match message.msg {
                Messages::BoxHover => {
                    let element = this.gui.get_element_mut(message.key).unwrap();
                    match message.event_type {
                        rugui::events::EventTypes::MouseEnter => {
                            let styles = &mut element.styles;
                            *styles.bg_color_mut() = Color::RED.with_alpha(0.5);
                        }
                        rugui::events::EventTypes::MouseLeave => {
                            let styles = &mut element.styles;
                            *styles.bg_color_mut() = Color::RED;
                        }
                        _ => {}
                    }
                }
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
                this.gui
                    .resize(this.window.inner_size().into(), &this.drawing.queue);
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
