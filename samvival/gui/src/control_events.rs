use game::prelude::*;
use game::entities::actions::ActionType;
use game::entities::reactions::ReactionType;
use game::entities::combat::AttackRef;
use action_bar::PlayerActionType;


pub enum ControlEvents {
    ActionSelected(PlayerActionType),
    ReactionSelected(ReactionType),
    AttackSelected(AttackRef),
    CounterattackSelected(AttackRef),
    ItemTransferRequested { item : Entity, from : Vec<Entity>, to : Vec<Entity> },
    EquipItemRequested { item : Entity, equip_on : Entity },
}