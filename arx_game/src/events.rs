use core::GameEventClock;

use prelude::*;
use common::hex::AxialCoord;
use std::fmt::Debug;

#[derive(Clone,Copy,Serialize,Deserialize,Debug)]
pub struct GameEventWrapper<E : GameEventType> {
    pub occurred_at : GameEventClock,
    pub event: E,
    state : GameEventState
}

impl <E: GameEventType> GameEventWrapper<E> {
    pub fn new(evt : E, state : GameEventState, occurred_at : GameEventClock) -> GameEventWrapper<E> {
        GameEventWrapper {
            occurred_at,
            event : evt,
            state
        }
    }
    pub fn is_ended(&self) -> bool {
        self.state.is_ended()
    }
    pub fn is_starting(&self) -> bool {
        self.state.is_starting()
    }
    pub fn is_continuing(&self) -> bool { self.state.is_continuing() }
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

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub enum GameEventState {
    Started,
    Continuing,
    Ended,
    StartedAndEnded
}

impl GameEventState {
    pub fn is_ended(&self) -> bool {
        self == &GameEventState::Ended || self == &GameEventState::StartedAndEnded
    }
    pub fn is_starting(&self) -> bool {
        self == &GameEventState::Started || self == &GameEventState::StartedAndEnded
    }
    pub fn is_continuing(&self) -> bool {
        self == &GameEventState::Continuing
    }
}


#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
pub enum CoreEvent {
    WorldInitialized,
    EffectEnded,
    TimePassed,
    Recomputation,
    EntityAdded(Entity),
    EntityRemoved(Entity),
    DataRegistered,
    Default
}
impl Default for CoreEvent {
    fn default() -> Self {
        CoreEvent::Default
    }
}

impl GameEventType for CoreEvent {
    fn beginning_of_time_event() -> Self {
        CoreEvent::WorldInitialized
    }
}