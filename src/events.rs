use nalgebra::{Point2, Vector2};

use crate::ElementKey;

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Events that can be triggered by the user
pub enum Event<'a> {
    /// A mouse button was clicked
    MouseDown {
        button: MouseButton,
        position: Point2<f32>,
    },
    /// A mouse button was released
    MouseUp {
        button: MouseButton,
        position: Point2<f32>,
    },
    /// The mouse was moved
    MouseMove {
        position: Point2<f32>,
        delta: Vector2<f32>,
    },
    /// The mouse wheel was scrolled
    Scroll { delta: f32, position: Point2<f32> },
    /// Logical key press
    ///
    /// This event considers the current keyboard layout and modifiers
    Input { text: &'a str },
    /// Physical key press
    ///
    /// This event fires when a key is pressed on the keyboard
    Key { code: u32 },
    /// Select the next element
    SelectNext,
    /// Select the previous element
    SelectPrevious,
    /// Window gained focus
    Focus,
    /// Window lost focus
    Blur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTypes {
    MouseDown,
    MouseUp,
    MouseMove,
    Scroll,
    Input,
    SelectNext,
    SelectPrevious,
    Focus,
    Blur,
}

/// Node response to an event
#[derive(Debug, Clone)]
pub enum EventResponse<Msg>
where
    Msg: Clone,
{
    Consumed,
    Ignored,
    Message(UserEvent<Msg>),
}

pub struct UserEvents<Msg>
where
    Msg: Clone,
{
    pub events: Vec<UserEvent<Msg>>,
}

#[derive(Clone, Debug)]
pub struct UserEvent<Msg>
where
    Msg: Clone,
{
    pub listener: EventListeners,
    pub msg: Msg,
    pub key: ElementKey,
}

#[derive(Clone, Debug)]
pub enum EventListeners {
    MouseDown,
    MouseUp,
    MouseMove,
    Scroll,
    Input,
    SelectNext,
    SelectPrevious,
    MouseEnter,
    MouseLeave,
}
