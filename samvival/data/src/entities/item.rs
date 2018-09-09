use entities::combat::Attack;
use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::AxialCoord;
use entities::common_entities::Taxon;
use game::entity;
use game::DicePool;
use entities::selectors::EntitySelector;

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct ItemData {
    pub attacks : Vec<Entity>,
    pub in_inventory_of: Option<Entity>,
    pub stack_limit : i32,
    pub stack_with : EntitySelector,
}

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct ToolData {
    pub tool_speed_bonus : i32,
    pub tool_harvest_dice_bonus : DicePool,
    pub tool_harvest_fixed_bonus : i32,
}
impl EntityData for ToolData {}

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
            stack_limit : 1,
            stack_with : EntitySelector::None
        }
    }
}

impl Default for ToolData {
    fn default() -> Self {
        ToolData {
            tool_speed_bonus : 0,
            tool_harvest_fixed_bonus : 0,
            tool_harvest_dice_bonus : DicePool::none(),
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
