use game::prelude::*;
use game::entities::actions::ActionType;
use game::entities::reactions::ReactionType;
use game::entities::reactions::ReactionTypeRef;
use game::entities::combat::AttackRef;
use action_bar::PlayerActionType;
use game::scenario::Scenario;
use game::universe::WorldRef;


#[derive(Clone)]
pub enum TacticalEvents {
    ActionSelected(PlayerActionType),
    CancelActiveAction,
    ReactionSelected(ReactionTypeRef),
    AttackSelected(AttackRef),
    CounterattackSelected(AttackRef),
    ItemTransferRequested { item : Entity, from : Vec<Entity>, to : Vec<Entity> },
    EquipItemRequested { item : Entity, equip_on : Entity },
    Save,
    MainMenu
}


pub enum GameModeEvent {
    Load(String),
    Save(String),
    InitScenario(Box<Scenario>),
    EnterTacticalMode(WorldRef, bool),
    MainMenu,
    Exit
}