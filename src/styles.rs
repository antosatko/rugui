//! `Element` styles

use std::{default, sync::Arc};

use crate::Point;
use bytemuck::Zeroable;

use crate::texture::Texture;
#[derive(Debug, Clone)]
pub struct StyleSheet {
    /// Transform of the element
    pub(crate) transform: Transform,
    /// Background of the element
    ///
    /// Each type of background can applied once
    ///
    /// Order of rendering:
    /// 1. Texture
    /// 3. LinGradient
    /// 4. RadGradient
    /// 2. Color
    ///
    /// Background is rgba(0, 0, 0, 0) by default
    pub(crate) background: Background,
    /// Border of the element
    ///
    /// Not implemented yet
    pub(crate) border: Border,

    pub(crate) text: Text,
    /// Visibility of the element
    ///
    /// If false, the element and its children will not be rendered
    pub(crate) visible: bool,

    pub(crate) alpha: f32,

    /// Hint for the gui engine that this element can be selected for keyboard input
    ///
    /// This allows the element to be selected using tab/arrows (if not consumed)
    pub selectable: bool,
    pub z_index: i32,

    pub(crate) flags: Flags,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub size: (Size, Side),
    pub color: Color,
    pub justify: Position,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            size: (Size::Pixel(20.0), Side::Max),
            color: Color::default(),
            justify: Position::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Transform {
    /// Rotation of the element
    ///
    /// Rotation will be inherited by children
    ///
    /// Not implemented yet
    pub rotation: Rotation,
    /// Position of the element relative to its parent
    pub position: Position,
    /// Alignment of the element relative to itself
    pub align: Position,
    /// Width of the element
    ///
    /// Can be overridden by min_width and max_width
    pub width: Size,
    /// Maximum width of the element that the element can not go above under any circumstances
    pub max_width: Size,
    /// Minimum width of the element that the element can not go below under any circumstances
    pub min_width: Size,
    /// Height of the element
    ///
    /// Can be overridden by min_height and max_height
    pub height: Size,
    /// Maximum height of the element that the element can not go above under any circumstances
    pub max_height: Size,
    /// Minimum height of the element that the element can not go below under any circumstances
    pub min_height: Size,
    /// Margin of the element
    ///
    /// Margin is the space between the element and its parent
    pub margin: Size,
    /// Padding of the element
    ///
    /// Padding is the space between the element and its children
    ///
    /// Not implemented yet
    pub padding: Size,
    /// Performs rounding operation for the position of the element
    ///
    /// Use this with scale_round to render with pixel precision
    pub position_round: Round,
    /// Performs rounding operation for the scale of the element
    ///
    /// Use this with position_round to render with pixel precision
    pub scale_round: Round,
    /// Performs rounding operation for the rotation of the element
    ///
    /// This will alwas force element into right angles
    pub rotation_round: Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Round {
    Ceil,
    Round,
    Floor,
    #[default]
    None,
}

#[derive(Debug)]
pub struct Flags {
    pub(crate) dirty_color: bool,
    pub(crate) dirty_texture: bool,
    pub(crate) dirty_lin_gradient: bool,
    pub(crate) dirty_rad_gradient: bool,
    pub(crate) dirty_text: bool,
    pub(crate) dirty_transform: bool,
    pub(crate) dirty_border: bool,
    pub(crate) dirty_alpha: bool,
    pub(crate) dirty_edges: bool,

    pub(crate) recalc_transform: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            dirty_color: true,
            dirty_texture: true,
            dirty_lin_gradient: true,
            dirty_rad_gradient: true,
            dirty_text: true,
            dirty_transform: true,
            dirty_border: true,
            dirty_alpha: true,
            dirty_edges: true,

            recalc_transform: true,
        }
    }
}

impl Clone for Flags {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self {
            transform: Transform {
                rotation: Rotation::None,
                position: Position::Center,
                align: Position::Center,
                width: Size::Fill,
                max_width: Size::None,
                min_width: Size::None,
                height: Size::Fill,
                max_height: Size::None,
                min_height: Size::None,
                margin: Size::None,
                padding: Size::None,
                position_round: Default::default(),
                scale_round: Default::default(),
                rotation_round: Default::default(),
            },
            background: Background {
                color: Color::zeroed(),
                texture: None,
                lin_gradient: None,
                rad_gradient: None,
                edges: Edges::default(),
            },
            border: Border {
                background: Background {
                    color: Color::zeroed(),
                    texture: None,
                    lin_gradient: None,
                    rad_gradient: None,
                    edges: Edges::default(),
                },
                width: Size::None,
                min_width: Size::None,
                max_width: Size::None,
                radius: Size::None,
                min_radius: Size::None,
                max_radius: Size::None,
                visible: false,
            },
            alpha: 1.0,
            text: Text::default(),
            visible: true,
            selectable: false,
            flags: Flags::default(),
            z_index: 0,
        }
    }
}

impl StyleSheet {
    pub fn get_width(&self, parent_width: f32, window_width: f32) -> f32 {
        let w = match self.transform.width {
            Size::Fill => parent_width,
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            Size::None => parent_width,
            Size::AbsFill => window_width,
            Size::AbsPercent(percent) => window_width * (percent / 100.),
        };
        let min = match self.transform.min_width {
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            Size::AbsFill => window_width,
            Size::AbsPercent(percent) => window_width * (percent / 100.),
            _ => 0.0,
        };
        let max = match self.transform.max_width {
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            Size::AbsFill => window_width,
            Size::AbsPercent(percent) => window_width * (percent / 100.),
            _ => std::f32::INFINITY,
        };
        let margin = match self.transform.margin {
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            Size::AbsPercent(percent) => window_width * (percent / 100.),
            _ => 0.0,
        };
        let result = (w - margin).min(max).max(min);
        match self.transform.scale_round {
            Round::Ceil => result.ceil(),
            Round::Round => result.round(),
            Round::Floor => result.floor(),
            Round::None => result,
        }
    }

    pub fn get_height(&self, parent_height: f32, window_height: f32) -> f32 {
        let h = match self.transform.height {
            Size::Fill => parent_height,
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            Size::None => parent_height,
            Size::AbsFill => window_height,
            Size::AbsPercent(percent) => window_height * (percent / 100.),
        };
        let min = match self.transform.min_height {
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            Size::AbsFill => window_height,
            Size::AbsPercent(percent) => window_height * (percent / 100.),
            _ => 0.0,
        };
        let max = match self.transform.max_height {
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            Size::AbsFill => window_height,
            Size::AbsPercent(percent) => window_height * (percent / 100.),
            _ => std::f32::INFINITY,
        };
        let margin = match self.transform.margin {
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            Size::AbsPercent(percent) => window_height * (percent / 100.),
            _ => 0.0,
        };
        let result = (h - margin).min(max).max(min);
        match self.transform.scale_round {
            Round::Ceil => result.ceil(),
            Round::Round => result.round(),
            Round::Floor => result.floor(),
            Round::None => result,
        }
    }

    pub fn get_x(&self, parent_x: f32, parent_width: f32, width: f32) -> f32 {
        let x = match self.transform.position {
            Position::BottomLeft | Position::Left | Position::TopLeft => {
                parent_x - parent_width / 2.0
            }
            Position::Bottom | Position::Center | Position::Top => parent_x,
            Position::BottomRight | Position::Right | Position::TopRight => {
                parent_x + parent_width / 2.0
            }
            Position::Custom(x, _) => match x {
                Size::Pixel(x) => parent_x + x,
                Size::Percent(percent) => parent_x + parent_width * (percent / 100.),
                _ => parent_x,
            },
        };
        let align = match self.transform.align {
            Position::BottomLeft | Position::Left | Position::TopLeft => width / 2.0,
            Position::Bottom | Position::Center | Position::Top => 0.0,
            Position::BottomRight | Position::Right | Position::TopRight => -width / 2.0,
            Position::Custom(x, _) => match x {
                Size::Pixel(x) => x,
                Size::Percent(percent) => width * (percent / 100.),
                _ => 0.0,
            },
        };

        let result = x + align;
        match self.transform.scale_round {
            Round::Ceil => result.ceil(),
            Round::Round => result.round(),
            Round::Floor => result.floor(),
            Round::None => result,
        }
    }

    pub fn get_y(&self, parent_y: f32, parent_height: f32, height: f32) -> f32 {
        let y = match self.transform.position {
            Position::TopLeft | Position::Top | Position::TopRight => parent_y - height / 2.0,
            Position::Left | Position::Center | Position::Right => parent_y,
            Position::BottomLeft | Position::Bottom | Position::BottomRight => {
                parent_y + height / 2.0
            }
            Position::Custom(_, y) => match y {
                Size::Pixel(y) => parent_y + y,
                Size::Percent(percent) => parent_y + parent_height * (percent / 100.),
                _ => parent_y,
            },
        };
        let align = match self.transform.align {
            Position::TopLeft | Position::Top | Position::TopRight => height / 2.0,
            Position::Left | Position::Center | Position::Right => 0.0,
            Position::BottomLeft | Position::Bottom | Position::BottomRight => -height / 2.0,
            Position::Custom(_, y) => match y {
                Size::Pixel(y) => y,
                Size::Percent(percent) => height * (percent / 100.),
                _ => 0.0,
            },
        };

        let result = y + align;
        match self.transform.scale_round {
            Round::Ceil => result.ceil(),
            Round::Round => result.round(),
            Round::Floor => result.floor(),
            Round::None => result,
        }
    }

    pub fn get_transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transfomr_mut(&mut self) -> &mut Transform {
        self.flags.recalc_transform = true;
        &mut self.transform
    }

    pub fn bg_color(&self) -> &Color {
        &self.background.color
    }

    pub fn bg_color_mut(&mut self) -> &mut Color {
        self.flags.dirty_color = true;
        &mut self.background.color
    }

    pub fn get_bg_texture(&self) -> Option<Arc<Texture>> {
        self.background.texture.clone()
    }

    pub fn set_bg_texture(&mut self, texture: Option<Arc<Texture>>) {
        self.flags.dirty_texture = true;
        self.background.texture = texture;
    }

    pub fn get_bg_lin_gradient(&self) -> Option<LinearGradient> {
        self.background.lin_gradient.clone()
    }

    pub fn set_bg_lin_gradient(&mut self, lin_gradient: Option<LinearGradient>) {
        self.flags.dirty_lin_gradient = true;
        self.background.lin_gradient = lin_gradient;
    }

    pub fn get_bg_rad_gradient(&self) -> Option<RadialGradient> {
        self.background.rad_gradient.clone()
    }

    pub fn set_bg_rad_gradient(&mut self, rad_gradient: Option<RadialGradient>) {
        self.flags.dirty_rad_gradient = true;
        self.background.rad_gradient = rad_gradient;
    }

    pub fn get_text(&self) -> &Text {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Text {
        self.flags.dirty_text = true;
        &mut self.text
    }

    pub fn get_border(&self) -> &Border {
        &self.border
    }

    pub fn border_mut(&mut self) -> &mut Border {
        self.flags.dirty_border = true;
        &mut self.border
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn get_text_size(&self) -> &(Size, Side) {
        &self.text.size
    }

    pub fn text_size_mut(&mut self) -> &mut (Size, Side) {
        self.flags.dirty_text = true;
        &mut self.text.size
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
        self.flags.dirty_alpha = true;
    }

    pub fn get_alpha(&mut self) -> f32 {
        self.alpha
    }

    pub fn get_edges(&self) -> &Edges {
        &self.background.edges
    }

    pub fn edges_mut(&mut self) -> &mut Edges {
        self.flags.dirty_edges = true;
        &mut self.background.edges
    }
}

/// Position of the element relative to its parent
#[derive(Default, Debug, Clone)]
pub enum Position {
    Top,
    TopLeft,
    TopRight,
    Right,
    Bottom,
    BottomRight,
    BottomLeft,
    Left,
    #[default]
    Center,
    Custom(Size, Size),
}

impl Position {
    pub fn normalized(&self, scale: Point) -> [f32; 2] {
        match self {
            Position::Top => [0.5, 0.0],
            Position::TopLeft => [0.0, 0.0],
            Position::TopRight => [1.0, 0.0],
            Position::Right => [1.0, 0.5],
            Position::Bottom => [0.5, 1.0],
            Position::BottomRight => [1.0, 1.0],
            Position::BottomLeft => [0.0, 1.0],
            Position::Left => [0.0, 0.5],
            Position::Center => [0.5, 0.5],
            Position::Custom(x, y) => {
                let x = match x {
                    Size::Pixel(x) => (*x + scale.x * 0.5) / scale.x,
                    Size::Percent(percent) => *percent / 100.0,
                    _ => 0.5,
                };
                let y = match y {
                    Size::Pixel(y) => (*y + scale.y * 0.5) / scale.y,
                    Size::Percent(percent) => *percent / 100.0,
                    _ => 0.5,
                };
                [x, y]
            }
        }
    }
}

/// Border of the element
///
/// Not implemented yet
#[derive(Debug, Clone, Default)]
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

#[derive(Clone, Copy, Debug, Default)]
/// Size of the element
///
/// Size is the width or height of the element
pub enum Size {
    None,
    #[default]
    Fill,
    Pixel(f32),
    Percent(f32),
    AbsFill,
    AbsPercent(f32),
}

#[derive(Clone, Copy, Default, Debug)]
/// Chooses which side should be used in a calculation
///
/// This helps for styles that apply for both width and height
pub enum Side {
    Width,
    Height,
    #[default]
    Max,
    Min,
}

#[derive(Clone, Copy, Debug, Default)]
/// Rotation of the element
///
/// Not implemented yet
pub enum Rotation {
    #[default]
    None,
    AbsNone,
    Deg(f32),
    Rad(f32),
    Percent(f32),
    AbsDeg(f32),
    AbsRad(f32),
    AbsPercent(f32),
}

/// Background of the element
///
/// Each type of background can applied once
///
/// Order of rendering:
/// 1. Texture
/// 3. LinGradient
/// 4. RadGradient
/// 2. Color
///
/// Background is rgba(0, 0, 0, 0) by default
#[derive(Debug, Clone, Default)]
pub struct Background {
    /// Color of the element
    ///
    /// Color is rgba(0, 0, 0, 0) by default
    ///
    /// Color is rendered last. If any other kind of background is applied,
    /// the color will be rendered last and can be used as a tint
    pub color: Color,
    pub texture: Option<Arc<Texture>>,
    /// Linear gradient of the element
    pub lin_gradient: Option<LinearGradient>,
    /// Radial gradient of the element
    pub rad_gradient: Option<RadialGradient>,
    pub edges: Edges,
}

#[derive(Debug, Clone)]
pub struct Edges {
    pub radius: (Size, Side),
    pub smooth: (Size, Side),
}

impl Default for Edges {
    fn default() -> Self {
        Self {
            radius: (Size::None, Side::Min),
            smooth: (Size::None, Side::Min),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ColorPoint {
    pub position: Position,
    pub color: Color,
}

#[derive(Debug, Clone, Default)]
pub struct LinearGradient {
    pub p1: ColorPoint,
    pub p2: ColorPoint,
}

#[derive(Debug, Clone, Default)]
pub struct RadialGradient {
    pub center: ColorPoint,
    pub radius: ColorPoint,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const GRAY: Self = Self {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };

    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const YELLOW: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const CYAN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const MAGENTA: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn with_alpha(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    pub fn with_red(&self, r: f32) -> Self {
        Self {
            r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }

    pub fn with_green(&self, g: f32) -> Self {
        Self {
            r: self.r,
            g,
            b: self.b,
            a: self.a,
        }
    }

    pub fn with_blue(&self, b: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b,
            a: self.a,
        }
    }
}

impl From<[f32; 4]> for Color {
    fn from(array: [f32; 4]) -> Self {
        Self {
            r: array[0],
            g: array[1],
            b: array[2],
            a: array[3],
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> [f32; 4] {
        [color.r, color.g, color.b, color.a]
    }
}


mod styles_proposition {
    struct Styles {

    }

    #[derive(Debug, Default, Clone, Copy)]
    enum Rotation {
        Deg(f32),
        Rad(f32),
        Right(u32),
        #[default]
        None,
    }
    
    /// Returns value
    enum Values {
        /// Perform an operation
        Expr(Box<Expression>),
        /// return a value
        Value(Value),

        /// `None` usually defaults to `Not applicable` or `0`
        /// 
        /// If the value has to be something, it defaults to `Fill`
        None,
    }
    
    /// Performs an operation
    struct Expression {
        /// Left side of operation
        left: Values,
        /// Right side of operation
        right: Values,
        /// Operator
        op: Op,
    }

    /// Choose measured unit
    pub enum Value {
        /// This is the space that is given to the element
        Container(RValue, Side),
        /// This is the space that is given to the first element
        ViewPort(RValue, Side),
        /// This is the space that the image fits into
        Image(RValue, Side),
        /// This is the smallest space that the text fits into
        Text(RValue, Side),
        /// This is the smallest space that the content fits into
        /// 
        /// returns based on criteria:
        /// 1. image, text - max(image, text)
        /// 2. image - image
        /// 3. text - text
        /// 4. ___ - [1px, 1px]
        Content(RValue, Side),
        /// Size in pixels
        Pixel(f32),

        /// `None` usually defaults to filling available space
        None,
    }

    /// Returns size of a specified side/equation of the measured unit
    pub enum Side {
        /// Returns width of the measured unit
        Width,
        /// Returns height of the measured unit
        Height,
        /// Returns the diameter of the measured unit
        Diameter,

        /// Returns result of `max(Width, Height)`
        Max,
        /// Returns result of `min(Width, Height)`
        Min,
        /// Returns result of `Width + Height`
        Sum,
        /// Returns result of `abs(Width - Height)`
        Distance,
        /// Reutrns result of `(Widht + Height) / 2`
        Midpoint,
    }

    /// Performs operation on size
    pub enum RValue {
        /// Returns a percentage of size `(size / 100) * Percent`
        Percent(f32),
        /// Returns a fraction of size `size * Fraction`
        Fraction(f32),
        /// Returns half of size `size / 2`
        Half,
        /// Returns whole `size`
        Full,
    }

    pub enum Op {
        Add,
        Sub,
        Mul,
        Div,
        Mod,
        Min,
        Max,
        Pow,
        Sqrt,
        Round,
        Floor,
        Ceil,
    }
}