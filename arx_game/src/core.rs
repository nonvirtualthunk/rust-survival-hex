use std::ops;

pub type GameEventClock = u64;


#[derive(Clone, Default, Debug)]
pub struct Reduceable<T: ops::Sub<Output=T> + ops::Add<Output=T> + Copy + Default> {
    base_value: T,
    reduced_by: T
}

impl<T: ops::Sub<Output=T> + ops::Add<Output=T> + Copy + Default> Reduceable<T> {
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
}