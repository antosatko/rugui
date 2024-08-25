use std::sync::Arc;

use examples_common::Drawing;
use rugui::{
    render::Color,
    styles::{Rotation, Size},
    Children, Element, Gui, Section,
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
                        .with_title("Children example")
                        .with_visible(false),
                )
                .unwrap(),
        );

        let drawing = pollster::block_on(Drawing::new(window.clone()));
        window.set_visible(true);

        let size = window.inner_size();
        let mut gui = Gui::new(size.into(), drawing.device.as_ref(), drawing.queue.as_ref());

        let mut bg = Element::new().with_label("hello element");
        let bg_styles = &mut bg.styles;
        bg_styles.transfomr_mut().margin = Size::Percent(10.0);
        *bg_styles.bg_color_mut() = Color::CYAN;

        let mut gray_screen = Element::new().with_label("gray screen");
        let gray_styles = &mut gray_screen.styles;
        gray_styles.transfomr_mut().margin = Size::Percent(25.0);
        *gray_styles.bg_color_mut() = Color::BLACK.with_alpha(0.8);

        let mut rows = Element::new().with_label("rows");
        let mut row1 = Element::new().with_label("row1");
        let mut row2 = Element::new().with_label("row2");
        let row1_styles = &mut row1.styles;
        row1_styles.transfomr_mut().rotation = Rotation::Percent(90.0);
        *row1_styles.bg_color_mut() = Color::RED;
        let row2_styles = &mut row2.styles;
        *row2_styles.bg_color_mut() = Color::GREEN;
        rows.children = Children::Rows {
            children: vec![
                Section {
                    element: gui.add_element(row1),
                    size: Size::Percent(25.0),
                },
                Section {
                    element: gui.add_element(row2),
                    size: Size::Percent(50.0),
                },
            ],
            spacing: Size::None,
        };

        let mut columns = Element::new().with_label("columns");
        let mut column1 = Element::new().with_label("column1");
        let mut column2 = Element::new().with_label("column2");
        let mut column3 = Element::new().with_label("column3");
        column3.children = Children::Element(gui.add_element(rows));
        let column1_styles = &mut column1.styles;
        column1_styles.transfomr_mut().margin = Size::Percent(10.0);
        *column1_styles.bg_color_mut() = Color::RED;
        let column2_styles = &mut column2.styles;
        *column2_styles.bg_color_mut() = Color::GREEN;
        let column3_styles = &mut column3.styles;
        *column3_styles.bg_color_mut() = Color::BLUE;
        columns.children = Children::Columns {
            children: vec![
                Section {
                    element: gui.add_element(column1),
                    size: Size::Percent(25.0),
                },
                Section {
                    element: gui.add_element(column2),
                    size: Size::Percent(50.0),
                },
                Section {
                    element: gui.add_element(column3),
                    size: Size::None,
                },
            ],
            spacing: Size::None,
        };

        bg.children =
            Children::Layers(vec![gui.add_element(columns), gui.add_element(gray_screen)]);
        let bg_key = gui.add_element(bg);
        gui.set_entry(Some(bg_key));

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
