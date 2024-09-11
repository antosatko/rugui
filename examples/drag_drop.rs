use std::{collections::HashMap, sync::Arc};

use examples_common::Drawing;
use rugui::{events::{EventTypes, WindowEvent}, styles::{Color, Position, Round, Side, Size}, Children, Element, ElementKey, Gui, Section};
use winit::{application::ApplicationHandler, window::CursorIcon};

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
    drag: Option<(Option<String>, ElementKey)>,
    drag_element: ElementKey,
    hovering: Option<ElementKey>
}

#[derive(Clone)]
pub enum Messages {
    BoxHover,
    BoxDrag,
    BoxDrop,
    MouseMove,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_title("Drag and drop example")
                        .with_visible(false),
                )
                .unwrap(),
        );

        let drawing = pollster::block_on(Drawing::new(window.clone()));
        window.set_visible(true);

        let size = window.inner_size();
        let mut gui = Gui::new(size.into(), &drawing.device, &drawing.queue);

        let points = HashMap::from([
            ((0, 3), String::from("Hope")),
            ((2, 7), String::from("This")),
            ((1, 2), String::from("Makes")),
            ((3, 5), String::from("Some")),
            ((8, 8), String::from("Sense")),
        ]);
        
        let mut entry = Element::new().with_label("Entry");
        entry.events.force(EventTypes::MouseUp, Messages::BoxDrop);
        entry.events.force(EventTypes::MouseMove, Messages::MouseMove);
        entry.styles.transfomr_mut().margin = Size::Percent(10.0);
        *entry.styles.bg_color_mut() = Color::GRAY;
        let mut rows = Element::new().with_label("Rows");
        let mut rows_children = Vec::new();
        for i in 0..10 {
            let mut columns = Element::new().with_label(&format!("Row {i}"));
            let mut col_children = Vec::new();
            for j in 0..10 {
                let mut columns = Element::new().with_label(&format!("Column {i},{j}"));
                columns.events.listen(EventTypes::MouseEnter, Messages::BoxHover);
                columns.events.listen(EventTypes::MouseLeave, Messages::BoxHover);
                columns.events.listen(EventTypes::MouseDown, Messages::BoxDrag);
                let styles = &mut columns.styles;
                *styles.bg_color_mut() = Color::WHITE.with_blue(i as f32 /10.).with_green(j as f32 /10.);
                styles.edges_mut().radius = (Size::Pixel(10.0), Side::Height);
                styles.transfomr_mut().margin = Size::Percent(10.0);
                styles.transfomr_mut().position_round = Round::Round;
                styles.transfomr_mut().scale_round = Round::Round;
                styles.text_mut().size = (Size::Percent(50.0), Side::Min);
                if let Some(str) = points.get(&(i, j)) {
                    columns.text_str(str);
                }
                col_children.push(Section { element: gui.add_element(columns), size: Size::None })
            }
            columns.children = Children::Columns { children: col_children, spacing: Size::None };
            rows_children.push(Section {
                element: gui.add_element(columns),
                size: Size::None
            })
        }
        rows.children = Children::Rows { children: rows_children, spacing: Size::None };

        let mut drag_element = Element::new().with_label("Drag Element");
        drag_element.styles.set_visible(false);
        drag_element.styles.transfomr_mut().width = Size::AbsPercent(10.0);
        drag_element.styles.transfomr_mut().height = Size::AbsPercent(10.0);
        drag_element.styles.transfomr_mut().max_height = Size::Pixel(150.0);
        drag_element.styles.transfomr_mut().max_width = Size::Pixel(150.0);
        drag_element.styles.transfomr_mut().align = Position::Top;
        *drag_element.styles.bg_color_mut() = Color::RED;
        drag_element.styles.transfomr_mut().position_round = Round::Round;
        drag_element.styles.transfomr_mut().scale_round = Round::Round;
        drag_element.styles.text_mut().size = (Size::Percent(50.0), Side::Min);
        drag_element.styles.set_alpha(0.5);

        let rows = gui.add_element(rows);
        let drag_element = gui.add_element(drag_element);
        let key = gui.add_element(entry.with_children(Children::Layers(vec![rows, drag_element])));
        gui.set_entry(Some(key));

        *self = App::App(Application {
            gui,
            drawing,
            window,
            drag: None,
            drag_element,
            hovering: None,
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

        // In the current iteration the events require to be reversed to work properly
        let mut events = Vec::new();
        while let Some(event) = this.gui.poll_event() {
            events.push(event);
        }

        for message in events.iter().rev() {
            let element = this.gui.get_element_mut(message.key).unwrap();
            match message.msg {
                Messages::BoxHover => {
                    match message.event_type {
                        EventTypes::MouseEnter => {
                            this.hovering = Some(message.key);
                            *element.styles.bg_color_mut() = Color::RED;
                            this.window.set_cursor(CursorIcon::Pointer);
                        }
                        EventTypes::MouseLeave => {
                            this.hovering = None;
                            *element.styles.bg_color_mut() = Color::YELLOW;
                            if this.drag.is_none() {
                                this.window.set_cursor(CursorIcon::Default);
                            }
                        }
                        _=> {}
                    }
                }
                Messages::BoxDrag => {
                    let text = element.text().map(|s|s.to_string());
                    element.set_text(None);
                    this.drag = Some((text.clone(), message.key));
                    let drag_element = this.gui.get_element_mut(this.drag_element).unwrap();
                    drag_element.styles.set_visible(true);
                    drag_element.set_text(text);
                }
                Messages::BoxDrop => {
                    match &mut this.drag {
                        Some(drag) => {
                            match this.hovering {
                                Some(hovering) => {{
                                    let hovering = this.gui.get_element(hovering.clone()).unwrap();
                                    let hovering_text = hovering.text().map(|s|s.to_string());
                                    let original = this.gui.get_element_mut(drag.1).unwrap();
                                    original.set_text(hovering_text);}
                                    let hovering = this.gui.get_element_mut(hovering).unwrap();
                                    hovering.set_text(drag.0.clone());
                                }
                                None => {
                                    let original = this.gui.get_element_mut(drag.1).unwrap();
                                    original.set_text(drag.0.clone());
                                }
                            }
                            let drag_element = this.gui.get_element_mut(this.drag_element).unwrap();
                            drag_element.styles.set_visible(false);
                        }
                        None => ()
                    }
                    this.drag = None;
                }
                Messages::MouseMove => {
                    let pos = if let WindowEvent::MouseMove { position, .. } = message.window_event {
                        position
                    } else {
                        continue;
                    };
                    let size = (this.gui.size().0 as f32 * 0.5, this.gui.size().1 as f32 * 0.5 - 10.0);
                    let drag_element = this.gui.get_element_mut(this.drag_element).unwrap();
                    drag_element.styles.transfomr_mut().position = Position::Custom(Size::Pixel(pos.x - size.0), Size::Pixel(pos.y - size.1));
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
