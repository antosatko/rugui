/*//! `Element` styles

use std::sync::Arc;

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
} */

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

impl From<(f32, f32, f32, f32)> for Color {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        Self { r, g, b, a }
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

pub mod styles_proposition {
    use std::{default, sync::Arc};

    use crate::{rotate_point, texture::Texture, ElementTransform, Point};

    pub struct StyleComponent<S> {
        pub(crate) style: S,
        pub(crate) dirty: bool,
    }

    impl<T> StyleComponent<T> {
        pub fn new(c: T) -> Self {
            Self {
                style: c,
                dirty: true,
            }
        }
    }

    pub struct Styles {
        pub position: StyleComponent<Position>,
        pub width: StyleComponent<Values>,
        pub max_width: StyleComponent<Option<Values>>,
        pub min_width: StyleComponent<Option<Values>>,
        pub height: StyleComponent<Values>,
        pub max_height: StyleComponent<Option<Values>>,
        pub min_height: StyleComponent<Option<Values>>,
        pub rotation: StyleComponent<Rotation>,
        pub bg_color: StyleComponent<Colors>,
        pub bg_texture: StyleComponent<Option<Arc<Texture>>>,
        pub bg_linear_gradient: StyleComponent<Option<LinearGradient>>,
        pub bg_radial_gradient: StyleComponent<Option<RadialGradient>>,
        pub margin: StyleComponent<Values>,
        pub padding: StyleComponent<Values>,
        pub alpha: StyleComponent<f32>,
        pub text_color: StyleComponent<Colors>,
        pub text_size: StyleComponent<Values>,
        pub edges_radius: StyleComponent<Values>,
        pub edges_smooth: StyleComponent<Values>,
        pub visible: bool,
        pub selectable: bool,
        pub z_index: i32,
    }

    impl Default for Styles {
        fn default() -> Self {
            Self {
                position: Position::new_c(),
                width: StyleComponent::new(Values::Value(Value::Container(
                    RValue::Full,
                    Side::Width,
                ))),
                max_width: StyleComponent::new(None),
                min_width: StyleComponent::new(None),
                height: StyleComponent::new(Values::Value(Value::Container(
                    RValue::Full,
                    Side::Height,
                ))),
                max_height: StyleComponent::new(None),
                min_height: StyleComponent::new(None),
                rotation: StyleComponent::new(Rotation::None),
                bg_color: StyleComponent::new(Colors::Rgba(0.0, 0.0, 0.0, 0.0)),
                margin: StyleComponent::new(Values::Value(Value::Zero)),
                padding: StyleComponent::new(Values::Value(Value::Zero)),
                text_color: StyleComponent::new(Colors::BLACK),
                text_size: StyleComponent::new(Values::Value(Value::Pixel(50.0))),
                bg_texture: StyleComponent::new(None),
                bg_linear_gradient: StyleComponent::new(None),
                bg_radial_gradient: StyleComponent::new(None),
                edges_radius: StyleComponent::new(Values::Value(Value::Zero)),
                edges_smooth: StyleComponent::new(Values::Value(Value::Zero)),
                alpha: StyleComponent::new(1.0),
                visible: true,
                selectable: false,
                z_index: 0,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Position {
        pub parent: Parent,
        pub value: PositionValues,
        pub offset: (Option<Values>, Option<Values>),
    }

    impl Position {
        pub (crate) fn new_c() -> StyleComponent<Self> {
            StyleComponent::new(Self {
                parent: Parent::Container,
                value: PositionValues::Center,
                offset: (None, None),
            })
        }

        pub const CTOP: Self = Self {
            value: PositionValues::Top,
            parent: Parent::Container,
            offset: (None, None),
        };
        pub const CCENTER: Self = Self {
            value: PositionValues::Center,
            parent: Parent::Container,
            offset: (None, None),
        };
        pub const CBOTTOM: Self = Self {
            value: PositionValues::Bottom,
            parent: Parent::Container,
            offset: (None, None),
        };
        pub const CLEFT: Self = Self {
            value: PositionValues::Left,
            parent: Parent::Container,
            offset: (None, None),
        };
        pub const CRIGHT: Self = Self {
            value: PositionValues::Right,
            parent: Parent::Container,
            offset: (None, None),
        };
        pub const VPCENTER: Self = Self {
            value: PositionValues::Center,
            parent: Parent::ViewPort,
            offset: (None, None),
        };
        pub const VPTOP: Self = Self {
            value: PositionValues::Top,
            parent: Parent::ViewPort,
            offset: (None, None),
        };
        pub const VPBOTTOM: Self = Self {
            value: PositionValues::Bottom,
            parent: Parent::ViewPort,
            offset: (None, None),
        };
        pub const VPLEFT: Self = Self {
            value: PositionValues::Left,
            parent: Parent::ViewPort,
            offset: (None, None),
        };
        pub const VPRIGHT: Self = Self {
            value: PositionValues::Right,
            parent: Parent::ViewPort,
            offset: (None, None),
        };
        pub fn with_parent(mut self, parent: Parent) -> Self {
            self.parent = parent;
            self
        }
        pub fn with_value(mut self, value: PositionValues) -> Self {
            self.value = value;
            self
        }
        pub fn with_offset(mut self, offset: (Option<Values>, Option<Values>)) -> Self {
            self.offset = offset;
            self
        }
        pub fn with_offset_x(mut self, offset_x: Option<Values>) -> Self {
            self.offset.0 = offset_x;
            self
        }
        pub fn with_offset_y(mut self, offset_y: Option<Values>) -> Self {
            self.offset.1 = offset_y;
            self
        }
    }

    impl Default for Position {
        fn default() -> Self {
            Self {
                parent: Parent::Container,
                value: PositionValues::Center,
                offset: (None, None),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum PositionValues {
        Top,
        TopLeft,
        TopRight,
        Center,
        Left,
        Right,
        Bottom,
        BottomLeft,
        BottomRight,
    }

    #[derive(Debug, Clone)]
    pub enum Parent {
        ViewPort,
        Container,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Colors {
        Rgb(f32, f32, f32),
        Rgba(f32, f32, f32, f32),
        Hsl(f32, f32, f32),
        Cmyk(f32, f32, f32, f32),
    }

    impl Colors {
        pub const TRANSPARENT: Self = Self::Rgba(0.0, 0.0, 0.0, 0.0);
        pub const WHITE: Self = Self::Rgb(1.0, 1.0, 1.0);
        pub const BLACK: Self = Self::Rgb(0.0, 0.0, 0.0);
        pub const RED: Self = Self::Rgb(1.0, 0.0, 0.0);
        pub const GREEN: Self = Self::Rgb(0.0, 1.0, 0.0);
        pub const BLUE: Self = Self::Rgb(0.0, 0.0, 1.0);
        pub const YELLOW: Self = Self::Rgb(1.0, 1.0, 0.0);
        pub const CYAN: Self = Self::Rgb(0.0, 1.0, 1.0);
        pub const MAGENTA: Self = Self::Rgb(1.0, 0.0, 1.0);
        pub const GRAY: Self = Self::Rgb(0.5, 0.5, 0.5);
        pub const LIGHT_GRAY: Self = Self::Rgb(0.75, 0.75, 0.75);
        pub const DARK_GRAY: Self = Self::Rgb(0.25, 0.25, 0.25);

        pub fn to_rgba(&self) -> (f32, f32, f32, f32) {
            match self {
                Colors::Rgb(r, g, b) => (*r, *g, *b, 1.0),
                Colors::Rgba(r, g, b, a) => (*r, *g, *b, *a),
                Colors::Hsl(h, s, l) => Self::hsl_to_rgba(*h, *s, *l),
                Colors::Cmyk(c, m, y, k) => Self::cmyk_to_rgba(*c, *m, *y, *k),
            }
        }

        pub fn hsl_to_rgba(h: f32, s: f32, l: f32) -> (f32, f32, f32, f32) {
            let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
            let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
            let m = l - c / 2.0;
            let (r, g, b) = match h {
                h if h < 60.0 => (c, x, 0.0),
                h if h < 120.0 => (x, c, 0.0),
                h if h < 180.0 => (0.0, c, x),
                h if h < 240.0 => (0.0, x, c),
                h if h < 300.0 => (x, 0.0, c),
                _ => (c, 0.0, x),
            };
            (r + m, g + m, b + m, 1.0)
        }

        pub fn cmyk_to_rgba(c: f32, m: f32, y: f32, k: f32) -> (f32, f32, f32, f32) {
            let r = 1.0 - (c * (1.0 - k) + k);
            let g = 1.0 - (m * (1.0 - k) + k);
            let b = 1.0 - (y * (1.0 - k) + k);
            (r, g, b, 1.0)
        }

        pub fn with_alpha(&self, a: f32) -> Self {
            match *self {
                Colors::Rgb(r, g, b) => Colors::Rgba(r, g, b, a),
                Colors::Rgba(r, g, b, _) => Colors::Rgba(r, g, b, a),
                Colors::Hsl(h, s, l) => {
                    let (r, g, b, _) = Self::hsl_to_rgba(h, s, l);
                    Colors::Rgba(r, g, b, a)
                }
                Colors::Cmyk(c, m, y, k) => {
                    let (r, g, b, _) = Self::cmyk_to_rgba(c, m, y, k);
                    Colors::Rgba(r, g, b, a)
                }
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy)]
    pub enum Rotation {
        Deg(f32),
        Rad(f32),
        #[default]
        None,
        AbsDeg(f32),
        AbsRad(f32),
        AbsNone,
    }

    impl Rotation {
        pub fn calc(&self, container: &Container, view_port: &ViewPort) -> f32 {
            match self {
                Rotation::Deg(deg) => *deg + container.rotation,
                Rotation::Rad(rad) => rad.to_degrees() + container.rotation,
                Rotation::None => container.rotation,
                Rotation::AbsDeg(deg) => *deg,
                Rotation::AbsRad(rad) => rad.to_degrees(),
                Rotation::AbsNone => 0.0,
            }
        }
    }

    /// Returns value
    #[derive(Debug, Clone)]
    pub enum Values {
        /// Perform an operation
        Expr(Box<Expression>),
        /// return a value
        Value(Value),
        /// return the result of the function|
        Function(Box<Function>),
    }

    /// Performs an operation
    #[derive(Debug, Clone)]
    pub struct Expression {
        /// Left side of operation
        left: Values,
        /// Right side of operation
        right: Values,
        /// Operator
        op: Op,
    }

    /// A function
    #[derive(Debug, Clone)]
    pub struct Function {
        value: Values,
        fun: Functions,
    }

    /// Choose measured unit
    #[derive(Debug, Clone)]
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
        /// Shortcut for `Value::Pixel(0.0)`
        Zero,
    }

    /// Returns size of a specified side/equation of the measured unit
    #[derive(Debug, Clone)]
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
        /// Returns result of `Width - Height`
        Distance,
        /// Reutrns result of `(Width + Height) / 2`
        Midpoint,
    }

    /// Performs operation on size
    #[derive(Debug, Clone)]
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

    #[derive(Debug, Clone)]
    pub enum Op {
        Add,
        Sub,
        Mul,
        Div,
        Mod,
        Min,
        Max,
        Pow,
    }

    #[derive(Debug, Clone)]
    pub enum Functions {
        Round,
        Floor,
        Ceil,
        Sqrt,
        Abs,
    }

    impl<S> StyleComponent<S> {
        pub fn get(&self) -> &S {
            &self.style
        }
        pub fn get_mut(&mut self) -> &mut S {
            self.dirty = true;
            &mut self.style
        }
        pub fn set(&mut self, style: S) {
            self.dirty = true;
            self.style = style;
        }
    }

    impl Position {
        pub fn calc(&self, container: &Container, view_port: &ViewPort) -> Point { 
            let cont = match self.parent {
                Parent::Container => container,
                Parent::ViewPort => &Container {
                    image: None,
                    position: Point::new(view_port.0 / 2.0, view_port.1 / 2.0),
                    rotation: 0.0,
                    size: Point::new(view_port.0, view_port.1),
                },
            };
            let offset_x = self
                .offset
                .0
                .as_ref()
                .map(|v| v.calc(container, view_port))
                .unwrap_or(0.0);
            let offset_y = self
                .offset
                .1
                .as_ref()
                .map(|v| v.calc(container, view_port))
                .unwrap_or(0.0);
            macro_rules! corner {
                ($left: expr, $right: expr) => {{
                    let point = Point::new(
                        cont.position.x + $left + offset_x,
                        cont.position.y + $right + offset_y,
                    );
                    rotate_point(point, cont.position, -cont.rotation)
                }};
            }
            let result = match &self.value {
                PositionValues::Top => corner!(0.0, -cont.size.y * 0.5),          // Centered horizontally, top vertically
                PositionValues::TopLeft => corner!(-cont.size.x * 0.5, -cont.size.y * 0.5),
                PositionValues::TopRight => corner!(cont.size.x * 0.5, -cont.size.y * 0.5),
                PositionValues::Center => corner!(0.0, 0.0),
                PositionValues::Left => corner!(-cont.size.x * 0.5, 0.0),
                PositionValues::Right => corner!(cont.size.x * 0.5, 0.0),
                PositionValues::Bottom => corner!(0.0, cont.size.y * 0.5),         // Centered horizontally, bottom vertically
                PositionValues::BottomLeft => corner!(-cont.size.x * 0.5, cont.size.y * 0.5),
                PositionValues::BottomRight => corner!(cont.size.x * 0.5, cont.size.y * 0.5),
            };
            result
        }
    }

    impl Values {
        pub fn calc(&self, container: &Container, view_port: &ViewPort) -> f32 {
            match self {
                Values::Expr(expr) => expr.calc(container, view_port),
                Values::Value(val) => val.calc(container, view_port),
                Values::Function(fun) => fun.fun.calc(fun.value.calc(container, view_port)),
            }
        }
    }

    impl Functions {
        pub fn calc(&self, value: f32) -> f32 {
            match self {
                Functions::Round => value.round(),
                Functions::Floor => value.floor(),
                Functions::Ceil => value.ceil(),
                Functions::Sqrt => value.sqrt(),
                Functions::Abs => value.abs(),
            }
        }
    }

    impl Expression {
        pub fn calc(&self, contaner: &Container, view_port: &ViewPort) -> f32 {
            let left = self.left.calc(contaner, view_port);
            let right = self.right.calc(contaner, view_port);
            self.op.calc(left, right)
        }
    }

    impl Op {
        pub fn calc(&self, left: f32, right: f32) -> f32 {
            match self {
                Op::Add => left + right,
                Op::Sub => left - right,
                Op::Mul => left * right,
                Op::Div => left / right,
                Op::Mod => left % right,
                Op::Min => left.min(right),
                Op::Max => left.max(right),
                Op::Pow => left.powf(right),
            }
        }
    }

    impl Value {
        pub fn calc(&self, contaner: &Container, view_port: &ViewPort) -> f32 {
            match self {
                Value::Container(r_value, side) => {
                    r_value.calc(side.get_size(contaner.size.x, contaner.size.y))
                }
                Value::ViewPort(r_value, side) => {
                    r_value.calc(side.get_size(view_port.0, view_port.1))
                }
                Value::Image(r_value, side) => match &contaner.image {
                    Some(img) => r_value.calc(side.get_size(img.size.x, img.size.y)),
                    None => r_value.calc(side.get_size(contaner.size.x, contaner.size.y)),
                },
                Value::Text(r_value, side) => todo!("Ouch thats gonna take a while"),
                Value::Content(r_value, side) => todo!("Ouch thats gonna take a while"),
                Value::Pixel(num) => *num,
                Value::Zero => 0.0,
            }
        }
    }

    impl RValue {
        pub fn calc(&self, side: f32) -> f32 {
            match self {
                RValue::Percent(p) => side * (p / 100.0),
                RValue::Fraction(f) => side * f,
                RValue::Half => side / 2.0,
                RValue::Full => side,
            }
        }
    }

    impl Side {
        pub fn get_size(&self, width: f32, height: f32) -> f32 {
            match self {
                Side::Width => width,
                Side::Height => height,
                Side::Diameter => (width * width + height * height).sqrt(),
                Side::Max => width.max(height),
                Side::Min => width.min(height),
                Side::Sum => width + height,
                Side::Distance => width - height,
                Side::Midpoint => (width + height) / 2.0,
            }
        }
    }

    pub struct LinearGradient {
        pub p1: ColorPoint,
        pub p2: ColorPoint,
    }

    impl LinearGradient {
        pub fn new(p1: ColorPoint, p2: ColorPoint) -> Self {
            Self { p1, p2 }
        }

        pub(crate) fn calc(&self, container: &Container, view_port: &ViewPort) -> ((Point, Colors), (Point, Colors)) {
            let p1 = self.p1.position.calc(container, view_port);
            let p2 = self.p2.position.calc(container, view_port);
            (
                (p1, self.p1.color),
                (p2, self.p2.color),
            )
        }
    }

    pub struct RadialGradient {
        pub center: ColorPoint,
        pub outer: ColorPoint,
    }

    impl RadialGradient {
        pub fn new(center: ColorPoint, outer: ColorPoint) -> Self {
            Self { center, outer }
        }

        pub(crate) fn calc(&self, container: &Container, view_port: &ViewPort) -> ((Point, Colors), (Point, Colors)) {
            let center = self.center.position.calc(container, view_port);
            let outer = self.outer.position.calc(container, view_port);
            ((center, self.center.color), (outer, self.outer.color))
        }
    }

    pub struct ColorPoint {
        pub position: Position,
        pub color: Colors,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Container {
        pub position: Point,
        pub size: Point,
        pub rotation: f32,
        pub image: Option<Rectangle>,
    }

    impl From<ElementTransform> for Container {
        fn from(transform: ElementTransform) -> Self {
            Self {
                position: transform.position,
                size: transform.scale,
                rotation: transform.rotation,
                image: None,
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct ViewPort(pub f32, pub f32);

    #[derive(Debug, Clone, Copy)]
    pub struct Rectangle {
        pub position: Point,
        pub size: Point,
    }
}
