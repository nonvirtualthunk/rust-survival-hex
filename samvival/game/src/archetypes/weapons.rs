use game::prelude::*;
use common::prelude::*;
use entities::item::*;
use entities::combat::*;
use std::collections::HashMap;
use entities::common::IdentityData;
use entities::common::Taxon;
use entities::common::taxon;
use entities::common::taxonomy;

use archetypes::ArchetypeLibrary;

pub fn weapon_archetypes() -> ArchetypeLibrary {
    let mut archetypes_by_name = HashMap::new();

    archetypes_by_name.insert(strf("longbow"), EntityBuilder::new()
        .with(ItemData {
            attacks: vec![Attack {
                name: "bowshot",
                attack_type: AttackType::Projectile,
                ap_cost: 4,
                damage_dice: DicePool {
                    die: 8,
                    count: 1,
                },
                damage_bonus: 1,
                to_hit_bonus: 1,
                primary_damage_type: DamageType::Piercing,
                secondary_damage_type: None,
                range: 10,
                min_range: 2,
                ammunition_kind: Some(taxonomy::projectiles::Arrow)
            }],
            ..Default::default()
        })
        .with(IdentityData::of_kind(taxon("longbow", &taxonomy::Bow)))
    );

    archetypes_by_name.insert(strf("longsword"), EntityBuilder::new()
        .with(ItemData {
            attacks: vec![
                Attack {
                    name: "stab",
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
                },
                Attack {
                    name: "slash",
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
                }],
            ..Default::default()
        })
        .with(IdentityData::of_kind(taxon("longsword", &taxonomy::Sword)))
    );

    archetypes_by_name.insert(strf("shortsword"), EntityBuilder::new()
        .with(ItemData {
            attacks: vec![
                Attack {
                    name: "stab",
                    attack_type: AttackType::Melee,
                    ap_cost: 3,
                    damage_dice: DicePool::of(1, 8),
                    damage_bonus: 0,
                    to_hit_bonus: 1,
                    primary_damage_type: DamageType::Piercing,
                    secondary_damage_type: None,
                    range: 1,
                    min_range: 0,
                    ammunition_kind: None,
                },
                Attack {
                    name: "slash",
                    attack_type: AttackType::Melee,
                    ap_cost: 3,
                    damage_dice: DicePool::of(2, 4),
                    damage_bonus: 0,
                    to_hit_bonus: 0,
                    primary_damage_type: DamageType::Slashing,
                    secondary_damage_type: None,
                    range: 1,
                    min_range: 0,
                    ammunition_kind: None,
                }],
            ..Default::default()
        })
        .with(IdentityData::of_kind(taxon("shortsword", &taxonomy::Sword)))
    );

    archetypes_by_name.insert(strf("longspear"), EntityBuilder::new()
        .with(ItemData {
            attacks: vec![
                Attack {
                    name: "stab",
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
                },
                Attack {
                    name: "smack",
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
                },
                Attack {
                    name: "throw",
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
                }],
            ..Default::default()
        })
        .with(IdentityData::of_kind(taxon("longspear", &taxonomy::Spear)))
    );


    let default = EntityBuilder::new()
        .with(ItemData {
            attacks: vec![
                Attack {
                    name: "default",
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
                }],
            ..Default::default()
        })
        .with(IdentityData::of_kind(taxon("default", &taxonomy::Weapon)));

    ArchetypeLibrary {
        archetypes_by_name,
        default,
    }
}