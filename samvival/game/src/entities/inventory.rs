use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::hex::AxialCoord;
use game::core::*;

#[derive(Clone, Debug, Default, PrintFields)]
pub struct EquipmentData {
    pub equipped : Vec<Entity>,
}
impl EntityData for EquipmentData {}

pub trait EquipmentDataStore {
    fn equipment(&self, ent : Entity) -> &EquipmentData;
}
impl EquipmentDataStore for WorldView {
    fn equipment(&self, ent: Entity) -> &EquipmentData {
        self.data::<EquipmentData>(ent)
    }
}


#[derive(Clone, Debug, Default, PrintFields)]
pub struct InventoryData {
    pub items : Vec<Entity>,
    pub inventory_size : Option<u32>,

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
