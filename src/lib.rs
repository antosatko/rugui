use std::{collections::HashMap, sync::Arc};

use events::{EventPoll, EventResponse, EventTypes, WindowEvent};
use image::{DynamicImage, GenericImage, ImageBuffer, Rgba};
use nalgebra::Point2;
use render::{
    GpuBound, RenderElement, RenderLinearGradient, RenderRadialGradient,
};
use styles::{Size, StyleSheet};

pub mod events;
pub mod render;
pub mod styles;
pub mod texture;

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
    pub debug: bool,
}

struct InputState {
    mouse: Point2<f32>,
    prev_mouse: Point2<f32>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse: Point2::new(0.0, 0.0),
            prev_mouse: Point2::new(0.0, 0.0),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ElementKey {
    id: u64,
}

impl<Msg> Gui<Msg>
where
    Msg: Clone,
{
    pub fn new(size: (u32, u32), device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
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
            debug: false,
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
                position: Point2::new(self.size.0 as f32 / 2.0, self.size.1 as f32 / 2.0),
                scale: Point2::new(self.size.0 as f32, self.size.1 as f32),
                rotation: 0.0,
            };
            self.element_transform(key, &transform);
        }
    }

    pub fn event(&mut self, event: events::WindowEvent) {
        self.events.queue.push(event);
    }

    fn resolve_events(&mut self) {
        let entry_key = if let Some(entry) = self.entry {
            entry
        } else {
            return;
        };
        while let Some(event) = self.events.queue.pop() {
            match &event {
                WindowEvent::MouseMove { position } => {
                    self.input.prev_mouse = self.input.mouse;
                    self.input.mouse = *position;
                }
                _ => {}
            }
            self.element_event(entry_key, &event);
        }
    }

    fn element_event(&mut self, key: ElementKey, event: &events::WindowEvent) -> EventResponse {
        // propagate event to children
        let element = match self.elements.get_mut(&key) {
            Some(element) => element,
            None => return EventResponse::Ignored,
        };
        if !element.styles.visible {
            return EventResponse::Ignored;
        }
        let mut response = EventResponse::Ignored;
        match element.children.to_owned() {
            Children::Element(child) => {
                response = self.element_event(child, event);
            }
            Children::Layers(children) => {
                for child in children {
                    response = self.element_event(child, event);
                    if response == EventResponse::Consumed {
                        break;
                    }
                }
            }
            Children::Rows { children, .. } => {
                for child in children {
                    response = self.element_event(child.element, event);
                    if response == EventResponse::Consumed {
                        break;
                    }
                }
            }
            Children::Columns { children, .. } => {
                for child in children {
                    response = self.element_event(child.element, event);
                    if response == EventResponse::Consumed {
                        break;
                    }
                }
            }
            Children::None => {}
        }
        if response == EventResponse::Consumed {
            return response;
        }
        let element = match self.elements.get_mut(&key) {
            Some(element) => element,
            None => return EventResponse::Ignored,
        };
        match event {
            events::WindowEvent::MouseDown { button } => {
                let position = self.input.mouse;
                if element.transform.point_collision(position) {
                    if let Some(msg) = element.event_listeners.get(&EventTypes::MouseDown) {
                        self.events.events.push(events::Event {
                            event_type: EventTypes::MouseDown,
                            event: event.clone(),
                            msg: msg.clone(),
                            key,
                        });
                    }
                }
            }
            events::WindowEvent::MouseUp { button } => {
                let position = self.input.mouse;
                if element.transform.point_collision(position) {
                    if let Some(msg) = element.event_listeners.get(&EventTypes::MouseUp) {
                        self.events.events.push(events::Event {
                            event_type: EventTypes::MouseUp,
                            event: event.clone(),
                            msg: msg.clone(),
                            key,
                        });
                    }
                }
            }
            events::WindowEvent::Scroll { delta } => {
                let position = self.input.mouse;
                if element.transform.point_collision(position) {
                    if let Some(msg) = element.event_listeners.get(&EventTypes::Scroll) {
                        todo!();
                    }
                }
            }
            WindowEvent::Input { text } => {
                if let Some(msg) = element.event_listeners.get(&EventTypes::Input) {
                    todo!();
                }
            }
            WindowEvent::SelectNext => {
                if let Some(msg) = element.event_listeners.get(&EventTypes::SelectNext) {
                    self.events.events.push(events::Event {
                        event_type: EventTypes::SelectNext,
                        event: event.clone(),
                        msg: msg.clone(),
                        key,
                    });
                }
            }
            WindowEvent::SelectPrevious => {
                if let Some(msg) = element.event_listeners.get(&EventTypes::SelectPrevious) {
                    self.events.events.push(events::Event {
                        event_type: EventTypes::SelectPrevious,
                        event: event.clone(),
                        msg: msg.clone(),
                        key,
                    });
                }
            }
            events::WindowEvent::MouseMove { .. } => {
                let position = self.input.mouse;
                let prev = self.input.prev_mouse;
                let (this, prev) = (
                    element.transform.point_collision(position),
                    element.transform.point_collision(prev),
                );
                match (this, prev) {
                    (true, false) => {
                        if let Some(msg) = element.event_listeners.get(&EventTypes::MouseEnter) {
                            self.events.events.push(events::Event {
                                event_type: EventTypes::MouseEnter,
                                event: event.clone(),
                                msg: msg.clone(),
                                key,
                            });
                        }
                    }
                    (false, true) => {
                        if let Some(msg) = element.event_listeners.get(&EventTypes::MouseLeave) {
                            self.events.events.push(events::Event {
                                event_type: EventTypes::MouseLeave,
                                event: event.clone(),
                                msg: msg.clone(),
                                key,
                            });
                        }
                    }
                    _ => {}
                }
                if this {
                    if let Some(msg) = element.event_listeners.get(&EventTypes::MouseMove) {
                        self.events.events.push(events::Event {
                            event_type: EventTypes::MouseMove,
                            event: event.clone(),
                            msg: msg.clone(),
                            key,
                        });
                    }
                }
            }
            _ => {}
        };
        EventResponse::Ignored
    }

    fn traverse_elements(&self, key: ElementKey, f: &mut dyn FnMut(&Element<Msg>)) {
        let element = match self.elements.get(&key) {
            Some(element) => element,
            None => return,
        };
        f(element);
        match element.children.to_owned() {
            Children::Element(child) => {
                self.traverse_elements(child, f);
            }
            Children::Layers(children) => {
                for child in children {
                    self.traverse_elements(child, f);
                }
            }
            Children::Rows { children, .. } => {
                for child in children {
                    self.traverse_elements(child.element, f);
                }
            }
            Children::Columns { children, .. } => {
                for child in children {
                    self.traverse_elements(child.element, f);
                }
            }
            Children::None => return,
        }
    }

    fn traverse_elements_mut(&mut self, key: ElementKey, f: &mut dyn FnMut(&mut Element<Msg>)) {
        let element = match self.elements.get_mut(&key) {
            Some(element) => element,
            None => return,
        };
        f(element);
        match element.children.to_owned() {
            Children::Element(child) => {
                self.traverse_elements_mut(child, f);
            }
            Children::Layers(children) => {
                for child in children {
                    self.traverse_elements_mut(child, f);
                }
            }
            Children::Rows { children, .. } => {
                for child in children {
                    self.traverse_elements_mut(child.element, f);
                }
            }
            Children::Columns { children, .. } => {
                for child in children {
                    self.traverse_elements_mut(child.element, f);
                }
            }
            Children::None => return,
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
        if let Some(entry) = self.elements.get_mut(&entry_key) {
            entry.styles.flags.recalc_transform = true;
        }
        self.element_transform(
            *entry_key,
            &ElementTransform {
                position: Point2::new(size.0 as f32 / 2.0, size.1 as f32 / 2.0),
                scale: Point2::new(size.0 as f32, size.1 as f32),
                rotation: 0.0,
            },
        );
    }

    pub fn update(&mut self) {
        self.resolve_events();
        let entry_key = if let Some(entry) = &self.entry {
            entry
        } else {
            return;
        };
        self.element_transform(
            *entry_key,
            &ElementTransform {
                position: Point2::new(self.size.0 as f32 / 2.0, self.size.1 as f32 / 2.0),
                scale: Point2::new(self.size.0 as f32, self.size.1 as f32),
                rotation: 0.0,
            },
        );
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let entry_key = if let Some(entry) = &self.entry {
            entry
        } else {
            return;
        };
        self.traverse_elements_mut(*entry_key, &mut |e| e.write(device, queue))
    }

    fn element_transform(&mut self, key: ElementKey, transform: &ElementTransform) {
        let element = match self.elements.get_mut(&key) {
            Some(element) => element,
            None => return,
        };
        if element.styles.flags.recalc_transform {
            self.traverse_elements_mut(key, &mut |e| {
                e.styles.flags.recalc_transform = true;
            });
        }
        let element = match self.elements.get_mut(&key) {
            Some(element) => element,
            None => return,
        };
        if element.styles.flags.recalc_transform {
            let (width, height) = (
                element
                    .styles
                    .get_width(transform.scale.x, self.size.0 as f32),
                element
                    .styles
                    .get_height(transform.scale.y, self.size.1 as f32),
            );
            let (x, y) = (
                element
                    .styles
                    .get_x(transform.position.x, transform.scale.x, width),
                element
                    .styles
                    .get_y(transform.position.y, transform.scale.y, height),
            );
            let rotation = match element.styles.transform.rotation {
                styles::Rotation::None => transform.rotation,
                styles::Rotation::AbsNone => 0.0,
                styles::Rotation::Deg(deg) => deg.to_radians() + transform.rotation,
                styles::Rotation::Rad(rad) => rad + transform.rotation,
                styles::Rotation::Percent(percent) => {
                    (percent / 50.0) * std::f32::consts::PI + transform.rotation
                }
                styles::Rotation::AbsDeg(deg) => deg.to_radians(),
                styles::Rotation::AbsRad(rad) => rad,
                styles::Rotation::AbsPercent(percent) => (percent / 50.0) * std::f32::consts::PI,
            };
            let transform = ElementTransform {
                position: Point2::new(x, y),
                scale: Point2::new(width, height),
                rotation,
            };
            let pre_collision = element.transform.point_collision(self.input.mouse);

            element.transform = transform;
            element.styles.flags.dirty_transform = true;
            element.styles.flags.recalc_transform = false;

            let post_collision = element.transform.point_collision(self.input.mouse);
            match (pre_collision, post_collision) {
                (true, false) => {
                    if let Some(msg) = element.event_listeners.get(&EventTypes::MouseLeave) {
                        self.events.events.push(events::Event {
                            event_type: EventTypes::MouseLeave,
                            event: WindowEvent::MouseMove {
                                position: self.input.mouse,
                            },
                            msg: msg.clone(),
                            key,
                        });
                    }
                }
                (false, true) => {
                    if let Some(msg) = element.event_listeners.get(&EventTypes::MouseEnter) {
                        self.events.events.push(events::Event {
                            event_type: EventTypes::MouseEnter,
                            event: WindowEvent::MouseMove {
                                position: self.input.mouse,
                            },
                            msg: msg.clone(),
                            key,
                        });
                    }
                }
                _ => {}
            }
        }
        let transform = &element.transform;
        match element.children.to_owned() {
            Children::Element(child) => {
                let (pad_width, pad_height) = match &element.styles.transform.padding {
                    Size::Fill => (element.transform.scale.x, element.transform.scale.y),
                    Size::Pixel(pad) => (*pad, *pad),
                    Size::Percent(pad) => (
                        element.transform.scale.x * (pad / 100.),
                        element.transform.scale.y * (pad / 100.),
                    ),
                    Size::None => (0.0, 0.0),
                    Size::AbsFill => (self.size.0 as f32, self.size.1 as f32),
                    Size::AbsPercent(pad) => (
                        self.size.0 as f32 * (pad / 100.),
                        self.size.1 as f32 * (pad / 100.),
                    ),
                };
                let transform = ElementTransform {
                    scale: Point2::new(
                        transform.scale.x - pad_width,
                        transform.scale.y - pad_height,
                    ),
                    ..transform.clone()
                };
                self.element_transform(child.clone(), &transform);
                return;
            }
            Children::Layers(children) => {
                let (pad_width, pad_height) = match &element.styles.transform.padding {
                    Size::Fill => (element.transform.scale.x, element.transform.scale.y),
                    Size::Pixel(pad) => (*pad, *pad),
                    Size::Percent(pad) => (
                        element.transform.scale.x * (pad / 100.),
                        element.transform.scale.y * (pad / 100.),
                    ),
                    Size::None => (0.0, 0.0),
                    Size::AbsFill => (self.size.0 as f32, self.size.1 as f32),
                    Size::AbsPercent(pad) => (
                        self.size.0 as f32 * (pad / 100.),
                        self.size.1 as f32 * (pad / 100.),
                    ),
                };
                let transform = ElementTransform {
                    scale: Point2::new(
                        transform.scale.x - pad_width,
                        transform.scale.y - pad_height,
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
                        Size::Pixel(space) => space,
                        Size::Percent(space) => transform.scale.y * (space / 100.),
                        Size::Fill => remaining_height,
                        Size::None => remaining_height / len,
                        Size::AbsFill => self.size.1 as f32,
                        Size::AbsPercent(space) => self.size.1 as f32 * (space / 100.),
                    };
                    let position = if transform.rotation == 0.0 {
                        Point2::new(transform.position.x, y + space / 2.0)
                    } else {
                        let pivot = transform.position;
                        let point = Point2::new(transform.position.x, y + space / 2.0);
                        rotate_point(point, pivot, transform.rotation)
                    };
                    let transform = ElementTransform {
                        position,
                        scale: Point2::new(transform.scale.x, space),
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
                        Size::Pixel(space) => space,
                        Size::Percent(space) => transform.scale.x * (space / 100.),
                        Size::Fill => remaining_width,
                        Size::None => remaining_width / len,
                        Size::AbsFill => self.size.0 as f32,
                        Size::AbsPercent(space) => self.size.0 as f32 * (space / 100.),
                    };
                    let position = if transform.rotation == 0.0 {
                        Point2::new(x + space / 2.0, transform.position.y)
                    } else {
                        let pivot = transform.position;
                        let point = Point2::new(x + space / 2.0, transform.position.y);
                        rotate_point(point, pivot, transform.rotation)
                    };
                    let transform = ElementTransform {
                        position,
                        scale: Point2::new(space, transform.scale.y),
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
        let entry_key = match &self.entry {
            Some(entry) => entry,
            None => return,
        };
        pass.set_bind_group(0, &self.gpu.dimensions_bind_group, &[]);

        self.render_element(*entry_key, pass);
    }

    fn render_element<'a>(&'a self, key: ElementKey, pass: &mut wgpu::RenderPass<'a>) {
        let element = match self.elements.get(&key) {
            Some(element) => element,
            None => return,
        };
        if !element.styles.visible {
            return;
        }
        if let Some(render) = &element.render_element {
            render.render(&self.gpu.pipelines, pass)
        }
        match element.children.to_owned() {
            Children::Element(child) => {
                self.render_element(child, pass);
            }
            Children::Layers(children) => {
                for child in children {
                    self.render_element(child, pass);
                }
            }
            Children::Rows { children, .. } => {
                for child in children {
                    self.render_element(child.element, pass);
                }
            }
            Children::Columns { children, .. } => {
                for child in children {
                    self.render_element(child.element, pass);
                }
            }
            Children::None => return,
        }
    }

    /*pub fn texture_from_bytes(&self, bytes: &[u8], label: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> Arc<texture::Texture> {
        Arc::new(texture::Texture::from_bytes(device, queue, bytes, label))
    }

    pub fn texture_from_image(
        &self,
        img: &image::DynamicImage,
        label: Option<&str>,
        device: &wgpu::Device, queue: &wgpu::Queue
    ) -> Arc<texture::Texture> {
        Arc::new(texture::Texture::from_image(device, queue, img, label))
    }

    */
    fn texture_from_image_buffer(
        &self,
        img: ImageBuffer<Rgba<u8>, Vec<u8>>,
        label: Option<&str>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Arc<texture::Texture> {
        let mut image = DynamicImage::new(img.width(), img.height(), image::ColorType::Rgba8);
        for (x, y, pixel) in img.enumerate_pixels() {
            image.put_pixel(x, y, *pixel);
        }
        Arc::new(texture::Texture::from_image(device, queue, &image, label))
    }
}

#[derive(Clone, Debug, Default)]
/// Transformation of a element
///
/// Element transformations are applied to the element and its children
/// when the element is rendered for the first time or when the element
/// or its parent is resized
pub struct ElementTransform {
    /// Position in x and y of the top left corner
    pub position: Point2<f32>,
    /// Scale in width and height
    pub scale: Point2<f32>,
    /// Rotation in radians
    pub rotation: f32,
}

impl ElementTransform {
    pub fn new(position: Point2<f32>, scale: Point2<f32>, rotation: f32) -> Self {
        Self {
            position,
            scale,
            rotation,
        }
    }

    pub fn zeroed() -> Self {
        Self {
            position: Point2::new(0.0, 0.0),
            scale: Point2::new(0.0, 0.0),
            rotation: 0.0,
        }
    }

    pub fn point_collision(&self, point: Point2<f32>) -> bool {
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

#[derive(Default)]
pub struct Element<Msg>
where
    Msg: Clone,
{
    text: Option<String>,
    pub label: Option<String>,
    pub render_element: Option<RenderElement>,
    pub styles: StyleSheet,
    pub event_listeners: HashMap<EventTypes, Msg>,
    pub children: Children,
    transform: ElementTransform,
    parent: ElementTransform,
}

impl<Msg> Element<Msg>
where
    Msg: Clone,
{
    pub fn new() -> Self {
        Self {
            text: None,
            label: None,
            render_element: None,
            styles: StyleSheet::default(),
            event_listeners: HashMap::new(),
            children: Children::None,
            transform: ElementTransform::zeroed(),
            parent: ElementTransform::zeroed(),
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn with_styles(mut self, styles: StyleSheet) -> Self {
        self.styles = styles;
        self
    }

    pub fn with_event_listener(mut self, event: EventTypes, msg: Msg) -> Self {
        self.event_listeners.insert(event, msg);
        self
    }

    pub fn with_children(mut self, children: Children) -> Self {
        self.children = children;
        self
    }

    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if let None = self.render_element {
            self.render_element = Some(RenderElement::zeroed(device))
        }
        let mut render_element = self.render_element.take().unwrap();
        if self.styles.flags.dirty_texture {
            if let Some(texture) = &self.styles.background.texture {
                render_element.set_texture(texture.clone());
            }

            self.styles.flags.dirty_texture = false;
        }
        if self.styles.flags.dirty_color {
            let color = self.styles.background.color;
            render_element.set_color(color, queue, device);
            self.styles.flags.dirty_color = false;
        }
        if self.styles.flags.dirty_transform {
            let transform = &self.transform;
            render_element.data.update_transform(transform);
        }
        if self.styles.flags.dirty_lin_gradient {
            if let Some(grad) = &self.styles.background.lin_gradient {
                match &mut render_element.linear_gradient {
                    Some(lin) => {
                        lin.from_style(grad);
                        lin.write_all(queue);
                    }
                    _ => {
                        let mut lin = RenderLinearGradient::zeroed(device);
                        lin.from_style(grad);
                        lin.write_all(queue);
                        render_element.linear_gradient = Some(lin);
                    }
                }
            }
            self.styles.flags.dirty_lin_gradient = false;
        }
        if self.styles.flags.dirty_rad_gradient {
            if let Some(grad) = &self.styles.background.rad_gradient {
                match &mut render_element.radial_gradient {
                    Some(rad) => {
                        rad.from_style(grad);
                        rad.write_all(queue);
                    }
                    _ => {
                        let mut rad = RenderRadialGradient::zeroed(device);
                        rad.from_style(grad);
                        rad.write_all(queue);
                        render_element.radial_gradient = Some(rad);
                    }
                }
            }
            self.styles.flags.dirty_rad_gradient = false;
        }

        render_element.write_all(queue);
        self.render_element = Some(render_element)
    }

    pub fn text(&self) -> &Option<String> {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Option<String> {
        &mut self.text
    }

    pub fn text_str(&mut self, str: &str) {
        match &mut self.text {
            Some(text) => {
                *text = str.to_string();
            }
            None => {
                self.text = Some(str.to_string());
            }
        }
    }

    pub fn text_string(&mut self, str: String) {
        match &mut self.text {
            Some(text) => {
                *text = str;
            }
            None => {
                self.text = Some(str);
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum Children {
    Element(ElementKey),
    Layers(Vec<ElementKey>),
    Rows {
        children: Vec<Section>,
        spacing: Size,
    },
    Columns {
        children: Vec<Section>,
        spacing: Size,
    },

    #[default]
    None,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub element: ElementKey,
    pub size: Size,
}

fn rotate_point(point: Point2<f32>, pivot: Point2<f32>, angle: f32) -> Point2<f32> {
    let sin = angle.sin();
    let cos = angle.cos();
    let translated_x = point.x - pivot.x;
    let translated_y = point.y - pivot.y;

    let rotated_x = translated_x * cos - translated_y * sin;
    let rotated_y = translated_x * sin + translated_y * cos;

    Point2::new(rotated_x + pivot.x, rotated_y + pivot.y)
}
