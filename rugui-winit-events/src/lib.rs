use nalgebra::Point2;
use rugui::events::WindowEvent as RuguiWindowEvent;
use winit::{
    event::WindowEvent as WinitWindowEvent,
    keyboard::{Key, NamedKey},
};

pub struct State {
}

impl State {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn event(&mut self, event: &WinitWindowEvent) -> Option<RuguiWindowEvent> {
        match event {
            WinitWindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let button = convert_mouse_button(*button)?;
                match state {
                    winit::event::ElementState::Pressed => {
                        Some(RuguiWindowEvent::MouseDown {
                            button,
                        })
                    }
                    winit::event::ElementState::Released => {
                        Some(RuguiWindowEvent::MouseUp {
                            button,
                        })
                    }
                }
            }
            WinitWindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                Some(RuguiWindowEvent::MouseMove {
                    position: Point2::new(position.x as f32, position.y as f32),
                })
            }
            WinitWindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                let delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => Point2::new(*x, *y),
                    winit::event::MouseScrollDelta::PixelDelta(delta) => {
                        Point2::new(delta.x as f32, delta.y as f32)
                    }
                };
                Some(RuguiWindowEvent::Scroll {
                    delta,
                })
            }
            WinitWindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => match &event.logical_key {
                Key::Named(name) => match name {
                    NamedKey::ArrowLeft => Some(RuguiWindowEvent::SelectNext),
                    NamedKey::ArrowRight => Some(RuguiWindowEvent::SelectPrevious),
                    _ => None,
                },
                Key::Character(c) => {
                    Some(RuguiWindowEvent::Input {
                        text: c.to_string(),
                    })
                }
                _ => None,
            },
            _ => None,
        }
    }
}

fn convert_mouse_button(button: winit::event::MouseButton) -> Option<rugui::events::MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(rugui::events::MouseButton::Left),
        winit::event::MouseButton::Right => Some(rugui::events::MouseButton::Right),
        winit::event::MouseButton::Middle => Some(rugui::events::MouseButton::Middle),
        _ => None,
    }
}
