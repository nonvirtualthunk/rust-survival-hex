use std::ops;
use rand::Rng;
use std::convert::From;

use num;

pub type GameEventClock = u64;


#[derive(Clone, Default, Debug)]
pub struct Reduceable<T: ops::Sub<Output=T> + ops::Add<Output=T> + Copy + Default + Into<f64>> {
    base_value: T,
    reduced_by: T
}

impl<T: ops::Sub<Output=T> + ops::Add<Output=T> + Copy + Default + Into<f64>> Reduceable<T> {
    pub fn value(&self) -> T {
        self.base_value - self.reduced_by
    }

    pub fn new(base_value: T) -> Reduceable<T> {
        Reduceable {
            base_value,
            ..Default::default()
        }
    }

    pub fn max_value(&self) -> T { self.base_value }
    pub fn cur_value(&self) -> T { self.base_value - self.reduced_by }
    pub fn reduce_by(&mut self, by : T) { self.reduced_by = self.reduced_by + by; }
    pub fn reduce_to(&mut self, to : T) { self.reduced_by = self.base_value - to; }
    pub fn reset(&mut self) { self.reduced_by = T::default(); }
    pub fn cur_fract(&self) -> f64 { self.cur_value().into() / self.max_value().into() }
    pub fn cur_reduced_by(&self) -> T { self.reduced_by }
}


#[derive(Clone, Debug, Copy)]
pub struct DicePool {
    pub die : u32,
    pub count : u32,
}
impl Default for DicePool {
    fn default() -> Self {
        DicePool {
            die : 1,
            count : 1
        }
    }
}

impl DicePool {
    pub fn roll<T : Rng>(&self, rng : &mut T) -> DiceRoll {
        let mut res : Vec<u32> = vec!();
        let mut total = 0u32;

        for _ in 0 .. self.count {
            let val = rng.gen_range(1,self.die+1);
            res.push(val);
            total += val;
        }

        DiceRoll {
            pool : self.clone(),
            die_results : res,
            total_result : total
        }
    }

    pub fn to_d20_string(&self) -> String {
        format!("{}d{}", self.count, self.die)
    }
}

#[derive(Clone, Debug)]
pub struct DiceRoll {
    pub pool : DicePool,
    pub die_results : Vec<u32>,
    pub total_result : u32
}


#[derive(Clone, Copy, Debug, Add, Sub, Div, AddAssign, SubAssign, MulAssign, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Oct(i64);

impl Oct {
    pub fn of_rounded<T : num::Float>(n : T) -> Oct {
        let f = n.to_f64().expect("somehow, used \"of\" to create an Oct with a type that didn't support it");
        Oct((f * 8.0).round() as i64)
    }
    pub fn of<T : num::Integer + num::ToPrimitive>(n : T) -> Oct {
        Oct(n.to_i64().expect("could not create oct from value") * 8)
    }
    pub fn of_parts(full : i32, eights : i32) -> Oct {
        Oct((full * 8 + eights) as i64)
    }


    pub fn zero() -> Oct {
        Oct::of(0)
    }

    pub fn as_f32(&self) -> f32 {
        (self.0 as f32) / 8.0
    }

    pub fn as_f64(&self) -> f64 {
        (self.0 as f64) / 8.0
    }

    pub fn as_u32_or_0(&self) -> u32 {
        self.floor().max(0.0) as u32
    }

    pub fn round(&self) -> i32 {
        ((self.0 as f64) / 8.0).round() as i32
    }

    pub fn floor(&self) -> f64 {
        ((self.0 as f64) / 8.0).floor()
    }

    pub fn as_i32(&self) -> i32 {
        self.floor() as i32
    }
}

impl <T : num::Integer + num::ToPrimitive> ops::Mul<T> for Oct {
    type Output = Oct;

    fn mul(self, rhs: T) -> Self::Output {
        Oct(self.0 * rhs.to_i64().expect("expected to be able to convert RHS to i64"))
    }
}
impl ops::Mul<Oct> for Oct {
    type Output = Oct;

    fn mul(self, rhs: Oct) -> Self::Output {
        Oct((self.0 * rhs.0) / 8)
    }
}


#[test]
pub fn test_oct_arithmetic() {
    assert_eq!((Oct::of(3) * Oct::of(4)).round(), 12);
    assert_eq!((Oct::of(2) + Oct::of_parts(1,2) * 4).round(), 7);
}