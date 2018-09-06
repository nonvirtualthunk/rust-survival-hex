use entities::combat::Attack;
use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::AxialCoord;
use entities::common_entities::Taxon;
use game::entity;

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct ItemData {
    pub attacks : Vec<Entity>,
    pub in_inventory_of: Option<Entity>,
    pub tool_speed_bonus : i32,
    pub tool_harvest_dice_bonus : DicePool,
    pub tool_harvest_fixed_bonus : i32,
    pub stack_limit : i32,
}

impl EntityData for ItemData {
    fn nested_entities(&self) -> Vec<Entity> {
        self.attacks.clone()
    }
}

impl Default for ItemData {
    fn default() -> Self {
        ItemData {
            attacks : Vec::new(),
            in_inventory_of : None,
            tool_speed_bonus : 0,
            tool_harvest_fixed_bonus : 0,
            tool_harvest_dice_bonus : DicePool::none(),
            stack_limit : 1
        }
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
