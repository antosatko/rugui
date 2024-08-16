use std::{collections::HashMap, sync::Arc};

use events::{EventResponse, EventTypes};
use nalgebra::Point2;
use render::{Color, GpuBound, LinearGradient, RadialGradient, RenderElement};
use styles::{Position, Size, StyleSheet};

pub mod events;
pub mod render;
pub mod styles;
pub mod texture;

pub struct Gui<Msg>
where
    Msg: Clone,
{
    nodes: HashMap<ElementKey, Element<Msg>>,
    entry: Option<ElementKey>,
    last_key: u64,
    size: (u32, u32),
    gpu: GpuBound,
    pub debug: bool,
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
            nodes: HashMap::new(),
            last_key: 0,
            entry: None,
            size,
            gpu,
            debug: false,
        };
        this
    }

    pub fn add_element(&mut self, element: Element<Msg>) -> ElementKey {
        let key = ElementKey { id: self.last_key };
        self.last_key += 1;
        self.nodes.insert(key, element);
        key
    }

    pub fn remove_node(&mut self, key: ElementKey) {
        self.nodes.remove(&key);
    }

    pub fn get_node(&self, key: ElementKey) -> Option<&Element<Msg>> {
        self.nodes.get(&key).map(|node| node)
    }

    pub fn get_node_mut(&mut self, key: ElementKey) -> Option<&mut Element<Msg>> {
        self.nodes.get_mut(&key).map(|node| node)
    }

    pub fn set_entry(&mut self, key: Option<ElementKey>) {
        if let Some(entry) = self.entry.take() {
            self.remove_node(entry);
        }
        self.entry = key;
        if let Some(key) = key {
            let transform = NodeTransform {
                position: Point2::new(self.size.0 as f32 / 2.0, self.size.1 as f32 / 2.0),
                scale: Point2::new(self.size.0 as f32, self.size.1 as f32),
                rotation: 0.0,
            };
            self.transform_element(key, &transform);
        }
    }

    pub fn event(&mut self, event: events::Event) -> EventResponse<Msg> {
        let entry_key = match &self.entry {
            Some(entry) => entry,
            None => return EventResponse::Ignored,
        };
        let node = match self.nodes.get(entry_key) {
            Some(node) => node,
            None => return EventResponse::Ignored,
        };
        todo!("Handle the event")
    }

    pub fn resize(&mut self, size: (u32, u32), queue: &wgpu::Queue) {
        self.size = size;
        self.gpu.resize((size.0, size.1), queue);
        let entry_key = if let Some(entry) = &self.entry {
            entry
        } else {
            return;
        };
        if self.debug {
            println!("Resizing window: x: {}, y: {}", size.0, size.1);
        }
        self.transform_element(
            *entry_key,
            &NodeTransform {
                position: Point2::new(size.0 as f32 / 2.0, size.1 as f32 / 2.0),
                scale: Point2::new(size.0 as f32, size.1 as f32),
                rotation: 0.0,
            },
        );
    }

    fn transform_element(&mut self, key: ElementKey, transform: &NodeTransform) {
        let node = match self.nodes.get_mut(&key) {
            Some(node) => node,
            None => return,
        };
        let styles = &node.styles;
        let (width, height) = (
            styles.get_width(transform.scale.x),
            styles.get_height(transform.scale.y),
        );
        let (x, y) = (
            styles.get_x(transform.position.x, transform.scale.x, width),
            styles.get_y(transform.position.y, transform.scale.y, height),
        );
        let transform = NodeTransform {
            position: Point2::new(x, y),
            scale: Point2::new(width, height),
            rotation: 0.0,
        };
        let color = styles.background.color;
        match &styles.background.texture {
            Some(texture) => {
                node.render_element.set_texture(texture.clone());
            }
            _ => {}
        }
        if let Some(grad) = &styles.background.rad_gradient {
            node.render_element.set_radial_gradient(grad.clone());
        }
        if let Some(grad) = &styles.background.lin_gradient {
            node.render_element.set_linear_gradient(grad.clone());
        }
        node.render_element.set_color(color, &self.gpu.proxy);
        node.render_element
            .set_transform(&transform, &self.gpu.proxy);
        match node.children.to_owned() {
            Children::Element(child) => {
                let (pad_width, pad_height) = match &node.styles.padding {
                    Size::Fill => (width, height),
                    Size::Pixel(pad) => (*pad, *pad),
                    Size::Percent(pad) => (width * (pad / 100.), height * (pad / 100.)),
                    Size::None => (0.0, 0.0),
                };
                let transform = NodeTransform {
                    position: Point2::new(x, y),
                    scale: Point2::new(width - pad_width, height - pad_height),
                    rotation: 0.0,
                };
                self.transform_element(child.clone(), &transform);
                return;
            }
            Children::Layers(children) => {
                let (pad_width, pad_height) = match &node.styles.padding {
                    Size::Fill => (width, height),
                    Size::Pixel(pad) => (*pad, *pad),
                    Size::Percent(pad) => (width * (pad / 100.), height * (pad / 100.)),
                    Size::None => (0.0, 0.0),
                };
                let transform = NodeTransform {
                    position: Point2::new(x, y),
                    scale: Point2::new(width - pad_width, height - pad_height),
                    rotation: 0.0,
                };
                for child in children {
                    self.transform_element(child, &transform);
                }
            }
            Children::Rows { children, .. } => {
                if children.is_empty() {
                    return;
                }
                let mut len = children.len() as f32;
                let mut remaining_height = height;
                let mut y = y - height / 2.0;
                for Spacing { element, spacing } in children {
                    if remaining_height <= 0.0 {
                        break;
                    }
                    let space = match spacing {
                        Size::Pixel(space) => space,
                        Size::Percent(space) => height * (space / 100.),
                        Size::Fill => remaining_height,
                        Size::None => remaining_height / len,
                    };
                    let transform = NodeTransform {
                        position: Point2::new(x, y + space / 2.0),
                        scale: Point2::new(width, space),
                        rotation: 0.0,
                    };
                    self.transform_element(element, &transform);
                    y += space;
                    remaining_height -= space;
                    len -= 1.0;
                }
            }
            Children::Columns { children, .. } => {
                if children.is_empty() {
                    return;
                }
                let mut len = children.len() as f32;
                let mut remaining_width = width;
                let mut x = x - width / 2.0;
                for Spacing { element, spacing } in children {
                    if remaining_width <= 0.0 {
                        break;
                    }
                    let space = match spacing {
                        Size::Pixel(space) => space,
                        Size::Percent(space) => width * (space / 100.),
                        Size::Fill => remaining_width,
                        Size::None => remaining_width / len,
                    };
                    let transform = NodeTransform {
                        position: Point2::new(x + space / 2.0, y),
                        scale: Point2::new(space, height),
                        rotation: 0.0,
                    };
                    self.transform_element(element, &transform);
                    x += space;
                    remaining_width -= space;
                    len -= 1.0;
                }
            }
            Children::None => return,
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
        let node = match self.nodes.get(&key) {
            Some(node) => node,
            None => return,
        };
        if !node.styles.visible {
            return;
        }
        node.render_element.render(&self.gpu.proxy.pipelines, pass);
        match node.children.to_owned() {
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

    pub fn texture_from_bytes(&self, bytes: &[u8], label: &str) -> Arc<texture::Texture> {
        Arc::new(texture::Texture::from_bytes(&self.gpu.proxy, bytes, label))
    }

    pub fn texture_from_image(
        &self,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Arc<texture::Texture> {
        Arc::new(texture::Texture::from_image(&self.gpu.proxy, img, label))
    }

    pub fn radial_gradient(
        &self,
        center: (Position, Color),
        outer: (Position, Color),
    ) -> RadialGradient {
        let mut grad = RadialGradient::zeroed(&self.gpu.proxy);
        let center_pos = center.0.normalized();
        grad.set_center(center_pos, &self.gpu.proxy);
        let outer_pos = outer.0.normalized();
        let dist = {
            let x = center_pos[0] - outer_pos[0];
            let y = center_pos[1] - outer_pos[1];
            (x * x + y * y).sqrt()
        };
        grad.set_radius(dist, &self.gpu.proxy);
        grad.set_center_color(center.1, &self.gpu.proxy);
        grad.set_outer_color(outer.1, &self.gpu.proxy);
        grad
    }

    pub fn linear_gradient(
        &self,
        start: (Position, Color),
        end: (Position, Color),
    ) -> LinearGradient {
        let mut grad = LinearGradient::zeroed(&self.gpu.proxy);
        let start_pos = start.0.normalized();
        grad.set_start(start_pos, &self.gpu.proxy);
        let end_pos = end.0.normalized();
        grad.set_end(end_pos, &self.gpu.proxy);
        grad.set_start_color(start.1, &self.gpu.proxy);
        grad.set_end_color(end.1, &self.gpu.proxy);
        grad
    }
}

#[derive(Clone, Debug)]
/// Transformation of a node
///
/// Node transformations are applied to the node and its children
/// when the node is rendered for the first time or when the node
/// or its parent is resized
pub struct NodeTransform {
    /// Position in x and y of the top left corner
    pub position: Point2<f32>,
    /// Scale in width and height
    pub scale: Point2<f32>,
    /// Rotation in radians
    pub rotation: f32,
}

pub struct Element<Msg>
where
    Msg: Clone,
{
    pub label: Option<String>,
    pub render_element: RenderElement,
    pub styles: StyleSheet,
    pub event_listeners: HashMap<EventTypes, Msg>,
    pub children: Children,
}

impl<Msg> Element<Msg>
where
    Msg: Clone,
{
    pub fn new(gui: &Gui<Msg>) -> Self {
        Self {
            label: None,
            render_element: RenderElement::zeroed(&gui.gpu.proxy),
            styles: StyleSheet::default(),
            event_listeners: HashMap::new(),
            children: Children::None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
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
}

#[derive(Clone, Debug)]
pub enum Children {
    Element(ElementKey),
    Layers(Vec<ElementKey>),
    Rows {
        children: Vec<Spacing>,
        spacing: Size,
    },
    Columns {
        children: Vec<Spacing>,
        spacing: Size,
    },

    None,
}

#[derive(Clone, Debug)]
pub struct Spacing {
    pub element: ElementKey,
    pub spacing: Size,
}
