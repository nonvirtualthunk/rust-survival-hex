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
    EntityAppears { character : Entity, at : AxialCoord },
    Strike { attacker : Entity, defender : Entity, damage_done : u32, hit : bool, killing_blow : bool },
    Attack { attacker : Entity, defender : Entity },
    Equip { character : Entity, item : Entity },
    TurnStart { turn_number : u32 },
    FactionTurnStart { turn_number : u32, faction : Entity },
    FactionTurnEnd { turn_number : u32, faction : Entity },
    WorldStart
}