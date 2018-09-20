use entities::combat::Attack;
use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::AxialCoord;
use entities::common_entities::Taxon;
use game::entity;
use game::DicePool;
use entities::selectors::EntitySelector;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StackWith {
    SameArchetype,
    Custom(EntitySelector)
}

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct ItemData {
    pub attacks : Vec<Entity>,
    pub in_inventory_of: Option<Entity>,
    pub stack_limit : i32,
    pub stack_with : StackWith,
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

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct WorthData {
    pub base_worth : Worth,
}
impl EntityData for WorthData {}
impl Default for WorthData {
    fn default() -> Self {
        WorthData { base_worth : Worth::worthless(0) }
    }
}
impl WorthData {
    pub fn new(worth : Worth) -> WorthData { WorthData { base_worth : worth } }
}

#[derive(PartialEq,PartialOrd,Ord,Eq,Clone,Copy,Debug,Serialize,Deserialize)]
pub struct Worth(i32);
impl Worth {
    const WORTHLESS : i32 = 10;
    const LOW : i32 = 20;
    const MEDIUM : i32 = 30;
    const HIGH : i32 = 40;
    const VERY_HIGH : i32 = 50;
    const MAX : i32 = 100;


    pub fn worthless(dv: i32) -> Worth { Worth(Worth::WORTHLESS + dv) }
    pub fn low(dv : i32) -> Worth { Worth(Worth::LOW + dv) }
    pub fn medium(dv : i32) -> Worth { Worth(Worth::MEDIUM + dv) }
    pub fn high(dv : i32) -> Worth { Worth(Worth::HIGH + dv) }
    pub fn very_high(dv : i32) -> Worth { Worth(Worth::VERY_HIGH + dv) }

    pub fn as_i32(&self, map_to_min : i32, map_to_max : i32) -> i32 {
        let pcnt = self.0 as f64 / Worth::MAX as f64;
        (map_to_min + ((map_to_max - map_to_min) as f64 * pcnt) as i32).min(map_to_max).max(map_to_min)
    }
}


impl Default for ItemData {
    fn default() -> Self {
        ItemData {
            attacks : Vec::new(),
            in_inventory_of : None,
            stack_limit : 1,
            stack_with : StackWith::SameArchetype
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