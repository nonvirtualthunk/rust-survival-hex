use std::ops;
use rand::Rng;
use std::convert::From;

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
}

#[derive(Clone, Debug)]
pub struct DiceRoll {
    pub pool : DicePool,
    pub die_results : Vec<u32>,
    pub total_result : u32
}