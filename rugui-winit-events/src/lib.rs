use rugui::Point;
use rugui::events::WindowEvent as RuguiWindowEvent;
use winit::{event::{ElementState, WindowEvent as WinitWindowEvent}, keyboard::Key};
pub fn event(event: &WinitWindowEvent) -> Option<RuguiWindowEvent> {
    match event {
        WinitWindowEvent::MouseInput {
            device_id,
            state,
            button,
        } => {
            let button = convert_mouse_button(*button)?;
            match state {
                winit::event::ElementState::Pressed => Some(RuguiWindowEvent::MouseDown { button }),
                winit::event::ElementState::Released => Some(RuguiWindowEvent::MouseUp { button }),
            }
        }
        WinitWindowEvent::CursorMoved {
            device_id,
            position,
        } => Some(RuguiWindowEvent::MouseMove {
            position: Point::new(position.x as f32, position.y as f32),
            last: Point::new(position.x as f32, position.y as f32),
        }),
        WinitWindowEvent::MouseWheel {
            device_id,
            delta,
            phase,
        } => {
            let delta = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => Point::new(*x, *y),
                winit::event::MouseScrollDelta::PixelDelta(delta) => {
                    Point::new(delta.x as f32, delta.y as f32)
                }
            };
            Some(RuguiWindowEvent::Scroll { delta })
        }
        WinitWindowEvent::KeyboardInput {
            device_id,
            event,
            is_synthetic,
        } => match &event.logical_key {
            Key::Character(c) => Some(RuguiWindowEvent::Input {
                text: c.to_string(),
            }),
            Key::Named(winit::keyboard::NamedKey::Tab) => if event.state == ElementState::Pressed {
                Some(RuguiWindowEvent::SelectNext)
            } else {
                None
            }
            _ => None
        },
        _ => None,
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
