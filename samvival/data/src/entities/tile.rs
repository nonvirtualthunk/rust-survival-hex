use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use game::core::*;
use std::ops::Deref;
use common::prelude::*;
use common::hex::*;
use entities::common_entities::Taxon;
use game::prelude::*;
use common::string::IStr;
use entities::effects::*;
use entities::EntitySelector;
use game::DicePool;
use game::EntityIndex;
use game::DataView;
use std::collections::HashMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct TileData {
    pub name_modifiers: Vec<String>,
    pub position: AxialCoord,
    pub occupied_by: Option<Entity>,
}
impl EntityData for TileData {}
impl TileData {
    pub fn is_occupied(&self) -> bool { self.occupied_by.is_some() }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct TerrainData {
    pub fertility: i8,
    pub cover: i8,
    pub elevation: i8,
    pub position: AxialCoord,
    pub move_cost: Sext,
    pub occupied_by: Option<Entity>,
    pub harvestables: HashMap<String, Entity>,
    pub kind : Taxon,
}
impl EntityData for TerrainData {
    fn nested_entities(&self) -> Vec<Entity> { self.harvestables.values().cloned().collect_vec() }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct VegetationData {
    pub cover: i8,
    pub move_cost: Sext,
    pub harvestables: HashMap<String, Entity>,
    pub kind : Taxon,
}
impl EntityData for VegetationData {
    fn nested_entities(&self) -> Vec<Entity> { self.harvestables.values().cloned().collect_vec() }
}


pub trait TileStore {
    fn tile(&self, coord: AxialCoord) -> &TileData;
    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData>;
    fn tile_ent(&self, coord: AxialCoord) -> TileEntity;
    fn tile_ent_opt(&self, coord: AxialCoord) -> Option<TileEntity>;
    fn terrain(&self, coord: AxialCoord) -> &TerrainData;
    fn vegetation(&self, coord: AxialCoord) -> &VegetationData;
}

impl TileStore for WorldView {
    fn tile(&self, coord: AxialCoord) -> &TileData {
        let tile_ent = self.entity_by_key(&coord).expect("Tile is expected to exist");
        self.data::<TileData>(tile_ent)
    }

    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData> {
        self.entity_by_key(&coord).map(|e| self.data::<TileData>(e))
    }

    fn terrain(&self, coord: AxialCoord) -> &TerrainData {
        let tile_ent = self.entity_by_key(&coord).expect("Tile is expected to exist");
        self.data::<TerrainData>(tile_ent)
    }
    fn vegetation(&self, coord: AxialCoord) -> &VegetationData {
        let tile_ent = self.entity_by_key(&coord).expect("Tile is expected to exist");
        self.data::<VegetationData>(tile_ent)
    }

    fn tile_ent(&self, coord: AxialCoord) -> TileEntity {
        self.tile_ent_opt(coord).expect("tile is expected to exist")
    }

    fn tile_ent_opt(&self, coord: AxialCoord) -> Option<TileEntity> {
        if let Some(entity) = self.entity_by_key(&coord) {
            Some(TileEntity { entity, data: self.data::<TileData>(entity) })
        } else {
            None
        }
    }
}

pub struct TileEntity<'a> {
    pub entity: Entity,
    pub data: &'a TileData,
}

impl<'a> Deref for TileEntity<'a> {
    type Target = TileData;

    fn deref(&self) -> &TileData {
        self.data
    }
}


pub trait IntoHarvestable {
    fn harvestable_data<'a>(&'a self, world: &'a WorldView) -> &'a Harvestable;
}

impl IntoHarvestable for Harvestable {
    fn harvestable_data<'a>(&'a self, world: &'a WorldView) -> &'a Harvestable { self }
}

impl IntoHarvestable for Entity {
    fn harvestable_data<'a>(&'a self, world: &'a WorldView) -> &'a Harvestable {
        world.data::<Harvestable>(*self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct RenewRate {
    pub fertility_dependent : bool,
    pub season_multipliers : HashMap<Season, f32>,
    pub rate : Sext
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DepletionBehavior {
    None, // do nothing when depleted
    Remove, // remove this harvestable when it is depleted
    RemoveLayer, // remove the entire layer this harvestable comes from (i.e. chopping down a forest)
    Custom(Vec<EffectReference>)
}
impl Default for DepletionBehavior { fn default() -> Self { DepletionBehavior::None } }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolUse {
    None, // does not use a tool
    Required, // must have a tool in order to harvest at all
    DifficultWithout { amount_limit : Option<i32>, ap_increase : Option<i32> }, // difficult to gather without a tool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct Harvestable {
    pub ap_per_harvest: i32,
    // if provided, indicates that this harvestable should have an effect on the name
    // of a tile that contains it
    pub name_modifiers: Vec<String>,
    pub amount: Reduceable<Sext>,
    pub dice_amount_per_harvest: DicePool,
    pub fixed_amount_per_harvest: i32,
    pub renew_rate: Option<RenewRate>,
    pub resource: Entity,
    pub on_depletion: DepletionBehavior,
    pub action_name: String,
    pub tool: EntitySelector,
    pub tool_use: ToolUse,
    pub skills_used: Vec<Skill>,
    pub character_requirements: EntitySelector,
}
impl Harvestable {
    pub fn requires_tool(&self) -> bool { self.tool_use == ToolUse::Required }
}

impl EntityData for Harvestable {}

impl Default for Harvestable {
    fn default() -> Self {
        Harvestable {
            amount: Reduceable::new(Sext::of(0)),
            name_modifiers: Vec::new(),
            ap_per_harvest: 1,
            dice_amount_per_harvest: DicePool::none(),
            fixed_amount_per_harvest: 0,
            renew_rate: None,
            resource: Entity::sentinel(),
            on_depletion: DepletionBehavior::None,
            action_name: strf("Sentinel"),
            tool: EntitySelector::None,
            tool_use: ToolUse::None,
            skills_used: Vec::new(),
            character_requirements: EntitySelector::Any,
        }
    }
}


pub struct TileAccessor<'a> {
    index : &'a EntityIndex<AxialCoord>,
    tiles : DataView<'a, TileData>,
    terrain : DataView<'a, TerrainData>,
    vegetation : DataView<'a, VegetationData>,
}

impl <'a> TileAccessor<'a> {
    pub fn new(from : &'a WorldView) -> TileAccessor<'a> {
        TileAccessor {
            index : from.entity_index::<AxialCoord>(),
            tiles : from.all_data_of_type::<TileData>(),
            terrain : from.all_data_of_type::<TerrainData>(),
            vegetation : from.all_data_of_type::<VegetationData>(),
        }
    }
    pub fn tile_opt(&self, pos : AxialCoord) -> Option<TileEntity> {
        self.index.get(&pos).map(|entity| TileEntity { entity : *entity, data : self.tiles.data(*entity) })
    }
    pub fn terrain(&'a self, ent : &TileEntity) -> &'a TerrainData { self.terrain.data(ent.entity) }
    pub fn terrain_at(&'a self, pos : AxialCoord) -> &'a TerrainData { self.index.get(&pos).map(|ent| self.terrain.data(*ent)).unwrap_or(self.terrain.sentinel()) }
    pub fn vegetation(&'a self, ent : &TileEntity) -> &'a VegetationData { self.vegetation.data(ent.entity) }
    pub fn vegetation_opt(&'a self, ent : &TileEntity) -> Option<&'a VegetationData> { self.vegetation.data_opt(ent.entity) }
}

//impl Harvestable {
//    pub fn simple<S : Into<String>>(action_name: S, amount : i32, renew_rate : Option<Sext>, resource : Entity,
//                                    on_depletion : Option<EffectReference>, tool : EntitySelector, requires_tool : bool,
//                                    skills : Vec<Skill>, character_requirements : EntitySelector) -> Harvestable {
//        Harvestable {
//            action_name : action_name.into(),
//            amount : Reduceable::new(Sext::of(amount)),
//            renew_rate,
//            resource,
//            on_depletion_effects : on_depletion.into_iter().collect_vec(),
//            tool,
//            requires_tool,
//            skills_used : skills,
//            character_requirements
//        }
//    }
//}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ConstantResources {
    pub straw: Entity,
    pub wood: Entity,
    pub iron: Entity,
    pub quarried_stone: Entity,
    pub loose_stone: Entity,
    pub fruit: Entity,
    pub dirt: Entity,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct Resources {
    pub main: ConstantResources,
    pub custom_resources: HashMap<String, Entity>,
}

impl EntityData for Resources {}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct FoodInfo {
    pub satiation: i32
}

impl EntityData for FoodInfo {}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct Material {
    pub edge: i32, // how well this material can hold an edge
    pub point: i32, // how well it can hold a point
    pub hardness: i32, // how hard it is to dint or break
    pub flammable: bool, // whether it can burn
    pub density: i32, // how dense it is
    pub strength: i32, // how strong it is, how much it can support
    pub ductile: bool, // whether it can be made into wire
    pub cordable: bool, // whether it can be made into rope
    pub magnetic: bool, // whether it is magnetic
    // Quality measures are relative to what their other stats would normally indicate, so
    // they are an after-the-fact modifier, not the core determinator of how good something made
    // from this will be
    pub item_quality: i32, // centered at 0, -8 is very bad quality, +8 is very good quality
    pub building_quality: i32, // as above
    pub material_effects : Vec<MaterialEffect>, // specific effects that apply to things made from this material

}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum MaterialEffectSelector {
    IngredientType(Taxon),
    ItemType(Taxon),
}
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum MaterialEffectType {
    DamageBonus(EntitySelector, i32),
    ToHitBonus(EntitySelector, i32),
    WeaponAttribute(AttributeType, i32)
}
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct MaterialEffect(pub MaterialEffectSelector, pub MaterialEffectType);


impl EntityData for Material {}


#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct IngredientData {
    pub effects_by_kinds : HashMap<Taxon, EffectReference>
}


use entities::common_entities::taxonomy;
use entities::common_entities::taxon_vec;
use game::World;
use game::EntityBuilder;
use entities::common_entities::IdentityData;


struct InitResourcesModifier {
    resources: Resources
}

use game::Modifier;
use game::ModifierType;
use entities::effects::Effect;
use entities::skill::Skill;
use entities::inventory::StackData;
use entities::item::ItemData;
use entities::combat::create_attack;
use entities::combat::Attack;
use entities::combat::DamageType;
use entities::combat::AttackType;
use entities::item::ToolData;
use entities::time::Season;
use entities::item::WorthData;
use entities::item::Worth;
use entities::attributes::AttributeType;
use entities::attributes::attributes;
//impl Modifier<Resources> for InitResourcesModifier {
//    fn modify(&self, data: &mut Resources, world: &WorldView) {
//        *data = self.resources.clone()
//    }
//
//    fn is_active(&self, world: &WorldView) -> bool { true }
//    fn modifier_type(&self) -> ModifierType { ModifierType::Permanent }
//}

impl Resources {
    pub fn init_resources(world: &mut World) {
        world.ensure_world_data::<Resources>();
        let mut main = world.world_data::<Resources>().main.clone();

        if main.straw.is_sentinel() {
            main.straw = EntityBuilder::new()
                .with(Material {
                    hardness: 1,
                    density: 2,
                    strength: 1,
                    flammable: true,
                    cordable: true,
                    ..Default::default()
                })
                .with(WorthData::new(Worth::low(1)))
                .with(ItemData { stack_limit: 6, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::Straw))
                .create(world);
        }

        if main.wood.is_sentinel() {
            let training_blade_material_effect = MaterialEffect(
                MaterialEffectSelector::IngredientType((&taxonomy::ingredient_types::WeaponHeadIngredient).into()),
                MaterialEffectType::WeaponAttribute(&attributes::TrainingWeapon, 1)
            );

            main.wood = EntityBuilder::new()
                .with(Material {
                    edge: 1,
                    point: 5,
                    hardness: 4,
                    density: 3,
                    strength: 4,
                    flammable: true,
                    material_effects: vec![training_blade_material_effect],
                    ..Default::default()
                })
                .with_creator(|world| ItemData {
                    stack_limit: 3,
                    attacks: vec![
                        create_attack(world, "smack",
                                      vec![&taxonomy::attacks::BludgeoningAttack, &taxonomy::attacks::ImprovisedAttack, &taxonomy::attacks::MeleeAttack],
                                      Attack {
                                          name: strf("smack"),
                                          to_hit_bonus: -1,
                                          primary_damage_type: DamageType::Bludgeoning,
                                          damage_dice: DicePool::of(1, 2),
                                          attack_type: AttackType::Melee,
                                          ap_cost: 4,
                                          ..Default::default()
                                      })],
                    ..Default::default()
                })
                .with(WorthData::new(Worth::medium(-1)))
                .with(ToolData { tool_speed_bonus: 1, ..Default::default() })
                .with(IdentityData::of_kinds(vec![
                    &taxonomy::resources::Wood,
                    &taxonomy::tools::Rod,
                    &taxonomy::weapons::MeleeWeapon,
                    &taxonomy::weapons::ImprovisedWeapon]))
                .create(world);
        }
        if main.iron.is_sentinel() {
            main.iron = EntityBuilder::new()
                .with(Material {
                    edge: 6,
                    point: 6,
                    hardness: 10,
                    strength: 10,
                    density: 6,
                    ductile: true,
                    magnetic: true,
                    ..Default::default()
                })
                .with(WorthData::new(Worth::medium(3)))
                .with(ItemData { stack_limit: 3, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::Iron))
                .create(world);
        }

        if main.quarried_stone.is_sentinel() {
            main.quarried_stone = EntityBuilder::new()
                .with(Material {
                    edge : 2,
                    point : 4,
                    hardness: 6,
                    density: 5,
                    strength: 4,
                    item_quality : 1,
                    building_quality : 1,
                    ..Default::default()
                })
                .with(WorthData::new(Worth::medium(0)))
                .with(ItemData { stack_limit: 1, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::QuarriedStone))
                .create(world);
        }

        if main.loose_stone.is_sentinel() {
            main.loose_stone = EntityBuilder::new()
                .with(Material {
                    edge : 1,
                    point : 3,
                    hardness: 6,
                    strength: 3,
                    density: 5,
                    item_quality: 0, // loose rocks are ok for making itmes out of
                    building_quality: -4, // but they're not very good for making buildings
                    ..Default::default()
                })
                .with(WorthData::new(Worth::medium(-2)))
                .with(ItemData { stack_limit: 3, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::LooseStone))
                .create(world)
        }

        if main.dirt.is_sentinel() {
            main.dirt = EntityBuilder::new()
                .with(Material {
                    density: 3,
                    hardness: 3,
                    ..Default::default()
                })
                .with(WorthData::new(Worth::low(-5)))
                .with(ItemData { stack_limit: 2, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::Dirt))
                .create(world);
        }

        if main.fruit.is_sentinel() {
            let fruit = EntityBuilder::new()
                .with(Material {
                    hardness: 1,
                    density: 1,
                    flammable: true,
                    cordable: true,
                    ..Default::default()
                })
                .with(WorthData::new(Worth::medium(0)))
                .with(ItemData { stack_limit: 6, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::Fruit))
                .create(world);
        }

        world.modify_world(Resources::main.set_to(main), "resource initialization");
    }
}