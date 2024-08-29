//! Winit integration helpers *(use `winit` flag)*


use crate::events::WindowEvent as RuguiWindowEvent;
use crate::Point;
use winit::{
    event::{ElementState, WindowEvent as WinitWindowEvent},
    keyboard::{Key, NamedKey},
};
pub fn event<Msg: Clone>(gui: &mut crate::Gui<Msg>, event: &WinitWindowEvent) {
    match event {
        WinitWindowEvent::MouseInput {
            device_id: _,
            state,
            button,
        } => match convert_mouse_button(*button) {
            Some(button) => match state {
                winit::event::ElementState::Pressed => {
                    gui.event(RuguiWindowEvent::MouseDown { button })
                }
                winit::event::ElementState::Released => {
                    gui.event(RuguiWindowEvent::MouseUp { button })
                }
            },
            _ => (),
        },
        WinitWindowEvent::CursorMoved {
            device_id: _,
            position,
        } => gui.event(RuguiWindowEvent::MouseMove {
            position: Point::new(position.x as f32, position.y as f32),
            last: Point::new(position.x as f32, position.y as f32),
        }),
        WinitWindowEvent::MouseWheel {
            device_id: _,
            delta,
            phase: _,
        } => {
            let delta = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => Point::new(*x, *y),
                winit::event::MouseScrollDelta::PixelDelta(delta) => {
                    Point::new(delta.x as f32, delta.y as f32)
                }
            };
            gui.event(RuguiWindowEvent::Scroll { delta })
        }
        WinitWindowEvent::KeyboardInput {
            device_id: _,
            event,
            is_synthetic: _,
        } => {
            match event.state {
                ElementState::Pressed => match &event.logical_key {
                    Key::Named(winit::keyboard::NamedKey::Tab) => {
                        gui.event(RuguiWindowEvent::SelectNext)
                    }
                    Key::Named(NamedKey::Control) => {
                        gui.input.control_pressed = true;
                    }
                    #[cfg(feature = "clipboard")]
                    Key::Character(c) if gui.input.control_pressed =>
                    {
                        if c.as_str() == "v" {
                            use clipboard::ClipboardProvider;
                            if let Some(clip) = &mut gui.clipboard_ctx {
                                match clip.get_contents() {
                                    Ok(text) => gui.event(RuguiWindowEvent::Input { text }),
                                    _ => (),
                                }
                            }
                        }
                    }
                    _ => (),
                },
                ElementState::Released => match &event.logical_key {
                    Key::Named(NamedKey::Control) => {
                        gui.input.control_pressed = false;
                    }
                    _ => (),
                },
            }
            if let Some(input) = key_input(event) {
                if !gui.input.control_pressed {
                    gui.event(RuguiWindowEvent::Input { text: input })
                }
            }
        }
        _ => (),
    }
}

fn convert_mouse_button(button: winit::event::MouseButton) -> Option<crate::events::MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(crate::events::MouseButton::Left),
        winit::event::MouseButton::Right => Some(crate::events::MouseButton::Right),
        winit::event::MouseButton::Middle => Some(crate::events::MouseButton::Middle),
        _ => None,
    }
}

fn key_input(event: &winit::event::KeyEvent) -> Option<String> {
    if event.state != ElementState::Pressed {
        return None;
    }
    match &event.logical_key {
        Key::Named(winit::keyboard::NamedKey::Tab) => return None,
        Key::Named(winit::keyboard::NamedKey::Backspace) => return None,
        _ => (),
    }
    match &event.text {
        Some(txt) => return Some(txt.to_string()),
        None => None,
    }
}
