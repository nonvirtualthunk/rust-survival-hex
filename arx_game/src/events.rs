use core::GameEventClock;

use entities::*;
use common::hex::AxialCoord;

#[derive(Clone,Copy)]
pub struct GameEventWrapper {
    pub occurred_at : GameEventClock,
    pub data : GameEvent
}

#[derive(Clone,Copy, Debug)]
pub enum GameEvent {
    Move { character : CharacterRef, from : AxialCoord, to : AxialCoord },
    Attack { attacker : CharacterRef, defender : CharacterRef, damage_done : u32, hit : bool, hit_chance : f64, killing_blow : bool },
    Equip { character : CharacterRef, item : ItemRef },
    TurnStart { turn_number : u32 },
    WorldStart
}