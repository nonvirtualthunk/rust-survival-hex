use game::prelude::*;
use common::hex::*;


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
    EffectEnded { entity : Option<Entity> },
    WorldStart
}
impl GameEventType for GameEvent {}