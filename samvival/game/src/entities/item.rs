use entities::combat::Attack;
use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::AxialCoord;
use entities::common::Taxon;


#[derive(Clone, Default, Debug, PrintFields)]
pub struct ItemData {
    pub attacks : Vec<Attack>,
    pub in_inventory_of: Option<Entity>
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
