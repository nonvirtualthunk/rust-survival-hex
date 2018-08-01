use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;

#[derive(Clone,Copy,Debug)]
pub struct Color(pub [f32; 4]);

impl Color {
    pub fn new (r:f32, g:f32, b:f32, a : f32) -> Color {
        Color([r,g,b,a])
    }

    pub fn white() -> Color {
        Color::new(1.0,1.0,1.0,1.0)
    }
    pub fn light_grey() -> Color {
        Color::new(0.8,0.8,0.8,1.0)
    }
    pub fn greyscale(brightness : f32) -> Color {
        Color::new(brightness,brightness,brightness,1.0)
    }
    pub fn black() -> Color {
        Color::new(0.0,0.0,0.0,1.0)
    }
    pub fn clear() -> Color {
        Color::new(1.0,1.0,1.0,0.0)
    }

    pub fn r(&self) -> f32 {
        self.0[0]
    }
    pub fn g(&self) -> f32 {
        self.0[1]
    }
    pub fn b(&self) -> f32 {
        self.0[2]
    }
    pub fn a(&self) -> f32 {
        self.0[3]
    }

    pub fn with_a(&self, a : f32) -> Color { Color::new(self.r(), self.g(), self.b(), a) }
}

impl Default for Color {
    fn default() -> Self {
        Color::white()
    }
}

impl<'a, 'b> Add<&'b Color> for &'a Color {
    type Output = Color;

    fn add(self, rhs: &'b Color) -> Self::Output {
        Color([self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3]])
    }
}
impl Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        Color([self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3]])
    }
}
impl<'a, 'b> Sub<&'b Color> for &'a Color {
    type Output = Color;

    fn sub(self, rhs: &'b Color) -> Self::Output {
        Color([self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
            self.0[3] - rhs.0[3]])
    }
}
impl Sub<Color> for Color {
    type Output = Color;

    fn sub(self, rhs: Color) -> Self::Output {
        Color([self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
            self.0[3] - rhs.0[3]])
    }
}
impl<'a> Mul<f64> for &'a Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Self::Output {
        Color([self.0[0] * rhs as f32,
            self.0[1] * rhs as f32,
            self.0[2] * rhs as f32,
            self.0[3] * rhs as f32])
    }
}
impl Mul<f64> for Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Self::Output {
        Color([self.0[0] * rhs as f32,
            self.0[1] * rhs as f32,
            self.0[2] * rhs as f32,
            self.0[3] * rhs as f32])
    }
}

impl<'a> Mul<f32> for &'a Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Color([self.0[0] * rhs,
            self.0[1] * rhs,
            self.0[2] * rhs,
            self.0[3] * rhs])
    }
}
impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Color([self.0[0] * rhs,
            self.0[1] * rhs,
            self.0[2] * rhs,
            self.0[3] * rhs])
    }
}


impl<'a> Mul<Color> for &'a Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color([self.0[0] * rhs.0[0],
            self.0[1] * rhs.0[1],
            self.0[2] * rhs.0[2],
            self.0[3] * rhs.0[3]])
    }
}
impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color([self.0[0] * rhs.0[0],
            self.0[1] * rhs.0[1],
            self.0[2] * rhs.0[2],
            self.0[3] * rhs.0[3]])
    }
}

impl PartialEq<Color> for Color {
    fn eq(&self, other: &Color) -> bool {
        self.0[0] == other.0[0] && self.0[1] == other.0[1] && self.0[2] == other.0[2] && self.0[3] == other.0[3]
    }
}