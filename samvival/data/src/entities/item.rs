use entities::combat::Attack;
use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::AxialCoord;
use entities::common_entities::Taxon;
use game::entity;

#[derive(Clone, Default, Debug, Serialize, Deserialize, PrintFields)]
pub struct ItemData {
    pub attacks : Vec<Entity>,
    pub in_inventory_of: Option<Entity>
}

impl EntityData for ItemData {
    fn nested_entities(&self) -> Vec<Entity> {
        self.attacks.clone()
    }
}


pub trait ItemDataStore {
    fn item(&self, ent : Entity) -> &ItemData;
}
impl ItemDataStore for WorldView {
    fn item(&self, ent: Entity) -> &ItemData {
        self.data::<ItemData>(ent)
    }
}
