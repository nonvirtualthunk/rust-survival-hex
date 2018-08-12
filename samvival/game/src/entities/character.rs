use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::hex::AxialCoord;
use game::core::*;
use common::hex::CartVec;
use common::Color;
use common::prelude::*;
use std::ops::Deref;
use noisy_float::types::R32;
use entities::common::PositionData;
use entities::common::ActionData;


#[derive(Default, Clone, Debug, PrintFields)]
pub struct GraphicsData {
    pub graphical_position: Option<CartVec>,
    pub color: Color,
}
impl EntityData for GraphicsData {}

#[derive(Clone, Debug, PrintFields)]
pub struct CharacterData {
    pub faction: Entity,
    pub health: Reduceable<i32>,
    pub action_points: Reduceable<i32>,
    pub move_speed: Sext,
    // represented in sexts
    pub moves: Sext,
    pub stamina: Reduceable<Sext>,
    pub stamina_recovery: Sext,
    pub sprite: String,
    pub name: String,
}

impl EntityData for CharacterData {}

//pub trait AsCharacterData<'a> {
//    fn resolve(&self, view : &'a WorldView) -> &'a CharacterData;
//}
//impl <'a> AsCharacterData<'a> for Entity {
//    fn resolve(&self, view: &'a WorldView) -> &'a CharacterData {
//        view.data::<CharacterData>(*self)
//    }
//}
//impl <'a> AsCharacterData<'a> for &'a CharacterData {
//    fn resolve(&self, view: &'a WorldView) -> &'a CharacterData {
//        self
//    }
//}

pub struct Character<'a> {
    pub entity: Entity,
    character: &'a CharacterData,
    pub position: &'a PositionData,
    pub graphics: &'a GraphicsData,
    pub action: &'a ActionData
}

impl<'a> Deref for Character<'a> {
    type Target = CharacterData;
    fn deref(&self) -> &CharacterData { self.character }
}

impl <'a> Character<'a> {
    pub fn effective_graphical_pos(&self) -> CartVec {
        self.graphics.graphical_position.unwrap_or_else(|| self.position.hex.as_cart_vec())
    }
}

pub trait CharacterStore {
    fn character(&self, ent: Entity) -> Character;
}

impl CharacterStore for WorldView {
    fn character(&self, ent: Entity) -> Character {
        Character {
            entity: ent,
            character: self.data::<CharacterData>(ent),
            position: self.data::<PositionData>(ent),
            graphics: self.data::<GraphicsData>(ent),
            action: self.data::<ActionData>(ent)
        }
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
            faction: Entity::default(),
            health: Reduceable::new(1),
            action_points: Reduceable::new(6),
            moves: Sext::zero(),
            move_speed: Sext::of(1),
            stamina: Reduceable::new(Sext::of(6)),
            stamina_recovery: Sext::of(1),
            sprite: strf("default/defaultium"),
            name: strf("unnamed"),
        }
    }
}

impl CharacterData {
    pub fn is_alive(&self) -> bool {
        self.health.cur_value() > 0
    }
    pub fn can_act(&self) -> bool { self.action_points.cur_value() > 0 }

    pub fn max_moves_remaining(&self, multiplier: f64) -> Sext {
        self.moves + Sext::of_rounded(self.move_speed.as_f64() * self.action_points.cur_value() as f64 * multiplier)
    }
    pub fn max_moves_per_turn(&self, multiplier: f64) -> Sext {
        self.move_speed * self.action_points.max_value()
    }
}

