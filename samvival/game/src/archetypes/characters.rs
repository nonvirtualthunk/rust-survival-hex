use archetypes::ArchetypeLibrary;
use common::prelude::*;
use data::entities::ActionData;
use data::entities::combat::*;
use data::entities::EquipmentData;
use data::entities::GraphicsData;
use data::entities::IdentityData;
use data::entities::InventoryData;
use data::entities::ModifierTrackingData;
use data::entities::movement::MovementData;
use data::entities::movement;
use data::entities::ObserverData;
use data::entities::PositionData;
use data::entities::SkillData;
use prelude::*;
use std::collections::HashMap;

pub fn character_archetypes() -> ArchetypeLibrary {
    let baseline: EntityBuilder = EntityBuilder::new()
        .with(SkillData::default())
        .with(InventoryData {
            items: Vec::new(),
            inventory_size: Some(5),
        })
        .with_creator(|world| MovementData {
            move_speed: Sext::of_parts(1, 0), // one and 0 sixths
            movement_types: vec![movement::create_walk_movement_type(world)],
            ..Default::default()
        })
        .with(EquipmentData::default())
        .with(PositionData::default())
        .with(GraphicsData::default())
        .with(ActionData::default())
        .with(ModifierTrackingData::default())
        .with(ObserverData { vision_range: 10, low_light_vision_range: 6, dark_vision_range: 3 })
        .with(IdentityData::of_kind(&taxonomy::Person));


    let mut archetypes_by_name = HashMap::new();

    archetypes_by_name.insert(strf("human"), baseline.clone()
        .with_creator(|world| CombatData {
            natural_attacks: vec![
                create_attack(world, "punch", vec![&taxonomy::attacks::NaturalAttack, &taxonomy::attacks::BludgeoningAttack, &taxonomy::attacks::MeleeAttack], Attack {
                    name: strf("punch"),
                    verb: None,
                    attack_type: AttackType::Melee,
                    ap_cost: 3,
                    damage_dice: DicePool::of(1,1),
                    damage_bonus: 0,
                    to_hit_bonus: 0,
                    primary_damage_type: DamageType::Bludgeoning,
                    secondary_damage_type: None,
                    range: 1,
                    min_range: 0,
                    ammunition_kind: None,
                    stamina_cost: 0,
                    pattern: HexPattern::Single,
                })],
            ..Default::default()
        }),
    );

    archetypes_by_name.insert(strf("mud monster"), baseline.clone()
        .with(CharacterData {
            sprite: String::from("void/monster"),
            name: String::from("Monster"),
            action_points: Reduceable::new(6),
            health: Reduceable::new(16),
            ..Default::default()
        })
        .with_creator(|world| MovementData {
            move_speed: Sext::of_rounded(0.75),
            movement_types: vec![movement::create_walk_movement_type(world)],
            ..Default::default()
        })
        .with_creator(|world| CombatData {
            natural_attacks: vec![
                create_attack(
                    world, "punch",
                    vec![&taxonomy::attacks::NaturalAttack, &taxonomy::attacks::BludgeoningAttack, &taxonomy::attacks::MeleeAttack],
                    Attack {
                        name: strf("slam"),
                        damage_dice: DicePool::of(1,4),
                        ..Default::default()
                    })],
            ..Default::default()
        })
        .with(SkillData::default())
        .with(EquipmentData::default())
        .with(GraphicsData::default())
        .with(PositionData::default())
        .with(IdentityData::of_kind(taxon("mud monster", &taxonomy::Monster))),
    );

    ArchetypeLibrary {
        archetypes_by_name,
        default: baseline,
    }
}