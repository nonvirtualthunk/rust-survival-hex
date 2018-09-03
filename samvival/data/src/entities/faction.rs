use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::hex::AxialCoord;
use game::core::*;
use common::Color;
use common::reflect::*;
use game::entity;

#[derive(Clone, Default, Debug, Serialize, Deserialize, PrintFields)]
pub struct FactionData {
    pub name : String,
    pub color : Color,
    pub player_faction : bool,
}

impl EntityData for FactionData {}

pub trait FactionStore {
    fn faction(&self, entity : Entity) -> &FactionData;
}
impl FactionStore for WorldView {
    fn faction(&self, entity: Entity) -> &FactionData {
        self.data::<FactionData>(entity)
    }
}


