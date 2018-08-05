use world::Entity;
use world::EntityData;
use world::WorldView;
use common::hex::AxialCoord;
use core::*;
use common::hex::CartVec;
use common::Color;
use common::prelude::*;

#[derive(Clone, Debug)]
pub struct CharacterData {
    pub faction : Entity,
    pub position: AxialCoord,
    pub graphical_position: Option<CartVec>,
    pub graphical_color: Color,
    pub health: Reduceable<i32>,
    pub action_points: Reduceable<i32>,
    pub move_speed: Oct, // represented in octs
    pub moves: Oct,
    pub stamina: Reduceable<Oct>,
    pub stamina_recovery: Oct,
    pub sprite : String,
    pub name : String
}
impl EntityData for CharacterData {}

pub trait CharacterStore {
    fn character(&self, ent : Entity) -> &CharacterData;
}
impl CharacterStore for WorldView {
    fn character(&self, ent: Entity) -> &CharacterData {
        self.data::<CharacterData>(ent)
    }
}
/*
    Action points : each AP may be turned into movement, or used for an action
    Health : if it reaches zero, character dies
    Move speed : each point represents an addition eighth movement point when turning an AP into move

*/

impl Default for CharacterData {
    fn default() -> Self {
        CharacterData {
            faction : Entity::default(),
            position : AxialCoord::new(0,0),
            health : Reduceable::new(1),
            action_points : Reduceable::new(8),
            moves : Oct::zero(),
            move_speed : Oct::of(1),
            stamina : Reduceable::new(Oct::of(8)),
            stamina_recovery : Oct::of(1),
            sprite : strf("default/defaultium"),
            name : strf("unnamed"),
            graphical_position : None,
            graphical_color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl CharacterData {
    pub fn effective_graphical_pos(&self) -> CartVec {
        self.graphical_position.unwrap_or_else(|| self.position.as_cart_vec())
    }
    pub fn is_alive(&self) -> bool {
        self.health.cur_value() > 0
    }
    pub fn can_act(&self) -> bool { self.action_points.cur_value() > 0 }

    pub fn max_moves_remaining(&self, multiplier : f64) -> Oct {
        self.moves + Oct::of_rounded(self.move_speed.as_f64() * self.action_points.cur_value() as f64 * multiplier)
    }
    pub fn max_moves_per_turn(&self, multiplier : f64) -> Oct {
        self.move_speed * self.action_points.max_value()
    }
}

