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

pub type Str = &'static str;

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

pub trait ExtendedCollection<T> {
    fn non_empty(&self) -> bool;

    fn map<U, F : Fn(&T) -> U>(&self, func : F) -> Vec<U>;

    fn foreach<F : Fn(&T)>(&self, func : F);

    fn all_match<F : Fn(&T) -> bool>(&self, func : F) -> bool;

    fn any_match<F : Fn(&T) -> bool>(&self, func : F) -> bool;
}

impl <T> ExtendedCollection<T> for Vec<T> {
    fn non_empty(&self) -> bool {
        ! self.is_empty()
    }

    fn map<U, F: Fn(&T) -> U>(&self, func: F) -> Vec<U> {
        self.iter().map(func).collect_vec()
    }

    fn foreach<F : Fn(&T)>(&self, func : F) {
        self.iter().foreach(func)
    }

    fn all_match<F : Fn(&T) -> bool>(&self, func : F) -> bool { self.iter().all(func) }

    fn any_match<F : Fn(&T) -> bool>(&self, func : F) -> bool { self.iter().any(func) }
}

trait ChopToU32 {
    fn as_u32_or_0(&self) -> u32;
}
impl ChopToU32 for i32 {
    fn as_u32_or_0(&self) -> u32 {
        if *self < 0 {
            0
        } else {
            (*self) as u32
        }
    }
}


pub enum Orientation {
    Horizontal,
    Vertical,
    Depth
}