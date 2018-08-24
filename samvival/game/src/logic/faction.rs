use game::WorldView;
use game::Entity;
use entities::CharacterStore;


pub fn is_enemy(view : &WorldView, a : Entity, b : Entity) -> bool {
    view.character(a).allegiance.faction != view.character(b).allegiance.faction
}

pub fn is_enemy_of_faction(view : &WorldView, faction : Entity, entity : Entity) -> bool {
    view.character(entity).allegiance.faction != faction
}

