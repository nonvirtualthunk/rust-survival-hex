use cgmath::Vector3;
use cgmath::Vector2;
use num::Float;
use num;

use interpolation;


pub use itertools::Itertools;

pub type Vec3i = Vector3<i32>;
pub type Vec3f = Vector3<f32>;

pub type Vec2i = Vector2<i32>;
pub type Vec2f = Vector2<f32>;

pub fn v3<T>(x: T, y: T, z: T) -> Vector3<T> {
    Vector3 { x, y, z }
}

pub fn v2<T>(x: T, y: T) -> Vector2<T> {
    Vector2 { x, y }
}

pub fn as_f64(v: Vec2f) -> [f64; 2] {
    [v.x as f64, v.y as f64]
}

pub fn strf<'a>(raw: &'a str) -> String {
    String::from(raw)
}

// Maps the input [0,1] to [0,1,0]
pub fn circlerp<T : Float + num::FromPrimitive>(linfract : T) -> T {
    let linfract = linfract.to_f64().expect("impossibly, failed to convert to f64");
    let ret = if linfract < 0.5 {
        linfract * 2.0
    } else {
        1.0 - (linfract - 0.5) * 2.0
    };
    num::FromPrimitive::from_f64(ret).unwrap()
}


pub trait ToStringWithSign {
    fn to_string_with_sign(&self) -> String;
}
impl ToStringWithSign for i32 {
    fn to_string_with_sign(&self) -> String {
        if *self < 0 {
            self.to_string()
        } else {
            format!("+{}", self.to_string())
        }
    }
}
impl ToStringWithSign for i64 {
    fn to_string_with_sign(&self) -> String {
        if *self < 0 {
            self.to_string()
        } else {
            format!("+{}", self.to_string())
        }
    }
}
impl ToStringWithSign for f64 {
    fn to_string_with_sign(&self) -> String {
        if *self < 0.0 {
            self.to_string()
        } else {
            format!("+{}", self.to_string())
        }
    }
}

pub trait ExtendedCollection {
    fn non_empty(&self) -> bool;
}

impl <T> ExtendedCollection for Vec<T> {
    fn non_empty(&self) -> bool {
        ! self.is_empty()
    }
}