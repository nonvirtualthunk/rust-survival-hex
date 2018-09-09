use cgmath::Vector3;
use cgmath::Vector2;
use num::Float;
use num;

use interpolation;


pub use itertools::Itertools;
use std::time::Duration;

use pretty_env_logger;

pub use hex::AxialCoord;

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
    fn sign_str(&self) -> Str;
}
impl ToStringWithSign for i32 {
    fn to_string_with_sign(&self) -> String {
        if *self < 0 {
            self.to_string()
        } else {
            format!("+{}", self.to_string())
        }
    }

    fn sign_str(&self) -> Str {
        if *self < 0 {
            "-"
        } else {
            "+"
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

    fn sign_str(&self) -> Str {
        if *self < 0 {
            "-"
        } else {
            "+"
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

    fn sign_str(&self) -> Str {
        if *self < 0.0 {
            "-"
        } else {
            "+"
        }
    }
}
impl ToStringWithSign for f32 {
    fn to_string_with_sign(&self) -> String {
        if *self < 0.0 {
            self.to_string()
        } else {
            format!("+{}", self.to_string())
        }
    }

    fn sign_str(&self) -> Str {
        if *self < 0.0 {
            "-"
        } else {
            "+"
        }
    }
}

pub trait RichString {
    fn capitalized(&self) -> String;
}

impl RichString for String {
    fn capitalized(&self) -> String {
        if self.is_empty() {
            self.clone()
        } else {
            let mut start = self.chars().take(1).collect::<String>().to_uppercase();
            let end = self.chars().skip(1).collect::<String>();

            start.push_str(end.as_str());
            start
        }
    }
}

impl RichString for Str {
    fn capitalized(&self) -> String {
        if self.is_empty() {
            strf(self)
        } else {
            let mut start = self.chars().take(1).collect::<String>().to_uppercase();
            let end = self.chars().skip(1).collect::<String>();

            start.push_str(end.as_str());
            start
        }
    }
}

pub trait ExtendedCollection<T> {
    fn non_empty(&self) -> bool;

    fn map<U, F : Fn(&T) -> U>(&self, func : F) -> Vec<U>;

    fn foreach<F : Fn(&T)>(&self, func : F);

    fn all_match<F : Fn(&T) -> bool>(&self, func : F) -> bool;

    fn any_match<F : Fn(&T) -> bool>(&self, func : F) -> bool;

    fn find<F : Fn(&T) -> bool>(&self, func : F) -> Option<&T>;

    fn extended_by<I: IntoIterator<Item = T>>(self, other : I) -> Self where Self : Sized;
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

    fn find<F : Fn(&T) -> bool>(&self, func : F) -> Option<&T> {
        self.iter().find(|t| (func)(t))
    }

    fn extended_by<I: IntoIterator<Item = T>>(mut self, other : I) -> Self where Self : Sized {
        self.extend(other);
        self
    }
}

pub trait ChopToU32 {
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

#[derive(Clone, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
    Depth
}

pub trait ToMillis {
    fn to_millis(&self) -> f64;
}
impl ToMillis for Duration {
    fn to_millis(&self) -> f64 {
        ((self.as_secs() * 1000) as f64) + self.subsec_millis() as f64
    }
}

pub fn rust_init() {
    pretty_env_logger::try_init().ok();
}



pub fn typename<T>() -> Str { unsafe {::std::intrinsics::type_name::<T>()} }



pub trait OptionalStringArg {
    fn into_string_opt(self) -> Option<String>;
}
impl OptionalStringArg for &'static str {
    fn into_string_opt(self) -> Option<String> {
        Some(String::from(self))
    }
}
impl OptionalStringArg for Option<String> {
    fn into_string_opt(self) -> Option<String> {
        self
    }
}
impl OptionalStringArg for String {
    fn into_string_opt(self) -> Option<String> {
        Some(self)
    }
}


#[macro_export] macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}