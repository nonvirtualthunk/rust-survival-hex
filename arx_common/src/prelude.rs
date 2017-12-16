use cgmath::Vector3;
use cgmath::Vector2;

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

pub fn as_f64(v : Vec2f) -> [f64; 2] {
    [v.x as f64, v.y as f64]
}