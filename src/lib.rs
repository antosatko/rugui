//! Heya!
//!
//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

use std::collections::HashMap;

#[cfg(feature = "clipboard")]
use clipboard::{ClipboardContext, ClipboardProvider};
use events::{ElementEvent, EventPoll, EventTypes, WindowEvent};
use cosmic_text::{Attrs, FontSystem, Metrics, SwashCache};
use image::{DynamicImage, GenericImage};
use render::{GpuBound, LinearGradientData, RadialGradientData, RenderElement, RenderElementData, RenderLinearGradient, RenderRadialGradient};
use styles::{Values, ViewPort};

pub mod events;
mod render;
pub mod styles;
pub mod texture;
#[cfg(feature = "winit")]
pub mod winit;

/// Context for the GUI engine
///
/// Always have only one in your application in order to save resources.
pub struct Gui<Msg>
where
    Msg: Clone,
{
    elements: HashMap<ElementKey, Element<Msg>>,
    events: EventPoll<Msg>,
    entry: Option<ElementKey>,
    last_key: u64,
    size: (u32, u32),
    gpu: GpuBound,
    input: InputState,
    font_system: Option<FontSystem>,
    swash_cache: Option<SwashCache>,
    select: Select,
    ordered: Vec<ElementKey>,
    #[cfg(feature = "clipboard")]
    clipboard_ctx: Option<ClipboardContext>,
}

struct InputState {
    pub(crate) mouse: Point,
    pub(crate) prev_mouse: Point,
    pub(crate) hover: Option<ElementKey>,
    pub(crate) control_pressed: bool,
}

pub(crate) struct Select {
    pub selected: Option<ElementKey>,
    pub selectables: Vec<ElementKey>,
}

impl Select {
    pub fn new() -> Self {
        Self {
            selected: None,
            selectables: Vec::new(),
        }
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse: Point::new(0.0, 0.0),
            prev_mouse: Point::new(0.0, 0.0),
            hover: None,
            control_pressed: false,
        }
    }
}

/// Key helps you access elements managed by the `Gui`
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ElementKey {
    id: u64,
}

impl<Msg> Gui<Msg>
where
    Msg: Clone,
{
    pub fn new(size: (u32, u32), device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let gpu = GpuBound::new(queue, device, size);
        let this = Self {
            elements: HashMap::new(),
            events: EventPoll {
                events: Vec::new(),
                queue: Vec::new(),
            },
            last_key: 0,
            entry: None,
            size,
            gpu,
            input: InputState::new(),
            font_system: Some(FontSystem::new()),
            swash_cache: Some(SwashCache::new()),
            select: Select::new(),
            ordered: Vec::new(),
            #[cfg(feature = "clipboard")]
            clipboard_ctx: ClipboardContext::new().ok(),
        };
        this
    }

    pub fn poll_event(&mut self) -> Option<events::Event<Msg>> {
        self.events.events.pop()
    }

    pub fn add_element(&mut self, element: Element<Msg>) -> ElementKey {
        let key = ElementKey { id: self.last_key };
        self.last_key += 1;
        self.elements.insert(key, element);
        key
    }

    pub fn remove_element(&mut self, key: ElementKey) {
        self.elements.remove(&key);
    }

    pub fn get_element(&self, key: ElementKey) -> Option<&Element<Msg>> {
        self.elements.get(&key).map(|element| element)
    }

    pub fn get_element_mut(&mut self, key: ElementKey) -> Option<&mut Element<Msg>> {
        self.elements.get_mut(&key).map(|element| element)
    }

    pub fn set_entry(&mut self, key: Option<ElementKey>) {
        if let Some(entry) = self.entry.take() {
            self.remove_element(entry);
        }
        self.entry = key;
        if let Some(key) = key {
            let transform = ElementTransform {
                position: Point::new(self.size.0 as f32 / 2.0, self.size.1 as f32 / 2.0),
                scale: Point::new(self.size.0 as f32, self.size.1 as f32),
                rotation: 0.0,
            };
            self.element_transform(key, &transform);
        }
    }

    pub fn event(&mut self, event: events::WindowEvent) {
        self.events.queue.push(event);
    }

    pub fn select_element(&mut self, key: ElementKey, msg: Msg) {
        match self.select.selected {
            Some(selected) => {
                if selected == key {
                    return;
                }
                let listeners = if let Some(element) = self.get_element(selected) {
                    match element.events.get(&EventTypes::Select) {
                        Some(m) => m.clone(),
                        None => return,
                    }
                } else {
                    return;
                };
                for EventListener { msg, .. } in listeners {
                    self.events.events.push(events::Event {
                        event_type: EventTypes::Select,
                        window_event: WindowEvent::SelectNext,
                        element_event: ElementEvent::Unselect,
                        msg: msg.clone(),
                        key: selected,
                    });
                }
            }
            None => (),
        }
        self.select.selected = Some(key);
        self.events.events.push(events::Event {
            event_type: EventTypes::Select,
            window_event: WindowEvent::SelectNext,
            element_event: ElementEvent::Select,
            msg,
            key,
        });
    }

    fn fix_hovers(&mut self, event: &events::WindowEvent) {
        let this_hover = self.find_hovered_element();
        if self.input.hover != this_hover {
            match self.input.hover {
                Some(key) => {
                    if let Some(e) = self.get_element(key) {
                        let element_event = ElementEvent::from_window_event(event, e, &self.input);
                        if let Some(listeners) = e.events.get(&EventTypes::MouseLeave) {
                            for EventListener { msg, .. } in listeners {
                                self.events.events.push(events::Event {
                                    event_type: EventTypes::MouseLeave,
                                    window_event: event.clone(),
                                    element_event: element_event.clone(),
                                    msg,
                                    key,
                                })
                            }
                        }
                    }
                }
                None => (),
            }
            match this_hover {
                Some(key) => {
                    if let Some(e) = self.get_element(key) {
                        let element_event = ElementEvent::from_window_event(&event, e, &self.input);
                        if let Some(listeners) = e.events.get(&EventTypes::MouseEnter) {
                            for EventListener { msg, .. } in listeners {
                                self.events.events.push(events::Event {
                                    event_type: EventTypes::MouseEnter,
                                    window_event: event.clone(),
                                    element_event: element_event.clone(),
                                    msg,
                                    key,
                                })
                            }
                        }
                    }
                }
                None => (),
            }
        }
        self.input.hover = this_hover;
    }

    fn find_hovered_element(&self) -> Option<ElementKey> {
        for key in self.ordered.iter().rev() {
            let element = if let Some(e) = self.get_element(*key) {
                e
            } else {
                continue;
            };
            if element.transform.point_collision(self.input.mouse) {
                return Some(*key);
            }
        }
        None
    }

    fn resolve_events(&mut self) {
        while let Some(event) = self.events.queue.pop() {
            match &event {
                WindowEvent::MouseMove { position, .. } => {
                    self.input.prev_mouse = self.input.mouse;
                    self.input.mouse = *position;

                    self.fix_hovers(&event)
                }
                WindowEvent::SelectNext => {
                    match &self.select.selected {
                        Some(selected) => {
                            let len = if self.select.selectables.len() == 0 {
                                continue;
                            } else {
                                self.select.selectables.len()
                            };
                            let listeners = if let Some(element) = self.get_element(*selected) {
                                match element.events.get(&EventTypes::Select) {
                                    Some(m) => m.clone(),
                                    None => return,
                                }
                            } else {
                                return;
                            };
                            match self.select.selectables.iter().position(|k| k == selected) {
                                Some(i) => {
                                    if i + 1 >= len {
                                        for EventListener { msg, .. } in listeners {
                                            self.events.events.push(events::Event {
                                                event_type: EventTypes::Select,
                                                window_event: WindowEvent::SelectNext,
                                                element_event: ElementEvent::Unselect,
                                                msg,
                                                key: *selected,
                                            });
                                        }
                                        self.select.selected = None;
                                    } else {
                                        for EventListener { msg, .. } in listeners {
                                            self.events.events.push(events::Event {
                                                event_type: EventTypes::Select,
                                                window_event: WindowEvent::SelectNext,
                                                element_event: ElementEvent::Unselect,
                                                msg,
                                                key: *selected,
                                            });
                                        }
                                        self.select.selected = Some(self.select.selectables[i + 1]);
                                        let listeners = if let Some(element) =
                                            self.get_element(self.select.selectables[i + 1])
                                        {
                                            match element.events.get(&EventTypes::Select) {
                                                Some(m) => m.clone(),
                                                None => return,
                                            }
                                        } else {
                                            return;
                                        };
                                        for EventListener { msg, .. } in listeners {
                                            self.events.events.push(events::Event {
                                                event_type: EventTypes::Select,
                                                window_event: WindowEvent::SelectNext,
                                                element_event: ElementEvent::Select,
                                                msg,
                                                key: self.select.selectables[i + 1],
                                            });
                                        }
                                    }
                                }
                                None => match self.select.selectables.first() {
                                    Some(key) => {
                                        for EventListener { msg, .. } in listeners {
                                            self.events.events.push(events::Event {
                                                event_type: EventTypes::Select,
                                                window_event: WindowEvent::SelectNext,
                                                element_event: ElementEvent::Select,
                                                msg,
                                                key: *selected,
                                            });
                                        }
                                        self.select.selected = Some(*key);
                                        let listeners =
                                            if let Some(element) = self.get_element(*key) {
                                                match element.events.get(&EventTypes::Select) {
                                                    Some(m) => m.clone(),
                                                    None => return,
                                                }
                                            } else {
                                                return;
                                            };
                                        for EventListener { msg, .. } in listeners {
                                            self.events.events.push(events::Event {
                                                event_type: EventTypes::Select,
                                                window_event: WindowEvent::SelectNext,
                                                element_event: ElementEvent::Select,
                                                msg,
                                                key: *key,
                                            });
                                        }
                                    }
                                    None => {
                                        for EventListener { msg, .. } in listeners {
                                            self.events.events.push(events::Event {
                                                event_type: EventTypes::Select,
                                                window_event: WindowEvent::SelectNext,
                                                element_event: ElementEvent::Unselect,
                                                msg,
                                                key: *selected,
                                            });
                                        }
                                        self.select.selected = None;
                                    }
                                },
                            }
                        }
                        None => match self.select.selectables.first() {
                            Some(key) => {
                                self.select.selected = Some(*key);
                                let listeners = if let Some(element) = self.get_element(*key) {
                                    match element.events.get(&EventTypes::Select) {
                                        Some(m) => m.clone(),
                                        None => return,
                                    }
                                } else {
                                    return;
                                };
                                for EventListener { msg, .. } in listeners {
                                    self.events.events.push(events::Event {
                                        event_type: EventTypes::Select,
                                        window_event: WindowEvent::SelectNext,
                                        element_event: ElementEvent::Select,
                                        msg,
                                        key: *key,
                                    });
                                }
                            }
                            None => (),
                        },
                    }
                    return;
                }
                WindowEvent::Input { text } => {
                    let key = if let Some(key) = self.select.selected {
                        key
                    } else {
                        return;
                    };
                    if let Some(e) = self.get_element(key) {
                        match e.events.get(&EventTypes::Input) {
                            Some(e) => {
                                for EventListener { msg, .. } in e {
                                    self.events.events.push(events::Event {
                                        event_type: EventTypes::Input,
                                        window_event: event.clone(),
                                        element_event: ElementEvent::Input { text: text.clone() },
                                        msg,
                                        key,
                                    });
                                }
                            }
                            None => {}
                        }
                    }
                    return;
                }
                _ => {}
            }
            //self.element_event(entry_key, &event);
            let mut consumed = false;
            for i in (0..self.ordered.len()).rev() {
                let element = if let Some(e) = self.get_element(self.ordered[i]) {
                    e
                } else {
                    continue;
                };

                match &event {
                    // Events that need to take into account cursor position
                    WindowEvent::MouseDown { .. }
                    | WindowEvent::MouseUp { .. }
                    | WindowEvent::Scroll { .. }
                    | WindowEvent::MouseMove { .. } => {
                        let event_type = event.clone().into();
                        match element.events.get(&event_type) {
                            Some(listeners) => {
                                let position = self.input.mouse;
                                if element.transform.point_collision(position) {
                                    let element_event = ElementEvent::from_window_event(
                                        &event,
                                        element,
                                        &self.input,
                                    );
                                    for EventListener {
                                        listener_type, msg, ..
                                    } in &listeners
                                    {
                                        match listener_type {
                                            EventListenerTypes::Listen => {
                                                if consumed {
                                                    continue;
                                                }
                                                self.events.events.push(events::Event {
                                                    event_type,
                                                    window_event: event.clone(),
                                                    element_event: element_event.clone(),
                                                    msg: msg.clone(),
                                                    key: self.ordered[i],
                                                });
                                                consumed = true;
                                            }
                                            EventListenerTypes::Peek => {
                                                if consumed {
                                                    continue;
                                                }
                                                self.events.events.push(events::Event {
                                                    event_type,
                                                    window_event: event.clone(),
                                                    element_event: element_event.clone(),
                                                    msg: msg.clone(),
                                                    key: self.ordered[i],
                                                });
                                            }
                                            EventListenerTypes::Force => {
                                                self.events.events.push(events::Event {
                                                    event_type,
                                                    window_event: event.clone(),
                                                    element_event: element_event.clone(),
                                                    msg: msg.clone(),
                                                    key: self.ordered[i],
                                                });
                                                consumed = true;
                                            }
                                        }
                                    }
                                }
                            }
                            None => continue,
                        }
                    }
                    WindowEvent::Input { .. } => (),
                    WindowEvent::SelectNext => (),
                    WindowEvent::SelectPrev => (),
                }
            }
        }
    }

    pub fn resize(&mut self, size: (u32, u32), queue: &wgpu::Queue) {
        self.resolve_events();
        self.size = size;
        self.gpu.resize((size.0, size.1), queue);
        let entry_key = if let Some(entry) = &self.entry {
            entry
        } else {
            return;
        };
        /*if let Some(entry) = self.elements.get_mut(&entry_key) {
            entry.styles.flags.recalc_transform = true;
        }*/
        self.element_transform(
            *entry_key,
            &ElementTransform {
                position: Point::new(size.0 as f32 / 2.0, size.1 as f32 / 2.0),
                scale: Point::new(size.0 as f32, size.1 as f32),
                rotation: 0.0,
            },
        );
    }

    pub fn update(&mut self) {
        self.resolve_events();
        let entry_key = if let Some(entry) = self.entry {
            entry
        } else {
            return;
        };
        self.ordered.clear();
        self.select.selectables.clear();
        self.order(entry_key);
        let mut ordered = self.ordered.clone();
        ordered.sort_by(|a, b| {
            self.get_element(*a)
                .map(|e| e.styles.z_index)
                .unwrap_or(0)
                .cmp(&self.get_element(*b).map(|e| e.styles.z_index).unwrap_or(0))
        });
        self.ordered = ordered;
        self.element_transform(
            entry_key,
            &ElementTransform {
                position: Point::new(self.size.0 as f32 / 2.0, self.size.1 as f32 / 2.0),
                scale: Point::new(self.size.0 as f32, self.size.1 as f32),
                rotation: 0.0,
            },
        );
    }

    fn order(&mut self, key: ElementKey) {
        let element = if let Some(element) = self.get_element(key) {
            if !element.styles.visible {
                return;
            }
            element
        } else {
            return;
        };
        if element.styles.selectable {
            self.select.selectables.push(key);
        }
        self.ordered.push(key);
        let element = if let Some(element) = self.get_element(key) {
            element
        } else {
            return;
        };
        match &element.children {
            Children::Element(key) => self.order(*key),
            Children::Layers(layers) => {
                let keys = layers.clone();
                for key in keys {
                    self.order(key);
                }
            }
            Children::Rows {
                children,
                spacing: _,
            } => {
                let keys = children.clone();
                for Section { element, .. } in keys {
                    self.order(element);
                }
            }
            Children::Columns {
                children,
                spacing: _,
            } => {
                let keys = children.clone();
                for Section { element, .. } in keys {
                    self.order(element);
                }
            }
            Children::None => (),
        }
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut font = self.font_system.take().unwrap();
        let mut swash = self.swash_cache.take().unwrap();
        for i in 0..self.ordered.len() {
            let e = if let Some(e) = self.get_element_mut(self.ordered[i]) {
                e
            } else {
                continue;
            };
            e.write(device, queue, &mut font, &mut swash)
        }
        self.font_system = Some(font);
        self.swash_cache = Some(swash);
    }

    fn element_transform(&mut self, key: ElementKey, transform: &ElementTransform) {
        let (container, view_port) = (
            transform.clone().into(),
            ViewPort(self.size.0 as f32, self.size.1 as f32),
        );
        let element = match self.elements.get_mut(&key) {
            Some(element) => element,
            None => return,
        };
        if true {
            let (width, height) = (
                element
                    .styles
                    .width.get().calc(&container, &view_port),
                element
                    .styles
                    .height.get().calc(&container, &view_port),
            );
            let pos = 
                element
                    .styles
                    .position
                    .get()
                    .calc(&container, &view_port)
            ;
            let rotation = element
                .styles
                .rotation
                .get()
                .calc(&container, &view_port);
            let margin = element.styles.margin.get().calc(&container, &view_port);
            let transform = ElementTransform {
                position: pos,
                scale: Point::new((width-margin).max(0.0), (height-margin).max(0.0)),
                rotation,
            };

            let pre_collision = element.transform.point_collision(self.input.mouse);
            element.transform = transform;
            

            let post_collision = element.transform.point_collision(self.input.mouse);
            match (pre_collision, post_collision) {
                (true, false) => {
                    if let Some(listeners) = element.events.get(&EventTypes::MouseLeave) {
                        for EventListener { msg, .. } in listeners {
                            let event = WindowEvent::MouseMove {
                                position: self.input.mouse,
                                last: self.input.prev_mouse,
                            };
                            self.events.events.push(events::Event {
                                event_type: EventTypes::MouseLeave,
                                element_event: ElementEvent::from_window_event(
                                    &event,
                                    &element,
                                    &self.input,
                                ),
                                window_event: event,
                                msg: msg.clone(),
                                key,
                            });
                        }
                    }
                }
                (false, true) => {
                    if let Some(listeners) = element.events.get(&EventTypes::MouseEnter) {
                        for EventListener { msg, .. } in listeners {
                            let event = WindowEvent::MouseMove {
                                position: self.input.mouse,
                                last: self.input.prev_mouse,
                            };
                            self.events.events.push(events::Event {
                                event_type: EventTypes::MouseEnter,
                                element_event: ElementEvent::from_window_event(
                                    &event,
                                    &element,
                                    &self.input,
                                ),
                                window_event: event,
                                msg: msg.clone(),
                                key,
                            });
                        }
                    }
                }
                _ => {}
            }
            let container = element.transform.clone().into();


            let edges_radius = element.styles.edges_radius.get().calc(&container, &view_port);
            element.render_element.1.edges[0] = edges_radius;
            let edges_smooth = element.styles.edges_smooth.get().calc(&container, &view_port);
            element.render_element.1.edges[1] = edges_smooth;

            let font_size = element.styles.text_size.get().calc(&container, &view_port);
            element.render_element.1.text_size = font_size;


            match element.styles.bg_linear_gradient.get() {
                Some(grad) => {
                    let calc = grad.calc(&container, &view_port);
                    let data = &mut element.render_element.1.lin_grad;
                    match data {
                        Some(lin) => {
                            lin.start = calc.0.0.into();
                            lin.end = calc.1.0.into();
                            lin.start_color = calc.0.1.to_rgba().into();
                            lin.end_color = calc.1.1.to_rgba().into();
                        }
                        None => {
                            let lin = LinearGradientData {
                                start: calc.0.0.into(),
                                end: calc.1.0.into(),
                                start_color: calc.0.1.to_rgba().into(),
                                end_color: calc.1.1.to_rgba().into(),
                            };
                            element.render_element.1.lin_grad = Some(lin);
                        }
                    }
                }
                None => {
                    
                }
            }
            match element.styles.bg_radial_gradient.get() {
                Some(grad) => {
                    let calc = grad.calc(&container, &view_port);
                    let data = &mut element.render_element.1.rad_grad;
                    match data {
                        Some(rad) => {
                            rad.center = calc.0.0.into();
                            rad.outer = calc.1.0.into();
                            rad.center_color = calc.0.1.to_rgba().into();
                            rad.outer_color = calc.1.1.to_rgba().into();
                        }
                        None => {
                            let rad = RadialGradientData {
                                center: calc.0.0.into(),
                                outer: calc.1.0.into(),
                                center_color: calc.0.1.to_rgba().into(),
                                outer_color: calc.1.1.to_rgba().into(),
                            };
                            element.render_element.1.rad_grad = Some(rad);
                        }
                    }
                }
                None => {
                    
                }
            }
        }
        let transform = &element.transform;
        match element.children.to_owned() {
            Children::Element(child) => {
                let padding = element.styles.padding.get().calc(&container, &view_port);
                let transform = ElementTransform {
                    scale: Point::new(
                        transform.scale.x - padding,
                        transform.scale.y - padding,
                    ),
                    ..transform.clone()
                };
                self.element_transform(child.clone(), &transform);
                return;
            }
            Children::Layers(children) => {
                let padding = element.styles.padding.get().calc(&container, &view_port);
                let transform = ElementTransform {
                    scale: Point::new(
                        transform.scale.x - padding,
                        transform.scale.y - padding,
                    ),
                    ..transform.clone()
                };
                for child in children {
                    self.element_transform(child, &transform);
                }
            }
            Children::Rows { children, .. } => {
                if children.is_empty() {
                    return;
                }
                let mut len = children.len() as f32;
                let mut remaining_height = transform.scale.y;
                let mut y = transform.position.y - transform.scale.y / 2.0;
                let transform = element.transform.clone();
                for Section {
                    element,
                    size: spacing,
                } in children
                {
                    if remaining_height <= 0.0 {
                        break;
                    }
                    let space = match spacing {
                        Some(spacing) => spacing.calc(&container, &view_port),
                        None => remaining_height / len,
                    };
                    let position = if transform.rotation == 0.0 {
                        Point::new(transform.position.x, y + space / 2.0)
                    } else {
                        let pivot = transform.position;
                        let point = Point::new(transform.position.x, y + space / 2.0);
                        rotate_point(point, pivot, transform.rotation)
                    };
                    let transform = ElementTransform {
                        position,
                        scale: Point::new(transform.scale.x, space),
                        rotation: transform.rotation,
                    };
                    y += space;
                    remaining_height -= space;
                    len -= 1.0;
                    self.element_transform(element, &transform);
                }
            }
            Children::Columns { children, .. } => {
                if children.is_empty() {
                    return;
                }
                let mut len = children.len() as f32;
                let mut remaining_width = transform.scale.x;
                let mut x = transform.position.x - transform.scale.x / 2.0;
                let transform = element.transform.clone();
                for Section {
                    element,
                    size: spacing,
                } in children
                {
                    if remaining_width <= 0.0 {
                        break;
                    }
                    let space = match spacing {
                        Some(spacing) => spacing.calc(&container, &view_port),
                        None => remaining_width / len,
                    };
                    let position = if transform.rotation == 0.0 {
                        Point::new(x + space / 2.0, transform.position.y)
                    } else {
                        let pivot = transform.position;
                        let point = Point::new(x + space / 2.0, transform.position.y);
                        rotate_point(point, pivot, transform.rotation)
                    };
                    let transform = ElementTransform {
                        position,
                        scale: Point::new(space, transform.scale.y),
                        rotation: transform.rotation,
                    };
                    self.element_transform(element, &transform);
                    x += space;
                    remaining_width -= space;
                    len -= 1.0;
                }
            }
            Children::None => (),
        };
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_bind_group(0, &self.gpu.dimensions_bind_group, &[]);

        for e in self.ordered.iter() {
            if let Some(e) = self.get_element(*e) {
                if let Some(re) = &e.render_element.0 {
                    re.render(&self.gpu.pipelines, pass)
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
/// Transformation of an element
///
/// Element transformations are applied to the element and its children
/// when the element is rendered for the first time or when the element
/// or its parent is resized
pub(crate) struct ElementTransform {
    /// Position in x and y of the top left corner
    pub position: Point,
    /// Scale in width and height
    pub scale: Point,
    /// Rotation in radians
    pub rotation: f32,
}

impl ElementTransform {
    pub fn zeroed() -> Self {
        Self {
            position: Point::new(0.0, 0.0),
            scale: Point::new(0.0, 0.0),
            rotation: 0.0,
        }
    }

    pub fn point_collision(&self, point: Point) -> bool {
        let point_rotated = rotate_point(point, self.position, -self.rotation);
        let width = self.scale.x / 2.0;
        let height = self.scale.y / 2.0;
        let x = self.position.x - width;
        let y = self.position.y - height;
        let x_max = self.position.x + width;
        let y_max = self.position.y + height;

        point_rotated.x >= x
            && point_rotated.x <= x_max
            && point_rotated.y >= y
            && point_rotated.y <= y_max
    }
}

/// Most basic building block of the Rugui library
#[derive(Default)]
pub struct Element<Msg>
where
    Msg: Clone,
{
    text: Option<(String, bool)>,
    pub label: Option<String>,
    pub render_element: (Option<RenderElement>, RenderElementData),
    pub styles: styles::Styles,
    pub events: EventListeners<Msg>,
    pub children: Children,
    text_buffer: Option<cosmic_text::Buffer>,
    transform: ElementTransform,
    _parent: ElementTransform,
}

/// Holds all event listeners for an `Element`
#[derive(Debug, Clone, Default)]
pub struct EventListeners<Msg: Clone> {
    pub(crate) events: Vec<EventListener<Msg>>,
}

/// Listens to events
#[derive(Debug, Clone)]
pub struct EventListener<Msg: Clone> {
    event_type: EventTypes,
    listener_type: EventListenerTypes,
    msg: Msg,
}

impl<Msg: Clone> EventListeners<Msg> {
    /// Normal type of event listener
    ///
    /// This listener will only catch unconsumed events and will then consume it
    pub fn listen(&mut self, event_type: EventTypes, msg: Msg) {
        self.events.push(EventListener {
            event_type,
            listener_type: EventListenerTypes::Listen,
            msg,
        })
    }
    /// Special type of event listener
    ///
    /// This listener will catch both consumed and unconsumed events and will consume them
    ///
    /// Use this for fancy backgrounds
    pub fn force(&mut self, event_type: EventTypes, msg: Msg) {
        self.events.push(EventListener {
            event_type,
            listener_type: EventListenerTypes::Force,
            msg,
        })
    }
    /// Special type of event listener
    ///
    /// This listener will only catch unconsumed events and will not consume it
    ///
    /// Use this for fancy overlays
    pub fn peek(&mut self, event_type: EventTypes, msg: Msg) {
        self.events.push(EventListener {
            event_type,
            listener_type: EventListenerTypes::Peek,
            msg,
        })
    }

    pub fn get(&self, event_type: &EventTypes) -> Option<Vec<EventListener<Msg>>> {
        let collection = self
            .events
            .iter()
            .filter(|e| &e.event_type == event_type)
            .map(|e| e.clone())
            .collect::<Vec<EventListener<Msg>>>();
        match collection.len() {
            0 => None,
            _ => Some(collection),
        }
    }

    fn new() -> Self {
        Self { events: Vec::new() }
    }
}

/// Describes privilege level for listener
#[derive(Debug, Clone, Default)]
pub enum EventListenerTypes {
    /// Normal type of event listener
    ///
    /// This listener will only catch unconsumed events and will then consume it
    #[default]
    Listen,
    /// Special type of event listener
    ///
    /// This listener will only catch unconsumed events and will not consume it
    ///
    /// Use this for fancy overlays
    Peek,
    /// Special type of event listener
    ///
    /// This listener will catch both consumed and unconsumed events and will consume them
    ///
    /// Use this for fancy backgrounds
    Force,
}

impl<Msg> Element<Msg>
where
    Msg: Clone,
{
    /// Creates a new `Element`
    pub fn new() -> Self {
        Self {
            text: None,
            label: None,
            render_element: (None, RenderElementData::default()),
            styles: styles::Styles::default(),
            events: EventListeners::new(),
            children: Children::None,
            text_buffer: None,
            transform: ElementTransform::zeroed(),
            _parent: ElementTransform::zeroed(),
        }
    }

    /// Configures label for `Element`
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Configures styles for `Element`
    pub fn with_styles(mut self, styles: styles::Styles) -> Self {
        self.styles = styles;
        self
    }

    /// Configures children for `Element`
    pub fn with_children(mut self, children: Children) -> Self {
        self.children = children;
        self
    }

    pub fn event_listen(mut self, event_type: EventTypes, msg: Msg) -> Self {
        self.events.listen(event_type, msg);
        self
    }
    
    pub fn event_peek(mut self, event_type: EventTypes, msg: Msg) -> Self {
        self.events.peek(event_type, msg);
        self
    }

    pub fn event_force(mut self, event_type: EventTypes, msg: Msg) -> Self {
        self.events.force(event_type, msg);
        self
    }

    pub(crate) fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
    ) {
        if let None = &self.render_element.0 {
            self.render_element.0 = Some(RenderElement::zeroed(device))
        }
        let mut render_element = self.render_element.0.take().unwrap();
        self.render_element.1.color = self.styles.bg_color.get().to_rgba().into();
        if self.styles.bg_texture.dirty {
            if let Some(texture) = &self.styles.bg_texture.get() {
                render_element.set_texture(texture.clone());
            }

            self.styles.bg_texture.dirty = false;
        }
        if self.styles.bg_color.dirty {
            let color = self.styles.bg_color.get().to_rgba().into();
            render_element.set_color(color, queue, device);
            self.styles.bg_color.dirty = false;
        }
        if self.styles.edges_radius.dirty || self.styles.edges_smooth.dirty {
            /*let radius = self.transform.edges_radius;
            let smooth = self.transform.edges_smooth;
            self.render_element.1.edges = [
                radius,
                smooth,
            ];
            queue.write_buffer(
                &render_element.edges_buffer,
                0,
                bytemuck::cast_slice(&render_element.data.edges),
            );
            self.styles.flags.dirty_edges = false;*/
            queue.write_buffer(
                &render_element.edges_buffer,
                0,
                bytemuck::cast_slice(&[
                    self.render_element.1.edges[0],
                    self.render_element.1.edges[1],
                ]),
            );

        }
        if self.styles.alpha.dirty {
            let alpha = *self.styles.alpha.get();
            self.render_element.1.alpha = alpha;
            self.styles.alpha.dirty = false;
        }
        //if self.styles.flags.dirty_transform {
            let transform = &self.transform;
            self.render_element.1.update_transform(transform);
            if let Some((_, flag)) = &mut self.text {
                *flag = true;
            }
            //self.styles.flags.dirty_transform = false;
        //}
        if let Some(grad) = self.render_element.1.lin_grad {
            match &mut render_element.linear_gradient {
                Some(lin) => {
                    lin.start = grad.start;
                    lin.end = grad.end;
                    lin.start_color = grad.start_color;
                    lin.end_color = grad.end_color;
                    lin.write_all(queue);
                }
                _ => {
                    let mut lin = RenderLinearGradient::zeroed(device);
                    lin.start = grad.start;
                    lin.end = grad.end;
                    lin.start_color = grad.start_color;
                    lin.end_color = grad.end_color;
                    lin.write_all(queue);
                    render_element.linear_gradient = Some(lin);
                }
            }
        }
        self.styles.bg_linear_gradient.dirty = false;
        
        if let Some(grad) = self.render_element.1.rad_grad {
            match &mut render_element.radial_gradient {
                Some(rad) => {
                    rad.center = grad.center;
                    rad.outer = grad.outer;
                    rad.center_color = grad.center_color;
                    rad.outer_color = grad.outer_color;
                    rad.write_all(queue);
                }
                _ => {
                    let mut rad = RenderRadialGradient::zeroed(device);
                    rad.center = grad.center;
                    rad.outer = grad.outer;
                    rad.center_color = grad.center_color;
                    rad.outer_color = grad.outer_color;
                    rad.write_all(queue);
                    render_element.radial_gradient = Some(rad);
                }
            }
        }
        match &mut self.text {
            Some((txt, dirty)) => {
                if *dirty && transform.scale.x > 0.0 && transform.scale.y > 0.0 {
                    match &mut self.text_buffer {
                        Some(tb) => {
                            let mut tb = tb.borrow_with(font_system);
                            tb.set_metrics(Metrics::new(
                                self.render_element.1.text_size,
                                self.render_element.1.text_size+3.0,
                            ));
                            tb.set_size(Some(self.transform.scale.x), Some(self.transform.scale.y));
                            let attrs = Attrs::new();
                            tb.set_text(&txt, attrs, cosmic_text::Shaping::Advanced);
                            let color = self.styles.text_color.get().to_rgba();
                            let mut image = DynamicImage::new(
                                self.transform.scale.x as u32,
                                self.transform.scale.y as u32,
                                image::ColorType::Rgba8,
                            );
                            tb.draw(swash_cache, cosmic_text::Color::rgba((color.0 * 255.0) as u8, (color.1 * 255.0) as u8, (color.2 * 255.0) as u8, (color.3 * 255.0) as u8), |x, y, _, _, color| {
                                if x < 0
                                    || y < 0
                                    || x >= self.transform.scale.x as i32
                                    || y >= self.transform.scale.y as i32
                                {
                                    return;
                                }
                                image.put_pixel(x as u32, y as u32, color.as_rgba().into())
                            });
                            self.text_buffer = Some(tb.clone());
                            let tex = texture::Texture::from_image(device, queue, &image, None);
                            render_element.text = Some(tex)
                        }
                        None => {
                            let mut tb = cosmic_text::Buffer::new(
                                font_system,
                                Metrics::new(
                                    self.render_element.1.text_size,
                                    self.render_element.1.text_size+3.0,
                                ),
                            );
                            let mut tb = tb.borrow_with(font_system);
                            tb.set_size(Some(self.transform.scale.x), Some(self.transform.scale.y));
                            let attrs = Attrs::new();
                            tb.set_text(&txt, attrs, cosmic_text::Shaping::Advanced);
                            tb.shape_until_scroll(true);
                            let color = self.styles.text_color.get().to_rgba();
                            let mut image = DynamicImage::new(
                                self.transform.scale.x as u32,
                                self.transform.scale.y as u32,
                                image::ColorType::Rgba8,
                            );
                            tb.draw(swash_cache, cosmic_text::Color::rgba((color.0 * 255.0) as u8, (color.1 * 255.0) as u8, (color.2 * 255.0) as u8, (color.3 * 255.0) as u8), |x, y, _, _, color| {
                                if x < 0
                                    || y < 0
                                    || x >= self.transform.scale.x as i32
                                    || y >= self.transform.scale.y as i32
                                {
                                    return;
                                }
                                image.put_pixel(x as u32, y as u32, color.as_rgba().into())
                            });
                            self.text_buffer = Some(tb.clone());
                            let tex = texture::Texture::from_image(device, queue, &image, None);
                            render_element.text = Some(tex)
                        }
                    }
                    *dirty = false;
                }
            }
            None => render_element.text = None
        }

        render_element.write_all(queue, self.render_element.1);
        self.render_element.0 = Some(render_element)
    }

    /// Returns text rendered inside the `Element`
    pub fn text(&self) -> Option<&String> {
        match &self.text {
            Some((str, _)) => Some(str),
            None => None,
        }
    }

    /// Configures text rendered inside the `Element`
    pub fn set_text(&mut self, text: Option<String>) {
        match text {
            Some(text) => self.text = Some((text, true)),
            None => self.text = None,
        }
    }

    /// Configures text rendered inside the `Element`
    pub fn text_str(&mut self, str: &str) {
        match &mut self.text {
            Some((text, dirty)) => {
                *dirty = true;
                *text = str.to_string();
            }
            None => {
                self.text = Some((str.to_string(), true));
            }
        }
    }

    /// Configures text rendered inside the `Element`
    pub fn text_string(&mut self, str: String) {
        match &mut self.text {
            Some((text, dirty)) => {
                *dirty = true;
                *text = str;
            }
            None => {
                self.text = Some((str, true));
            }
        }
    }

    pub(crate) fn place_point(&self, point: Point) -> Point {
        let x = point.x - self.transform.position.x;
        let y = point.y - self.transform.position.y;
        let point = Point::new(x, y);
        rotate_point(point, Point::new(0.0, 0.0), -self.transform.rotation)
    }
}

/// Describes how many `Children` an `Element` has and how they should be positioned
#[derive(Clone, Debug, Default)]
pub enum Children {
    /// Positions child `Element` on top of parent
    Element(ElementKey),
    /// Positions child `Elements` in layers on top of the parent
    Layers(Vec<ElementKey>),
    /// Positions child `Elements` in rows on top of the parent
    Rows {
        children: Vec<Section>,
        spacing: Option<Values>,
    },
    /// Positions child `Elements` in columns on top of the parent
    Columns {
        children: Vec<Section>,
        spacing: Option<Values>,
    },

    /// Element has no children
    #[default]
    None,
}

/// Describes allocated space for a child `Element` inside rows/columns
#[derive(Clone, Debug)]
pub struct Section {
    /// Child `Element`
    pub element: ElementKey,
    /// Allocated space
    pub size: Option<Values>,
}

fn rotate_point(point: Point, pivot: Point, angle: f32) -> Point {
    let sin = angle.sin();
    let cos = angle.cos();
    let translated_x = point.x - pivot.x;
    let translated_y = point.y - pivot.y;

    let rotated_x = translated_x * cos - translated_y * sin;
    let rotated_y = translated_x * sin + translated_y * cos;

    Point::new(rotated_x + pivot.x, rotated_y + pivot.y)
}

/// A point on the Gui context
#[derive(Debug, Copy, Clone, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn distance(&self, other: Point) -> f32 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        (x * x + y * y).sqrt()
    }
}

impl From<[f32; 2]> for Point {
    fn from(point: [f32; 2]) -> Self {
        Self {
            x: point[0],
            y: point[1],
        }
    }
}

impl From<(f32, f32)> for Point {
    fn from(point: (f32, f32)) -> Self {
        Self {
            x: point.0,
            y: point.1,
        }
    }
}

impl From<Point> for [f32; 2] {
    fn from(point: Point) -> [f32; 2] {
        [point.x, point.y]
    }
}

impl From<Point> for (f32, f32) {
    fn from(point: Point) -> (f32, f32) {
        (point.x, point.y)
    }
}

impl Point {
    /// Creates new `Point`
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub fn load_texture_from_memory(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[u8],
) -> texture::Texture {
    let img = image::load_from_memory(data).unwrap();
    let img = img.to_rgba8();
    let img = DynamicImage::ImageRgba8(img);
    texture::Texture::from_image(device, queue, &img, None)
}