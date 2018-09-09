use std::ops;
use std::convert::From;
use rand::Rng;
use std::convert::Into;
use itertools::Itertools;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::u64;
use std::fmt::Debug;
use common::prelude::ExtendedCollection;
use serde::Serialize;
use serde::Deserialize;
use serde::Serializer;
use serde::Deserializer;
use serde::de::Visitor;

//use num;

pub type GameEventClock = u64;
pub const MAX_GAME_EVENT_CLOCK : GameEventClock = u64::MAX;


pub trait ReduceableType: ops::Sub<Output=Self> + ops::Add<Output=Self> + Copy + Default + Into<f64> + PartialOrd<Self> {}

impl<T> ReduceableType for T where T: ops::Sub<Output=T> + ops::Add<Output=T> + Copy + Default + Into<f64> + PartialOrd<Self> {}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reduceable<T: ReduceableType> {
    base_value: T,
    reduced_by: T,
}

impl <T : ReduceableType> Display for Reduceable<T> where T : Display {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.cur_value())
    }
}

impl<T: ReduceableType> Reduceable<T> {
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
    pub fn reduce_by(&mut self, by: T) { self.reduced_by = self.reduced_by + by; }
    pub fn reduced_by(&self, by : T) -> Self { Reduceable { base_value : self.base_value, reduced_by : self.reduced_by + by } }
    pub fn recover_by(&mut self, by: T) {
        self.reduced_by = self.reduced_by - by;
        let zero = T::default();
        if self.reduced_by < zero {
            self.reduced_by = zero;
        }
    }
    pub fn recovered_by(&self, by: T) -> Self {
        let mut new_reduced_by = self.reduced_by - by;
        let zero = T::default();
        if new_reduced_by < zero {
            new_reduced_by = zero;
        }
        Reduceable { base_value : self.base_value, reduced_by : new_reduced_by }
    }
    pub fn reduce_to(&mut self, to: T) { self.reduced_by = self.base_value - to; }
    pub fn reduced_to(&self, to: T)  -> Self { Reduceable { base_value : self.base_value, reduced_by : self.base_value - to } }
    pub fn increase_by(&mut self, by : T) { self.base_value = self.base_value + by; }
    pub fn decrease_by(&mut self, by : T) { self.base_value = self.base_value - by; }
    pub fn reset(&mut self) { self.reduced_by = T::default(); }
    pub fn cur_fract(&self) -> f64 { self.cur_value().into() / self.max_value().into() }
    pub fn cur_reduced_by(&self) -> T { self.reduced_by }
}


#[derive(Clone, Debug, PartialEq)]
pub enum DicePool {
    Single { die : u32, count : u32 },
    Compound(Vec<DicePool>),
    None
}

impl Serialize for DicePool {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
        serializer.serialize_str(self.to_d20_string().as_str())
    }
}
impl <'de> Deserialize<'de> for DicePool {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        struct DicePoolVisitor;
        impl <'de> Visitor<'de> for DicePoolVisitor {
            type Value = DicePool;
            fn expecting(&self, formatter: &mut Formatter) -> Result<(), Error> {write!(formatter,"XdY")}

            fn visit_str<E>(self, v: &str) -> Result<DicePool, E> where E: serde::de::Error { match DicePool::from_str(v) { Some(dp) => Ok(dp), None => Err(E::custom("Could not parse dice pool")) } }
            fn visit_string<E>(self, v: String) -> Result<DicePool, E> where E: serde::de::Error { match DicePool::from_str(v) { Some(dp) => Ok(dp), None => Err(E::custom("Could not parse dice pool")) } }
        }
        deserializer.deserialize_str(DicePoolVisitor)
    }
}

impl Default for DicePool { fn default() -> Self { DicePool::none() } }

impl Display for DicePool {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.to_d20_string())
    }
}
impl ::common::ToStringWithSign for DicePool {
    fn to_string_with_sign(&self) -> String {
        format!("+ {}", self.to_d20_string())
    }

    fn sign_str(&self) -> &'static str {
        "+"
    }
}

impl DicePool {
    pub fn of(count: u32, die: u32) -> DicePool {
        DicePool::Single {
            die,
            count,
        }
    }
    pub fn none() -> DicePool {
        DicePool::None
    }

    pub fn from_str<S: Into<String>>(string: S) -> Option<DicePool> {
        let string = string.into();
        if string == "none" {
            Some(DicePool::none())
        } else if string.contains("+") {
            let sub_dice_iter = string.split("+").map(|s| DicePool::from_str(s));
            if sub_dice_iter.clone().all(|d| d.is_some()) {
                Some(DicePool::Compound(sub_dice_iter.flat_map(|i| i).collect_vec()))
            } else {
                None
            }
        } else {
            let parts = string.split("d").collect_vec();
            if parts.len() != 2 {
                None
            } else {
                let count: Result<u32, _> = parts[0].parse::<u32>();
                let die: Result<u32, _> = parts[1].parse::<u32>();
                if count.is_ok() && die.is_ok() {
                    Some(DicePool::of(count.unwrap(), die.unwrap()))
                } else {
                    None
                }
            }
        }
    }

    pub fn roll<T: Rng>(&self, rng: &mut T) -> DiceRoll {
        let mut res: Vec<u32> = vec!();
        let mut total = 0u32;

        match self {
            DicePool::Single { die, count } => {
                for _ in 0..*count {
                    let val = rng.gen_range(1, *die + 1);
                    res.push(val);
                    total += val;
                }
            },
            DicePool::Compound(pools) => {
                for pool in pools {
                    let DiceRoll { die_results, total_result, pool : _ } = pool.roll(rng);
                    res.extend(die_results);
                    total += total_result
                }
            },
            DicePool::None => {}
        }



        DiceRoll {
            pool: self.clone(),
            die_results: res,
            total_result: total,
        }
    }

    pub fn avg_roll(&self) -> f32 {
        match self {
            DicePool::Single { die, count } => ((*die + 1) as f32) * 0.5 * *count as f32,
            DicePool::Compound(pools) => pools.iter().map(|p| p.avg_roll()).sum(),
            DicePool::None => { 0.0f32 }
        }
    }

    pub fn min_roll(&self) -> u32 {
        match self {
            DicePool::Single { die, count } => *count,
            DicePool::Compound(pools) => pools.iter().map(|p| p.min_roll()).sum(),
            DicePool::None => { 0  }
        }
    }

    pub fn max_roll(&self) -> u32 {
        match self {
            DicePool::Single { die, count } => die * count,
            DicePool::Compound(pools) => pools.iter().map(|p| p.max_roll()).sum(),
            DicePool::None => { 0 }
        }
    }

    pub fn to_d20_string(&self) -> String {
        match self {
            DicePool::Single { die, count } => format!("{}d{}", *count, *die),
            DicePool::Compound(pools) => pools.iter().map(|p| p.to_d20_string()).join(" + "),
            DicePool::None => { String::from("none") }
        }
    }
}

impl ::std::ops::Add<DicePool> for DicePool {
    type Output = DicePool;

    fn add(self, rhs: DicePool) -> DicePool {
        if let DicePool::None = rhs {
            self
        } else {
            match self {
                DicePool::Compound(pools) => DicePool::Compound(pools.extended_by(vec![rhs])),
                DicePool::Single { .. } => DicePool::Compound(vec![self, rhs]),
                DicePool::None => rhs,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DiceRoll {
    pub pool: DicePool,
    pub die_results: Vec<u32>,
    pub total_result: u32,
}


#[derive(Clone, Copy, Debug, Add, Sub, Div, AddAssign, SubAssign, MulAssign, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Oct(i64);

impl Oct {
    pub fn of_rounded<T: num::Float>(n: T) -> Oct {
        let f = n.to_f64().expect("somehow, used \"of\" to create an Sext with a type that didn't support it");
        Oct((f * 8.0).round() as i64)
    }
    pub fn of<T: num::Integer + num::ToPrimitive>(n: T) -> Oct {
        Oct(n.to_i64().expect("could not create Sext from value") * 8)
    }
    pub fn of_parts(full: i32, eights: i32) -> Oct {
        Oct((full * 8 + eights) as i64)
    }
    pub fn part(eights: i32) -> Oct {
        Oct(eights as i64)
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

    pub fn ceil(&self) -> i32 { self.as_f32().ceil() as i32 }

    pub fn as_whole_and_parts(&self) -> (i64, i32) {
        (self.0 / 8, (self.0 % 8) as i32)
    }
}

impl Display for Oct {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let (whole, parts) = self.as_whole_and_parts();
        if parts != 0 {
            write!(f, "{} {}/8", whole, parts)
        } else {
            write!(f, "{}", whole)
        }
    }
}

impl<T: num::Integer + num::ToPrimitive> ops::Mul<T> for Oct {
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


impl Into<f64> for Oct {
    fn into(self) -> f64 {
        self.as_f64()
    }
}



#[derive(Clone, Copy, Debug, Add, Sub, Div, AddAssign, SubAssign, MulAssign, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Neg, Serialize, Deserialize)]
pub struct Sext(i64);

impl Sext {
    pub const fn of_int(i : i64) -> Sext {
        Sext(i * 6)
    }

    pub fn of_rounded<T: num::Float>(n: T) -> Sext {
        let f = n.to_f64().expect("somehow, used \"of\" to create an Sext with a type that didn't support it");
        Sext((f * 6.0).round() as i64)
    }
    pub fn of_rounded_up<T: num::Float>(n: T) -> Sext {
        let f = n.to_f64().expect("somehow, used \"of\" to create an Sext with a type that didn't support it");
        Sext((f * 6.0).ceil() as i64)
    }
    pub fn of<T: num::Integer + num::ToPrimitive>(n: T) -> Sext {
        Sext(n.to_i64().expect("could not create Sext from value") * 6)
    }
    pub fn of_parts(full: i32, eights: i32) -> Sext {
        Sext((full * 6 + eights) as i64)
    }
    pub fn part(eights: i32) -> Sext {
        Sext(eights as i64)
    }


    pub const fn zero() -> Sext {
        Sext(0)
    }

    pub fn as_f32(&self) -> f32 {
        (self.0 as f32) / 6.0
    }

    pub fn as_f64(&self) -> f64 {
        (self.0 as f64) / 6.0
    }

    pub fn as_u32_or_0(&self) -> u32 {
        self.floor().max(0.0) as u32
    }

    pub fn round(&self) -> i32 {
        ((self.0 as f64) / 6.0).round() as i32
    }

    pub fn floor(&self) -> f64 {
        ((self.0 as f64) / 6.0).floor()
    }

    pub fn as_i32(&self) -> i32 {
        self.floor() as i32
    }

    pub fn ceil(&self) -> i32 { self.as_f32().ceil() as i32 }

    pub fn as_whole_and_parts(&self) -> (i64, i32) {
        (self.0 / 6, (self.0 % 6) as i32)
    }
}

impl Display for Sext {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let (whole, parts) = self.as_whole_and_parts();
        if parts != 0 {
            write!(f, "{} and {}/6", whole, parts)
        } else {
            write!(f, "{}", whole)
        }
    }
}

impl<T: num::Integer + num::ToPrimitive> ops::Mul<T> for Sext {
    type Output = Sext;
    fn mul(self, rhs: T) -> Self::Output {
        Sext(self.0 * rhs.to_i64().expect("expected to be able to convert RHS to i64"))
    }
}

impl ops::Mul<Sext> for Sext {
    type Output = Sext;
    fn mul(self, rhs: Sext) -> Self::Output {
        Sext((self.0 * rhs.0) / 6)
    }
}


impl Into<f64> for Sext {
    fn into(self) -> f64 {
        self.as_f64()
    }
}


#[derive(Default)]
pub struct IntFract(i64, i64);




#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Progress<T: PartialOrd + Default + Clone + Debug> {
    pub current: T,
    pub required: T,
}

impl<T: PartialOrd + Default + Clone + Debug> Progress<T> {
    pub fn is_complete(&self) -> bool {
        !(self.current < self.required)
    }
    pub fn not_complete(&self) -> bool {
        !self.is_complete()
    }
    pub fn with_current(&self, t: T) -> Progress<T> {
        Progress {
            current: t,
            required: self.required.clone(),
        }
    }
    pub fn new(current : T, required : T) -> Progress<T> {
        Progress { current, required }
    }
}


#[test]
pub fn test_oct_arithmetic() {
    assert_eq!((Oct::of(3) * Oct::of(4)).round(), 12);
    assert_eq!((Oct::of(2) + Oct::of_parts(1, 2) * 4).round(), 7);
}