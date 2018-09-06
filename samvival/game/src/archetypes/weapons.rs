use game::prelude::*;
use common::prelude::*;
use data::entities::item::*;
use data::entities::combat::*;
use std::collections::HashMap;
use data::entities::IdentityData;
use data::entities::Taxon;
use data::entities::taxon;
use data::entities::taxonomy;
use data::entities::taxonomy::attacks::*;

use archetypes::ArchetypeLibrary;

pub fn weapon_archetypes() -> ArchetypeLibrary {
    let mut archetypes_by_name = HashMap::new();


    archetypes_by_name.insert(strf("longbow"), EntityBuilder::new()
        .with_creator(|world| ItemData {
            attacks: vec![create_attack(world, "bowshot", vec![&ProjectileAttack, &PiercingAttack], Attack {
                name: strf("bowshot"),
                verb: Some(strf("shoot")),
                attack_type: AttackType::Projectile,
                ap_cost: 4,
                damage_dice: DicePool::of(1,8),
                damage_bonus: 1,
                to_hit_bonus: 1,
                primary_damage_type: DamageType::Piercing,
                secondary_damage_type: None,
                range: 10,
                min_range: 2,
                ammunition_kind: Some((&taxonomy::projectiles::Arrow).into()),
                stamina_cost: 0,
                pattern: HexPattern::Single,
            })],
            ..Default::default()
        })
        .with(IdentityData::of_kind(taxon("longbow", &taxonomy::weapons::Bow))),
    );

    archetypes_by_name.insert(strf("longsword"), EntityBuilder::new()
        .with_creator(|world| ItemData {
            attacks: vec![
                create_attack(world, "stab", vec![&StabbingAttack, &MeleeAttack], Attack {
                    name: strf("stab"),
                    verb: None,
                    attack_type: AttackType::Melee,
                    ap_cost: 3,
                    damage_dice: DicePool::of(1, 10),
                    damage_bonus: 0,
                    to_hit_bonus: 1,
                    primary_damage_type: DamageType::Piercing,
                    secondary_damage_type: None,
                    range: 1,
                    min_range: 0,
                    ammunition_kind: None,
                    stamina_cost: 0,
                    pattern: HexPattern::Single,
                }),
                create_attack(world, "slash", vec![&SlashingAttack, &MeleeAttack], Attack {
                    name: strf("slash"),
                    verb: None,
                    attack_type: AttackType::Melee,
                    ap_cost: 4,
                    damage_dice: DicePool::of(2, 6),
                    damage_bonus: 1,
                    to_hit_bonus: 0,
                    primary_damage_type: DamageType::Slashing,
                    secondary_damage_type: None,
                    range: 1,
                    min_range: 0,
                    ammunition_kind: None,
                    stamina_cost: 0,
                    pattern: HexPattern::Single,
                })],
            ..Default::default()
        })
        .with(IdentityData::of_kind(taxon("longsword", &taxonomy::weapons::Sword))),
    );

//    archetypes_by_name.insert(strf("shortsword"), EntityBuilder::new()
//        .with(ItemData {
//            attacks: vec![
//                Attack {
//                    name: "stab",
//                    attack_type: AttackType::Melee,
//                    ap_cost: 3,
//                    damage_dice: DicePool::of(1, 8),
//                    damage_bonus: 0,
//                    to_hit_bonus: 1,
//                    primary_damage_type: DamageType::Piercing,
//                    secondary_damage_type: None,
//                    range: 1,
//                    min_range: 0,
//                    ammunition_kind: None,
//                },
//                Attack {
//                    name: "slash",
//                    attack_type: AttackType::Melee,
//                    ap_cost: 3,
//                    damage_dice: DicePool::of(2, 4),
//                    damage_bonus: 0,
//                    to_hit_bonus: 0,
//                    primary_damage_type: DamageType::Slashing,
//                    secondary_damage_type: None,
//                    range: 1,
//                    min_range: 0,
//                    ammunition_kind: None,
//                }],
//            ..Default::default()
//        })
//        .with(IdentityData::of_kind(taxon("shortsword", &taxonomy::Sword))),
//    );

    archetypes_by_name.insert(strf("longspear"), EntityBuilder::new()
        .with_creator(|world| ItemData {
            attacks: vec![
                create_attack(world, "stab", vec![&StabbingAttack, &ReachAttack], Attack {
                    name: strf("stab"),
                    verb: None,
                    attack_type: AttackType::Reach,
                    ap_cost: 5,
                    damage_dice: DicePool::of(1, 10),
                    damage_bonus: 2,
                    to_hit_bonus: 0,
                    primary_damage_type: DamageType::Piercing,
                    secondary_damage_type: None,
                    range: 2,
                    min_range: 2,
                    ammunition_kind: None,
                    stamina_cost: 0,
                    pattern: HexPattern::Single,
                }),
                create_attack(world, "smack", vec![&BludgeoningAttack, &MeleeAttack], Attack {
                    name: strf("smack"),
                    verb: None,
                    attack_type: AttackType::Melee,
                    ap_cost: 3,
                    damage_dice: DicePool::of(1, 4),
                    damage_bonus: 0,
                    to_hit_bonus: 1,
                    primary_damage_type: DamageType::Bludgeoning,
                    secondary_damage_type: None,
                    range: 1,
                    min_range: 0,
                    ammunition_kind: None,
                    stamina_cost: 0,
                    pattern: HexPattern::Single,
                }),
                create_attack(world, "throw", vec![&ThrownAttack, &PiercingAttack], Attack {
                    name: strf("throw"),
                    verb: Some(strf("throw your spear at")),
                    attack_type: AttackType::Thrown,
                    ap_cost: 4,
                    damage_dice: DicePool::of(1, 12),
                    damage_bonus: 2,
                    to_hit_bonus: -1,
                    primary_damage_type: DamageType::Piercing,
                    secondary_damage_type: None,
                    range: 4,
                    min_range: 2,
                    ammunition_kind: None,
                    stamina_cost: 0,
                    pattern: HexPattern::Single,
                })],
            ..Default::default()
        })
        .with(IdentityData::of_kind(taxon("longspear", &taxonomy::weapons::Spear))),
    );


    let default = EntityBuilder::new()
        .with_creator(|world| ItemData {
            attacks: vec![
                create_attack(world, "default", vec![&taxonomy::Attack], Attack {
                    name: strf("default"),
                    verb: None,
                    attack_type: AttackType::Melee,
                    ap_cost: 3,
                    damage_dice: DicePool::of(1, 6),
                    damage_bonus: 0,
                    to_hit_bonus: 0,
                    primary_damage_type: DamageType::Piercing,
                    secondary_damage_type: None,
                    range: 1,
                    min_range: 0,
                    ammunition_kind: None,
                    stamina_cost: 0,
                    pattern: HexPattern::Single,
                })],
            ..Default::default()
        })
        .with(IdentityData::of_kind(taxon("default", &taxonomy::Weapon)));

    ArchetypeLibrary {
        archetypes_by_name,
        default,
    }
}