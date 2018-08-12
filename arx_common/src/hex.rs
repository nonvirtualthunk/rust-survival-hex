use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Deref;

use prelude::*;

use noisy_float::prelude::R32;

use datastructures::PerfectHashable;
use std::fmt;


const AXIAL_DELTAS: [AxialCoord; 6] =
    [AxialCoord { q: 1, r: 0 }, AxialCoord { q: 1, r: -1 }, AxialCoord { q: 0, r: -1 },
        AxialCoord { q: -1, r: 0 }, AxialCoord { q: -1, r: 1 }, AxialCoord { q: 0, r: 1 }];

const _CUBE_DELTAS: [CubeCoord; 6] =
    [CubeCoord { x: 1, y: -1, z: 0 }, CubeCoord { x: 1, y: 0, z: -1 }, CubeCoord { x: 0, y: 1, z: -1 },
        CubeCoord { x: -1, y: 1, z: 0 }, CubeCoord { x: -1, y: 0, z: 1 }, CubeCoord { x: 0, y: -1, z: 1 }];

pub fn axial_delta(n: usize) -> &'static AxialCoord {
    &AXIAL_DELTAS[n]
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct AxialCoord {
    pub q: i32,
    pub r: i32
}
impl fmt::Display for AxialCoord {
    fn fmt(&self, f: & mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({},{})", self.q, self.r)
    }
}

#[derive(Copy,Clone,PartialEq,Debug,Add,Sub,Mul)]
pub struct CartVec(pub Vec2f);
impl Deref for CartVec {
    type Target = Vec2f;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl CartVec {
    pub fn new(x:f32, y:f32) -> CartVec {
        CartVec(v2(x,y))
    }

    pub fn normalize(&self) -> CartVec {
        let magnitude_squared = self.0.x * self.0.x + self.0.y * self.0.y;
        if magnitude_squared != 0.0 {
            let magnitude = magnitude_squared.sqrt();
            CartVec(v2(self.0.x / magnitude, self.0.y / magnitude))
        } else {
            CartVec(v2(0.0,0.0))
        }
    }
}

impl AxialCoord {
    pub fn as_cube_coord(&self) -> CubeCoord {
        CubeCoord { x: self.q, y: -self.q - self.r, z: self.r }
    }
    pub fn neighbor(&self, n: usize) -> AxialCoord {
        self + axial_delta(n)
    }
    pub fn neighbors_vec(&self) -> Vec<AxialCoord> {
        vec![AxialCoord { q: self.q + 1, r: self.r }, AxialCoord { q: self.q + 1, r: self.r -1 }, AxialCoord { q: self.q, r: self.r-1 },
        AxialCoord { q: self.q -1, r: self.r }, AxialCoord { q: self.q-1, r: self.r+1 }, AxialCoord { q: self.q, r: self.r+1 }]
    }
    pub fn neighbors(&self) -> [AxialCoord; 6] {
        [AxialCoord { q: self.q + 1, r: self.r }, AxialCoord { q: self.q + 1, r: self.r -1 }, AxialCoord { q: self.q, r: self.r-1 },
             AxialCoord { q: self.q -1, r: self.r }, AxialCoord { q: self.q-1, r: self.r+1 }, AxialCoord { q: self.q, r: self.r+1 }]
    }
    pub fn distance(&self, other: &AxialCoord) -> R32 {
        R32::from_f32(((self.q - other.q).abs()
            + (self.q + self.r - other.q - other.r).abs()
            + (self.r - other.r).abs()) as f32 / 2.0)
    }
    pub fn new(q: i32, r: i32) -> AxialCoord {
        AxialCoord { q, r }
    }
    pub fn as_cartesian(&self, size : f32) -> Vec2f {
        v2(self.q as f32 * 1.5 * size, (self.r as f32 + self.q as f32/2.0) as f32 * 1.73205080757 * size)
    }
    pub fn as_cart_vec(&self) -> CartVec {
        CartVec(self.as_cartesian(1.0))
    }
    pub fn rounded(&self) -> AxialCoord {
        CubeCoord::rounded(self.q as f32, self.r as f32).as_axial_coord()
    }
    pub fn from_cartesian(pixel : &Vec2f, size : f32) -> AxialCoord {
        let q = (pixel.x * 0.666666666667) / size;
        let r = ((-pixel.x / 3.0) + (3.0f64.sqrt() as f32/3.0) * pixel.y) / size;
        return CubeCoord::rounded(q, r).as_axial_coord()
    }
    pub fn from_cart_coord(coord : CartVec) -> AxialCoord {
        AxialCoord::from_cartesian(coord.deref(), 1.0)
    }
}

impl<'a, 'b> Add<&'b AxialCoord> for &'a AxialCoord {
    type Output = AxialCoord;

    fn add(self, rhs: &'b AxialCoord) -> Self::Output {
        AxialCoord::new(self.q + rhs.q, self.r + rhs.r)
    }
}

impl Add<AxialCoord> for AxialCoord {
    type Output = AxialCoord;

    fn add(self, rhs: AxialCoord) -> Self::Output {
        AxialCoord::new(self.q + rhs.q, self.r + rhs.r)
    }
}

impl PerfectHashable for AxialCoord {
    fn hash(&self) -> usize {
        ((self.q+250) * 500 + (self.r+250)) as usize
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct CubeCoord {
    x: i32,
    y: i32,
    z: i32
}

impl CubeCoord {
    pub fn new(x: i32, y: i32, z: i32) -> CubeCoord {
        CubeCoord { x, y, z }
    }
    pub fn distance(&self, other: &CubeCoord) -> f32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()) as f32 / 2.0
    }
    pub fn as_axial_coord(&self) -> AxialCoord {
        AxialCoord::new(self.x, self.z)
    }
    pub fn rounded(q : f32, r : f32) -> CubeCoord {
        let x = q;
        let y = -q - r;
        let z = r;

        let mut rx = x.round();
        let mut ry = y.round();
        let mut rz = z.round();

        let x_diff = (rx - x).abs();
        let y_diff = (ry - y).abs();
        let z_diff = (rz - z).abs();

        if x_diff > y_diff && x_diff > z_diff {
            rx = -ry-rz
        } else if y_diff > z_diff {
            ry = -rx-rz
        } else {
            rz = -rx-ry
        }
        return CubeCoord::new(rx as i32, ry as i32, rz as i32)
    }
}

#[test]
fn test_basic_functionality() {
    let ax1 = AxialCoord::new(3,3);
    let cu1 = ax1.as_cube_coord();
    assert_eq!(ax1, cu1.as_axial_coord());

    let ax2 = AxialCoord::new(1,2);
    let ax3 = ax1 + ax2;
    assert_eq!(ax3, AxialCoord::new(4,5));

    let ax4 = ax1 + ax3;
    assert_eq!(ax4, AxialCoord::new(7,8));

    let cart1 = ax4.as_cartesian(1.0);
    assert_eq!(ax4, AxialCoord::from_cartesian(&cart1, 1.0));
}