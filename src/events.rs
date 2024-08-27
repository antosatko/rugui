use crate::{Point, Element, ElementKey, InputState};

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
    MouseDown { button: MouseButton },
    /// A mouse button was released
    MouseUp { button: MouseButton },
    /// The mouse was moved
    MouseMove { position: Point, last: Point },
    /// The mouse wheel was scrolled
    Scroll { delta: Point },
    /// Logical key press
    ///
    /// This event considers the current keyboard layout and modifiers
    Input { text: String },
    SelectNext,
    SelectPrev,
}

#[derive(Debug, Clone)]
pub enum ElementEvent {
    /// A mouse button was clicked
    MouseDown { 
        button: MouseButton,
        position: Point,
    },
    /// A mouse button was released
    MouseUp { 
        button: MouseButton,
        position: Point,
    },
    /// The mouse was moved
    MouseMove { position: Point, last: Point },
    /// The mouse wheel was scrolled
    Scroll { delta: Point, position: Point },
    /// Logical key press
    ///
    /// This event considers the current keyboard layout and modifiers
    Input { text: String },
    Select,
    Unselect,
}

impl ElementEvent {
    pub(crate) fn from_window_event<M: Clone>(event: &WindowEvent, element: &Element<M>, inputs: &InputState) -> Self {
        match event {
            WindowEvent::MouseDown { button } => ElementEvent::MouseDown { button: button.clone(), position: element.place_point(inputs.mouse) },
            WindowEvent::MouseUp { button } => ElementEvent::MouseUp { button: button.clone(), position: element.place_point(inputs.mouse) },
            WindowEvent::MouseMove { .. } => ElementEvent::MouseMove { position: element.place_point(inputs.mouse), last: element.place_point(inputs.prev_mouse) },
            WindowEvent::Scroll { delta } => ElementEvent::Scroll { delta: delta.clone(), position: element.place_point(inputs.mouse) },
            WindowEvent::Input { text } => ElementEvent::Input { text: text.clone() },
            WindowEvent::SelectNext => unreachable!("ble ble contact the developer"),
            WindowEvent::SelectPrev => unreachable!("ble ble contact the developer"),
        }
    }
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
    Select,
}

/// Element response to an event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResponse {
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
    pub window_event: WindowEvent,
    pub element_event: ElementEvent,
    pub msg: Msg,
    pub key: ElementKey,
}
