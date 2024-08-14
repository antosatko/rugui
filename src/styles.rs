use std::sync::Arc;

use bytemuck::Zeroable;

use crate::{render::Color, texture::Texture};

pub struct StyleSheet {
    pub rotation: Rotation,
    pub position: Position,
    pub width: Size,
    pub max_width: Size,
    pub min_width: Size,
    pub height: Size,
    pub max_height: Size,
    pub min_height: Size,
    pub background: Background,
    pub border: Border,
    pub margin: Size,
    pub padding: Size,
    pub visible: bool,
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self {
            rotation: Rotation::None,
            position: Position::Center,
            width: Size::Fill,
            max_width: Size::None,
            min_width: Size::None,
            height: Size::Fill,
            max_height: Size::None,
            min_height: Size::None,
            background: Background {
                color: Color::zeroed(),
                texture: None,
                lin_gradient: None,
                rad_gradient: None,
            },
            border: Border {
                background: Background {
                    color: Color::zeroed(),
                    texture: None,
                    lin_gradient: None,
                    rad_gradient: None,
                },
                width: Size::None,
                min_width: Size::None,
                max_width: Size::None,
                radius: Size::None,
                min_radius: Size::None,
                max_radius: Size::None,
                visible: false,
            },
            margin: Size::Pixel(0.0),
            padding: Size::Pixel(0.0),
            visible: true,
        }
    }
}

impl StyleSheet {
    pub fn get_width(&self, parent_width: f32) -> f32 {
        let w = match self.width {
            Size::Fill => parent_width,
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            Size::None => parent_width,
        };
        let min = match self.min_width {
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            _ => 0.0,
        };
        let max = match self.max_width {
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            _ => std::f32::INFINITY,
        };
        let margin = match self.margin {
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            _ => 0.0,
        };
        (w - margin).max(min).min(max)
    }

    pub fn get_height(&self, parent_height: f32) -> f32 {
        let h = match self.height {
            Size::Fill => parent_height,
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            Size::None => parent_height,
        };
        let min = match self.min_height {
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            _ => 0.0,
        };
        let max = match self.max_height {
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            _ => std::f32::INFINITY,
        };
        let margin = match self.margin {
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            _ => 0.0,
        };
        (h - margin).max(min).min(max)
    }
}

pub enum Position {
    Top,
    TopLeft,
    TopRight,
    Right,
    Bottom,
    BottomRight,
    BottomLeft,
    Left,
    Center,
    Custom(Size, Size),
}

pub struct Border {
    pub background: Background,
    pub width: Size,
    pub min_width: Size,
    pub max_width: Size,
    pub radius: Size,
    pub min_radius: Size,
    pub max_radius: Size,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Size {
    None,
    Fill,
    Pixel(f32),
    Percent(f32),
}

pub enum Rotation {
    None,
    Deg(f32),
    Rad(f32),
    Percent(f32),
}

pub struct Background {
    pub color: Color,
    pub texture: Option<Arc<Texture>>,
    pub lin_gradient: Option<LinGradient>,
    pub rad_gradient: Option<RadGradient>,
}

pub struct LinGradient {
    pub p1: (Position, Color),
    pub p2: (Position, Color),
}

pub struct RadGradient {
    pub p1: (Position, Color),
    pub p2: (Position, Color),
}