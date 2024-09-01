use std::sync::Arc;

use examples_common::Drawing;
use rugui::{
    styles::{Rotation, Round, Size, Color}, Children, Element, Gui
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
                        .with_title("Text example")
                        .with_visible(false),
                )
                .unwrap(),
        );

        let drawing = pollster::block_on(Drawing::new(window.clone()));
        window.set_visible(true);

        let size = window.inner_size();
        let mut gui = Gui::new(size.into(), &drawing.device, &drawing.queue);

        let mut element = Element::new().with_label("hello element");
        let styles = &mut element.styles;
        styles.transfomr_mut().max_width = Size::Percent(50.0);
        styles.transfomr_mut().max_height = Size::Percent(80.0);
        styles.transfomr_mut().rotation = Rotation::Deg(25.0);
        *styles.bg_color_mut() = Color::MAGENTA;

        styles.text_size_mut().0 = Size::Pixel(50.0);
        element.text_str("A rotated text looks pretty ugly, so maybe try to use it as little as possible.");

        let mut small_box = Element::new();
        small_box.text_str("This looks a lot better but still needs some improvements. I will look into that.👍");
        small_box.styles.transfomr_mut().rotation = Rotation::AbsNone;
        small_box.styles.transfomr_mut().margin = Size::Percent(40.0);
        small_box.styles.transfomr_mut().position_round = Round::Round;
        small_box.styles.transfomr_mut().scale_round = Round::Round;
        *small_box.styles.bg_color_mut() = Color::YELLOW;

        element.children = Children::Element(gui.add_element(small_box));

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
        this.window.request_redraw();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll)
    }
}
