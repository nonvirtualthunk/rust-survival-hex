use core::GameEventClock;

use prelude::*;
use common::hex::AxialCoord;
use std::fmt::Debug;

#[derive(Clone,Copy)]
pub struct GameEventWrapper<E : GameEventType> {
    pub occurred_at : GameEventClock,
    pub data : E
}

pub trait GameEventType : Clone + Copy + Debug {}



#[derive(Clone,Copy,Debug)]
pub enum CoreEvent {
    WorldInitialized,
    EffectEnded,
    TimePassed,
    EntityAdded(Entity),
    EntityRemoved(Entity),
}

impl GameEventType for CoreEvent {}