use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::hex::AxialCoord;
use game::core::*;
use game::entity;
use entities::selectors::EntitySelector;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Fields)]
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


#[derive(Clone, Debug, Default, Serialize, Deserialize, Fields)]
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


#[derive(Clone, Debug, Default, Serialize, Deserialize, Fields)]
pub struct StackData {
    pub stack_of : Entity,
    pub stack_size : i32,
    pub stack_limit : i32,
}
impl EntityData for StackData {}