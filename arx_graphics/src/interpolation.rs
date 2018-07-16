use std::ops;

use common::prelude::circlerp;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;

// todo, replace with a trait alias when they implement that
pub trait Interpolateable<T> : ops::Mul<f32, Output=T> + ops::Add<Output=T> + ops::Sub<Output=T> + Clone {}
impl <T> Interpolateable<T> for T where T : ops::Mul<f32, Output=T> + ops::Add<Output=T> + ops::Sub<Output=T> + Clone {}

#[derive(Debug)]
pub struct Interpolation<T: Interpolateable<T>> {
    pub start : T,
    pub delta : T,
    pub interpolation_type : InterpolationType,
    pub circular : bool
}

pub enum InterpolationType {
    Linear,
    Exponential { power : f64 },
    Constant,
    Custom { function : Box<Fn(f64) -> f64> }
}

impl Debug for InterpolationType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            InterpolationType::Linear => write!(f, "Linear"),
            InterpolationType::Exponential {power} => write!(f, "Exponential({})", power),
            InterpolationType::Constant => write!(f, "Constant"),
            InterpolationType::Custom { .. } => write!(f, "Custom")
        }
    }
}

impl <T: Interpolateable<T>> Interpolation<T> {
    pub fn interpolate(&self, fract : f64) -> T {
        match self.interpolation_type {
            InterpolationType::Constant => self.start.clone(),
            _ => {
                let effective_fract = match self.interpolation_type {
                    InterpolationType::Linear => fract,
                    InterpolationType::Exponential { power } => fract.powf(power),
                    InterpolationType::Custom { ref function } => (function)(fract),
                    InterpolationType::Constant => fract // cannot actually be reached
                };
                let effective_fract = if self.circular {
                    circlerp(effective_fract)
                } else {
                    effective_fract
                };

                self.start.clone() + self.delta.clone() * effective_fract as f32
            }
        }
    }


    pub fn constant (start : T) -> Interpolation<T> {
        Interpolation {
            start : start.clone(),
            delta : start.clone() - start.clone(),
            interpolation_type : InterpolationType::Constant,
            circular : false
        }
    }

    pub fn linear_from_endpoints (start : T, end : T) -> Interpolation<T> {
        Interpolation {
            start : start.clone(),
            delta : end.clone() - start.clone(),
            interpolation_type : InterpolationType::Linear,
            circular : false
        }
    }

    pub fn linear_from_delta (start : T, delta : T) -> Interpolation<T> {
        Interpolation {
            start,
            delta,
            interpolation_type : InterpolationType::Linear,
            circular : false
        }
    }

    pub fn exponential_from_endpoints (start : T, end : T, power : f64) -> Interpolation<T> {
        Interpolation {
            start : start.clone(),
            delta : end.clone() - start.clone(),
            interpolation_type : InterpolationType::Exponential { power },
            circular : false
        }
    }

    pub fn exponential_from_delta (start : T, delta : T, power : f64) -> Interpolation<T> {
        Interpolation {
            start,
            delta,
            interpolation_type : InterpolationType::Exponential { power },
            circular : false
        }
    }

    pub fn circular(mut self) -> Self {
        self.circular = true;
        self
    }

    pub fn with_interpolation(mut self, interpolation_type : InterpolationType) -> Self {
        self.interpolation_type = interpolation_type;
        self
    }
}