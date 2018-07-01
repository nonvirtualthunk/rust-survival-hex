use core::GameEventClock;

use world::Entity;
use entities::*;
use common::hex::AxialCoord;

#[derive(Clone,Copy)]
pub struct GameEventWrapper {
    pub occurred_at : GameEventClock,
    pub data : GameEvent
}

#[derive(Clone,Copy, Debug)]
pub enum GameEvent {
    Move { character : Entity, from : AxialCoord, to : AxialCoord },
    Attack { attacker : Entity, defender : Entity, damage_done : u32, hit : bool, hit_chance : f64, killing_blow : bool },
    Equip { character : Entity, item : Entity },
    TurnStart { turn_number : u32 },
    WorldStart
}