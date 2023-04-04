use std::fmt;
use std::fmt::Display;

#[derive(Clone)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn square(side_size: u32) -> Self {
        Dimensions {
            width: side_size,
            height: side_size,
        }
    }
}

impl Display for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

pub struct RGBColor {
    r: u8,
    g: u8,
    b: u8,
}

impl RGBColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl Display for RGBColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
    }
}

pub struct Offset {
    pub x: i32,
    pub y: i32,
}

impl Offset {
    pub fn from_y(y: i32) -> Self {
        Offset { x: 0, y }
    }

    fn default() -> Offset {
        Offset { x: 0, y: 0 }
    }
}

fn sign(v: i32) -> char {
    if v < 0 {
        '-'
    } else {
        '+'
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            sign(self.x),
            self.x.abs(),
            sign(self.y),
            self.y.abs()
        )
    }
}

pub struct Extent {
    pub size: Dimensions,
    pub offset: Offset,
}

impl Extent {
    pub fn no_offset(size: &Dimensions) -> Self {
        Self {
            size: size.clone(),
            offset: Offset::default(),
        }
    }
}

impl Display for Extent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.size, self.offset)
    }
}
