use core::GameEventClock;

use prelude::*;
use common::hex::AxialCoord;
use std::fmt::Debug;

#[derive(Clone,Copy)]
pub struct GameEventWrapper<E : GameEventType> {
    pub occurred_at : GameEventClock,
    pub event: E,
    pub state : GameEventState
}

impl <E: GameEventType> GameEventWrapper<E> {
    pub fn is_ended(&self) -> bool {
        self.state == GameEventState::Ended
    }
    pub fn is_starting(&self) -> bool {
        self.state == GameEventState::Started
    }
    pub fn event_and_state(evt : E, state : GameEventState) -> GameEventWrapper<E> {
        GameEventWrapper {
            occurred_at : 0,
            event : evt,
            state
        }
    }
    pub fn if_ended(&self) -> Option<&E> {
        if self.is_ended() {
            Some(&self.event)
        } else {
            None
        }
    }
    pub fn if_starting(&self) -> Option<&E> {
        if self.is_starting() {
            Some(&self.event)
        } else {
            None
        }
    }
}

pub trait GameEventType : Clone + Debug {
    fn beginning_of_time_event() -> Self;
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum GameEventState {
    Started,
    Continuing,
    Ended,
}

impl GameEventState {
    pub fn is_ended(&self) -> bool {
        if let GameEventState::Ended = self {
            true
        } else {
            false
        }
    }
}


#[derive(Clone,Copy,Debug)]
pub enum CoreEvent {
    WorldInitialized,
    EffectEnded,
    TimePassed,
    EntityAdded(Entity),
    EntityRemoved(Entity),
}

impl GameEventType for CoreEvent {
    fn beginning_of_time_event() -> Self {
        CoreEvent::WorldInitialized
    }
}