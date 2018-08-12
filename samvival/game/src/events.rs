use game::prelude::*;
use common::hex::*;
use entities::combat::AttackReference;
use entities::combat::DamageType;
use entities::reactions::ReactionType;


#[derive(Clone, Debug)]
pub enum GameEvent {
    Move { character : Entity, from : AxialCoord, to : AxialCoord },
    EntityAppears { entity: Entity, at : AxialCoord },
    DamageTaken { entity : Entity, damage_taken : u32, damage_types : Vec<DamageType> },
    EntityDied { entity : Entity },
    Strike { attacker : Entity, defender : Entity, damage_done : u32, hit : bool, killing_blow : bool, strike_number : u8 },
    Attack { attacker : Entity, defender : Entity },
    Equip { character : Entity, item : Entity },
    Unequip { character : Entity, item : Entity },
    TurnStart { turn_number : u32 },
    FactionTurn { turn_number : u32, faction : Entity },
    EffectEnded { entity : Option<Entity> },
    WorldStart,
    SelectedAttackChanged { entity : Entity, attack_ref : AttackReference },
    SelectedReactionChanged { entity : Entity, reaction_type : ReactionType },
    ReactionEffectApplied { entity : Entity }
}

impl GameEventType for GameEvent {
    fn beginning_of_time_event() -> Self {
        GameEvent::WorldStart
    }
}