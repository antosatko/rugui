use std::sync::Arc;

use examples_common::Drawing;
use rugui::{
    styles::{
        ColorPoint, Colors, Position, RValue, RadialGradient, Side,
        Value, Values,
    },
    Element, ElementKey, Gui,
};
use winit::{application::ApplicationHandler, window::Window};

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
    element: ElementKey,
    t: f32,
    window: Arc<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_title("Hello example")
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
        //styles.bg_color.set(Colors::CYAN.with_alpha(0.5));
        styles
            .width
            .set(Values::Value(Value::Container(RValue::Full, Side::Min)));
        styles
            .height
            .set(Values::Value(Value::Container(RValue::Full, Side::Min)));
        styles.edges_radius.set(Values::Value(Value::Container(RValue::Half, Side::Width)));
        styles.edges_smooth.set(Values::Value(Value::Pixel(2.0)));
        //styles.bg_texture.set(Some(Arc::new(texture)));
        /*styles.bg_linear_gradient.set(Some(LinearGradient{
            p1: ColorPoint { position: Position::default(), color: Colors::WHITE },
            p2: ColorPoint { position: Position{
                value: PositionValues::TopLeft,
                ..Default::default()
            }, color: Colors::BLACK },
        }));*/
        styles.bg_radial_gradient.set(Some(RadialGradient {
            center: ColorPoint {
                position: Position::CCENTER.with_offset_x(Some(Values::Value(Value::Pixel(0.0)))),
                color: Colors::WHITE,
            },
            outer: ColorPoint {
                position: Position::CLEFT,
                color: Colors::BLUE,
            },
        }));

        element.text_str("Hello world!");
        let element_key = gui.add_element(element);
        gui.set_entry(Some(element_key));

        *self = App::App(Application {
            gui,
            drawing,
            element: element_key,
            t: 0.0,
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
                this.t += 2.0;
                let element = this.gui.get_element_mut(this.element).unwrap();
                element.styles.text_color.set(Colors::Hsl(this.t, 1.0, 0.5));
                this.gui.update();
                this.gui.prepare(&this.drawing.device, &this.drawing.queue);
                this.drawing.draw(&mut this.gui);
                this.window.request_redraw();
            }
            _ => {}
        }
    }
}
