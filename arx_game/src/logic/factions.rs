use WorldView;
use Entity;
use entities::CharacterStore;


pub fn is_enemy(view : &WorldView, a : Entity, b : Entity) -> bool {
    view.character(a).faction != view.character(b).faction
}

pub fn is_enemy_of_faction(view : &WorldView, faction : Entity, entity : Entity) -> bool {
    view.character(entity).faction != faction
}

