use world::Entity;
use world::EntityData;
use world::WorldView;
use common::hex::AxialCoord;
use core::*;
use common::Color;

#[derive(Clone, Default, Debug)]
pub struct FactionData {
    pub name : String,
    pub color : Color
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


