use nalgebra::{Point2, Vector2};

use crate::ElementKey;

#[derive(Debug, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Events that can be triggered by the user
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// A mouse button was clicked
    MouseDown {
        button: MouseButton,
    },
    /// A mouse button was released
    MouseUp {
        button: MouseButton,
    },
    /// The mouse was moved
    MouseMove {
        position: Point2<f32>,
    },
    /// The mouse wheel was scrolled
    Scroll { delta:  Point2<f32> },
    /// Logical key press
    ///
    /// This event considers the current keyboard layout and modifiers
    Input { text: String },
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
    MouseEnter,
    MouseLeave,
    Scroll,
    Input,
    SelectNext,
    SelectPrevious,
}

/// Element response to an event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResponse
{
    Consumed,
    Ignored,
}

pub struct EventPoll<Msg>
where
    Msg: Clone,
{
    pub queue: Vec<WindowEvent>,
    pub events: Vec<Event<Msg>>,
}

#[derive(Clone, Debug)]
pub struct Event<Msg>
where
    Msg: Clone,
{
    pub event_type: EventTypes,
    pub event: WindowEvent,
    pub msg: Msg,
    pub key: ElementKey,
}
