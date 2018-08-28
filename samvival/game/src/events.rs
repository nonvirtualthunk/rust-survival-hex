use game::prelude::*;
use common::hex::*;
use entities::combat::AttackRef;
use entities::combat::DamageType;
use entities::reactions::ReactionType;
use entities::combat::AttackType;
use entities::combat::Attack;
use entities::combat::StrikeResult;


#[derive(Clone, Debug)]
pub enum GameEvent {
    Move { character : Entity, from : AxialCoord, to : AxialCoord, cost : Sext },
    EntityAppears { entity: Entity, at : AxialCoord },
    DamageTaken { entity : Entity, damage_taken : u32, damage_types : Vec<DamageType> },
    EntityDied { entity : Entity },
    Strike { attacker : Entity, defender : Entity, attack : Box<Attack>, strike_result : Box<StrikeResult> },
    Attack { attacker : Entity, defender : Entity },
    Equip { character : Entity, item : Entity },
    Unequip { character : Entity, item : Entity },
    RemoveFromInventory { item : Entity, from_inventory : Entity },
    AddToInventory { item : Entity, to_inventory : Entity } ,
    TurnStart { turn_number : u32 },
    FactionTurn { turn_number : u32, faction : Entity },
    EffectEnded { entity : Option<Entity> },
    WorldStart,
    SelectedAttackChanged { entity : Entity, attack_ref : AttackRef },
    SelectedCounterattackChanged { entity : Entity, attack_ref : AttackRef },
    SelectedReactionChanged { entity : Entity, reaction_type : ReactionType },
    ReactionEffectApplied { entity : Entity }
}

impl GameEventType for GameEvent {
    fn beginning_of_time_event() -> Self {
        GameEvent::WorldStart
    }
}