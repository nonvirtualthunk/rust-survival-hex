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

#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct TileData {
    pub main_terrain_name: String,
    pub secondary_terrain_name: Option<String>,
    pub position: AxialCoord,
    pub move_cost: Sext,
    pub cover: i8,
    pub occupied_by: Option<Entity>,
    pub elevation: i8,
}

impl EntityData for TileData {}


pub trait TileStore {
    fn tile(&self, coord: AxialCoord) -> &TileData;
    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData>;
    fn tile_ent(&self, coord: AxialCoord) -> TileEntity;
    fn tile_ent_opt(&self, coord: AxialCoord) -> Option<TileEntity>;
}

impl TileStore for WorldView {
    fn tile(&self, coord: AxialCoord) -> &TileData {
        let tile_ent = self.entity_by_key(&coord).expect("Tile is expected to exist");
        self.data::<TileData>(tile_ent)
    }

    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData> {
        self.entity_by_key(&coord).map(|e| self.data::<TileData>(e))
    }

    fn tile_ent(&self, coord: AxialCoord) -> TileEntity {
        self.tile_ent_opt(coord).expect("tile is expected to exist")
    }

    fn tile_ent_opt(&self, coord: AxialCoord) -> Option<TileEntity> {
        if let Some(entity) = self.entity_by_key(&coord) {
            Some(TileEntity { entity, data: self.data::<TileData>(entity), harvest_data: self.data_opt::<HarvestableData>(entity) })
        } else {
            None
        }
    }
}

pub struct TileEntity<'a> { pub entity: Entity, pub data: &'a TileData, pub harvest_data: Option<&'a HarvestableData> }

impl<'a> Deref for TileEntity<'a> {
    type Target = TileData;

    fn deref(&self) -> &TileData {
        self.data
    }
}


#[derive(Default, Clone, Debug, Serialize, Deserialize, Fields)]
pub struct HarvestableData {
    pub harvestables: HashMap<String, Entity>
}

impl EntityData for HarvestableData {}

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
pub struct Harvestable {
    pub ap_per_harvest: i32,
    pub amount: Reduceable<Sext>,
    pub dice_amount_per_harvest: DicePool,
    pub fixed_amount_per_harvest: i32,
    pub renew_rate: Option<Sext>,
    pub resource: Entity,
    pub on_depletion_effects: Vec<EffectReference>,
    pub action_name: String,
    pub tool: EntitySelector,
    pub requires_tool: bool,
    pub skills_used: Vec<Skill>,
    pub character_requirements: EntitySelector,
}

impl EntityData for Harvestable {}

impl Default for Harvestable {
    fn default() -> Self {
        Harvestable {
            amount: Reduceable::new(Sext::of(0)),
            ap_per_harvest: 1,
            dice_amount_per_harvest: DicePool::of(1, 1),
            fixed_amount_per_harvest: 0,
            renew_rate: None,
            resource: Entity::sentinel(),
            on_depletion_effects: Vec::new(),
            action_name: strf("Sentinel"),
            tool: EntitySelector::Any,
            requires_tool: false,
            skills_used: Vec::new(),
            character_requirements: EntitySelector::Any,
        }
    }
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
    pub stone: Entity,
    pub fruit: Entity,
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
    pub edge: i32,
    // how well it can hold an edge
    pub hardness: i32,
    pub flammable: bool,
    pub density: i32,
    pub ductile: bool,
    pub cordable: bool,
    pub magnetic: bool,
    pub stack_limit: i32,

}

impl EntityData for Material {}


use entities::common_entities::taxonomy;
use entities::common_entities::taxon;
use entities::common_entities::taxon_vec;
use std::collections::HashMap;
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
                    density: 1,
                    flammable: true,
                    cordable: true,
                    ..Default::default()
                })
                .with(ItemData { stack_limit: 6, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::Straw))
                .create(world);
        }

        if main.wood.is_sentinel() {
            main.wood = EntityBuilder::new()
                .with(Material {
                    hardness: 4,
                    density: 3,
                    flammable: true,
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
                    tool_speed_bonus: 1,
                    ..Default::default()
                })
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
                    hardness: 10,
                    density: 6,
                    ductile: true,
                    magnetic: true,
                    stack_limit: 3,
                    ..Default::default()
                })
                .with(ItemData { stack_limit: 3, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::Iron))
                .create(world);
        }

        if main.stone.is_sentinel() {
            main.stone = EntityBuilder::new()
                .with(Material {
                    hardness: 6,
                    density: 5,
                    ..Default::default()
                })
                .with(ItemData { stack_limit: 1, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::Stone))
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
                .with(ItemData { stack_limit: 6, ..Default::default() })
                .with(IdentityData::of_kind(&taxonomy::resources::Fruit))
                .create(world);
        }

        world.modify_world(Resources::main.set_to(main), "resource initialization");
    }
}