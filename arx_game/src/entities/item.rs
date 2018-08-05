use entities::combat::Attack;
use world::Entity;
use world::EntityData;
use world::WorldView;
use common::AxialCoord;

#[derive(Clone, Default, Debug)]
pub struct ItemData {
    pub primary_attack : Option<Attack>,
    pub secondary_attack : Option<Attack>,
    pub held_by : Option<Entity>,
    pub position : Option<AxialCoord>
}

impl EntityData for ItemData {}
pub trait ItemDataStore {
    fn item(&self, ent : Entity) -> &ItemData;
}
impl ItemDataStore for WorldView {
    fn item(&self, ent: Entity) -> &ItemData {
        self.data::<ItemData>(ent)
    }
}
