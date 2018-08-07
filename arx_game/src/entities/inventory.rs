use world::Entity;
use world::EntityData;
use world::WorldView;
use common::hex::AxialCoord;
use core::*;

#[derive(Clone, Debug, Default, PrintFields)]
pub struct InventoryData {
    pub equipped : Vec<Entity>,
    pub inventory : Vec<Entity>,
}
impl EntityData for InventoryData {}

pub trait InventoryDataStore {
    fn inventory(&self, ent : Entity) -> &InventoryData;
}
impl InventoryDataStore for WorldView {
    fn inventory(&self, ent: Entity) -> &InventoryData {
        self.data::<InventoryData>(ent)
    }
}
