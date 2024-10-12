//! `Element` styles

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

use std::sync::Arc;

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
            width: StyleComponent::new(Values::Value(Value::Container(RValue::Full, Side::Width))),
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
    pub(crate) fn new_c() -> StyleComponent<Self> {
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
    pub fn calc(&self, container: &Container, _view_port: &ViewPort) -> f32 {
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
            PositionValues::Top => corner!(0.0, -cont.size.y * 0.5), // Centered horizontally, top vertically
            PositionValues::TopLeft => corner!(-cont.size.x * 0.5, -cont.size.y * 0.5),
            PositionValues::TopRight => corner!(cont.size.x * 0.5, -cont.size.y * 0.5),
            PositionValues::Center => corner!(0.0, 0.0),
            PositionValues::Left => corner!(-cont.size.x * 0.5, 0.0),
            PositionValues::Right => corner!(cont.size.x * 0.5, 0.0),
            PositionValues::Bottom => corner!(0.0, cont.size.y * 0.5), // Centered horizontally, bottom vertically
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
            Value::ViewPort(r_value, side) => r_value.calc(side.get_size(view_port.0, view_port.1)),
            Value::Image(r_value, side) => match &contaner.image {
                Some(img) => r_value.calc(side.get_size(img.size.x, img.size.y)),
                None => r_value.calc(side.get_size(contaner.size.x, contaner.size.y)),
            },
            Value::Text(_r_value, _side) => todo!("Ouch thats gonna take a while"),
            Value::Content(_r_value, _side) => todo!("Ouch thats gonna take a while"),
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

    pub(crate) fn calc(
        &self,
        container: &Container,
        view_port: &ViewPort,
    ) -> ((Point, Colors), (Point, Colors)) {
        let p1 = self.p1.position.calc(container, view_port);
        let p2 = self.p2.position.calc(container, view_port);
        ((p1, self.p1.color), (p2, self.p2.color))
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

    pub(crate) fn calc(
        &self,
        container: &Container,
        view_port: &ViewPort,
    ) -> ((Point, Colors), (Point, Colors)) {
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
