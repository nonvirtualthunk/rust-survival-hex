use game::entities::actions::ActionType;
use game::entities::reactions::ReactionType;
use game::entities::combat::AttackReference;


pub enum ControlEvents {
    ActionSelected(ActionType),
    ReactionSelected(ReactionType),
    AttackSelected(AttackReference),
    CounterattackSelected(AttackReference),
}