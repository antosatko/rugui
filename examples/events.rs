use std::sync::Arc;

use examples_common::Drawing;
use rugui::{events::ElementEvent, styles::{Position, RadialGradient, Size, Color}, Element, Gui};
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
    gui: Gui<Messages>,
    drawing: Drawing,
    window: Arc<winit::window::Window>,
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
            .events
            .listen(rugui::events::EventTypes::MouseMove, Messages::BoxHover);
        let styles = &mut element.styles;
        styles.set_bg_rad_gradient(Some(RadialGradient {
            center: rugui::styles::ColorPoint { position: rugui::styles::Position::Center, color: Color::BLACK },
            radius: rugui::styles::ColorPoint { position: rugui::styles::Position::Top, color: Color::RED.with_alpha(0.0) }
        }));
        let element_key = gui.add_element(element);
        gui.set_entry(Some(element_key));

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

        rugui::winit::event(&mut this.gui, &event);

        while let Some(message) = this.gui.poll_event() {
            match message.msg {
                Messages::BoxHover => {
                    let element = this.gui.get_element_mut(message.key).unwrap();
                    match message.element_event {
                        ElementEvent::MouseMove { position, .. } => {
                            let styles = &mut element.styles;
                            let mut grad = styles.get_bg_rad_gradient().unwrap();
                            grad.center.position = Position::Custom(Size::Pixel(position.x), Size::Pixel(position.y));
                            styles.set_bg_rad_gradient(Some(grad));
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
