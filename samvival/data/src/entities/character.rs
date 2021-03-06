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
use entities::common_entities::PositionData;
use entities::actions::ActionData;
use entities::time::TimeOfDay;
use entities::movement::MovementData;
use game::entity;

#[derive(Default, Clone, Debug, Serialize, Deserialize, Fields)]
pub struct GraphicsData {
    pub graphical_position: Option<CartVec>,
    pub color: Color,
}
impl EntityData for GraphicsData {}

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct CharacterData {
    pub health: Reduceable<i32>,
    pub action_points: Reduceable<i32>,
    pub stamina: Reduceable<Sext>,
    pub stamina_recovery: Sext,
    pub sprite: String,
    pub name: String,
}

impl EntityData for CharacterData {}

pub struct Character<'a> {
    pub entity: Entity,
    character: &'a CharacterData,
    pub position: &'a PositionData,
    pub graphics: &'a GraphicsData,
    pub action: &'a ActionData,
    pub allegiance: &'a AllegianceData,
    pub movement: &'a MovementData,
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
            action: self.data::<ActionData>(ent),
            allegiance: self.data::<AllegianceData>(ent),
            movement: self.data::<MovementData>(ent)
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
            health: Reduceable::new(1),
            action_points: Reduceable::new(6),
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
}


#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct ObserverData {
    pub vision_range : i32,
    pub low_light_vision_range : i32,
    pub dark_vision_range : i32,
}

impl ObserverData {
    pub fn vision_range_at_time(&self, time_of_day : TimeOfDay) -> i32 {
        match time_of_day {
            TimeOfDay::Dusk | TimeOfDay::Dawn => self.low_light_vision_range,
            TimeOfDay::Daylight => self.vision_range,
            TimeOfDay::Night => self.dark_vision_range,
        }
    }
}
impl EntityData for ObserverData {}


#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct AllegianceData {
    pub faction: Entity
}
impl EntityData for AllegianceData {}

impl Default for AllegianceData {
    fn default() -> Self {
        AllegianceData {
            faction : Entity::default()
        }
    }
}