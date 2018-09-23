use game::prelude::*;
use common::prelude::*;
use data::entities::item::*;
use data::entities::combat::*;
use std::collections::HashMap;
use data::entities::{IdentityData, ItemArchetype};
use data::entities::Taxon;

use data::entities::taxonomy;
use data::entities::taxonomy::attacks::*;

use archetypes::ArchetypeLibrary;
use data::entities::taxonomy::ingredient_types::*;
use data::entities::taxonomy::materials::*;
use data::archetype::EntityArchetype;
use data::entities::recipes::*;
use data::entities::EntitySelector;


pub fn create_weapon_archetypes(world: &mut World) {
    let longbow = EntityBuilder::new()
        .with(ItemArchetype {
            attacks: vec![(IdentityData::of_name_and_kinds("bowshot", vec![&ProjectileAttack, &PiercingAttack]), Attack {
                name: strf("bowshot"),
                verb: Some(strf("shoot")),
                attack_type: AttackType::Projectile,
                ap_cost: 4,
                damage_dice: DicePool::of(1, 8),
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
            worth: Worth::medium(0),
            ..Default::default()
        })
        .with(IdentityData::of_kind(&taxonomy::weapons::Longbow))
        .create(world);

    let longsword_item_archetype = ItemArchetype {
        attacks: vec![
            (IdentityData::of_name_and_kinds("stab", vec![&StabbingAttack, &MeleeAttack]), Attack {
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
            (IdentityData::of_name_and_kinds("slash", vec![&SlashingAttack, &MeleeAttack]), Attack {
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
        worth: Worth::medium(0),
        ..Default::default()
    };
    let longsword = EntityBuilder::new()
        .with(longsword_item_archetype.clone())
        .with(IdentityData::of_kind(&taxonomy::weapons::Longsword))
        .create(world);

    let stone_longsword = EntityBuilder::new()
        .with({
            let mut base = longsword_item_archetype.clone();
            // stone swords are heavy, blunt, slow, and harder to use
            for (ident, attack) in &mut base.attacks {
                attack.secondary_damage_type = Some(DamageType::Bludgeoning);
                attack.damage_bonus -= 2;
                attack.to_hit_bonus -= 2;
                attack.stamina_cost += 1;
                attack.ap_cost += 1;
            }
            base
        })
        .with(IdentityData::of_kind(Taxon::new(world, "stone longsword", &taxonomy::weapons::Longsword)))
        .create(world);

    let training_longsword = EntityBuilder::new()
        .with({
            let mut base = longsword_item_archetype.clone();
            // training swords are, obviously, pretty useless
            for (ident, attack) in &mut base.attacks {
                attack.secondary_damage_type = Some(DamageType::Bludgeoning);
                attack.damage_dice = DicePool::of(1, 2);
                attack.damage_bonus = 0;
                attack.to_hit_bonus = -1;
            }
            base
        })
        .with(IdentityData::of_kind(Taxon::new(world, "training longsword", &taxonomy::weapons::Longsword)))
        .create(world);


    let longspear = EntityBuilder::new()
        .with(ItemArchetype {
            attacks: vec![
                (IdentityData::of_name_and_kinds("stab", vec![&StabbingAttack, &ReachAttack]), Attack {
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
                (IdentityData::of_name_and_kinds("smack", vec![&BludgeoningAttack, &MeleeAttack]), Attack {
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
                (IdentityData::of_name_and_kinds("throw", vec![&ThrownAttack, &PiercingAttack]), Attack {
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
            worth: Worth::medium(0),
            ..Default::default()
        })
        .with(IdentityData::of_kind(Taxon::new(world, "longspear", &taxonomy::weapons::Spear)))
        .create(world);

    let hatchet = EntityBuilder::new()
        .with(ItemArchetype {
            attacks: vec![
                (IdentityData::of_name_and_kinds("chop", vec![&SlashingAttack]), Attack {
                    name: strf("chop"),
                    verb: None,
                    attack_type: AttackType::Melee,
                    ap_cost: 4,
                    damage_dice: DicePool::of(1, 4),
                    damage_bonus: 2,
                    to_hit_bonus: 0,
                    primary_damage_type: DamageType::Slashing,
                    ..Default::default()
                })],
            tool_data: Some(ToolData {
                tool_harvest_fixed_bonus: 1,
                tool_speed_bonus: 1,
                tool_harvest_dice_bonus: DicePool::none(),
            }),
            worth: Worth::low(-1),
            ..Default::default()
        })
        .with(IdentityData::of_kind(&taxonomy::tools::Hatchet))
        .create(world);


    let pickaxe = EntityBuilder::new()
        .with(ItemArchetype {
            attacks: vec![
                (IdentityData::of_name_and_kinds("stab", vec![&StabbingAttack, &ImprovisedAttack]), Attack {
                    name: strf("stab"),
                    verb: None,
                    attack_type: AttackType::Melee,
                    ap_cost: 8,
                    damage_dice: DicePool::of(1, 10),
                    damage_bonus: 1,
                    to_hit_bonus: 0,
                    primary_damage_type: DamageType::Piercing,
                    ..Default::default()
                })],
            tool_data: Some(ToolData {
                tool_harvest_fixed_bonus: 0,
                tool_speed_bonus: 1,
                tool_harvest_dice_bonus: DicePool::of(1, 2),
            }),
            worth: Worth::low(0),
            ..Default::default()
        })
        .with(IdentityData::of_kind(Taxon::new(world, "pickaxe", &taxonomy::tools::Pickaxe)))
        .create(world);


    let hatchet_recipe = EntityBuilder::new()
        .with(Recipe::new(EntityArchetype::Archetype(hatchet))
            .with_ingredient(&Haft, EntitySelector::is_one_of(vec![&Wood, &Metal]), 2)
            .with_ingredient(&Axehead, EntitySelector::is_either(&Metal, &Stone), 2)
        ).create(world);

    let longsword_recipe = EntityBuilder::new()
        .with(Recipe::new(EntityArchetype::Archetype(longsword))
            .name_from(&Blade)
            .with_ingredient(&Haft, EntitySelector::is_either(&Wood, &Metal), 2)
            .with_ingredient(&Blade, EntitySelector::is_one_of(vec![&Wood, &Metal, &Stone]), 3)
        ).create(world);

    let stone_longsword_recipe = EntityBuilder::new()
        .with(Recipe::new_child(EntityArchetype::Archetype(stone_longsword), longsword_recipe)
            .with_ingredient(&Blade, EntitySelector::is_a(&Stone), 3)).create(world);

    let training_longsword_recipe = EntityBuilder::new()
        .with(Recipe::new_child(EntityArchetype::Archetype(training_longsword), longsword_recipe)
            .with_ingredient(&Blade, EntitySelector::is_a(&Wood), 3)).create(world);
}

#[test]
pub fn test_thing () {
    use logic;
    use samvival_core;
    use entities::Resources;
    let mut world = samvival_core::create_world();
    samvival_core::initialize_world(&mut world);
    let view = world.view();

    create_weapon_archetypes(&mut world);

    let catalog = RecipeCatalogView::of(world.view());

    let resources = view.world_data::<Resources>();
    let mut wood_bits = Vec::new();
    for i in 0..5 {
        wood_bits.push(world.clone_entity(resources.main.wood));
    }

    println!("Recipes by parent: {:?}", catalog.recipes_by_parent);
    for recipe in catalog.root_recipes() {
        let recipe_dat = view.data::<Recipe>(*recipe);
        if let EntityArchetype::Archetype(arch) = recipe_dat.result {
            let mut ingredients = HashMap::new();
            ingredients.insert(Taxon::of(&taxonomy::ingredient_types::Haft), vec![wood_bits[0], wood_bits[1]]);
            ingredients.insert(Taxon::of(&taxonomy::ingredient_types::Blade), vec![wood_bits[2], wood_bits[3], wood_bits[4]]);

            println!("Recipes by parent[x]: {:?}", catalog.recipes_by_parent.get_vec(recipe));
            if view.data::<IdentityData>(arch).effective_name() == "longsword" {
                let children_with_depth = catalog.self_and_child_recipes_of(*recipe);
                println!("All children: {:?}", children_with_depth);
                let valid_children = children_with_depth.into_iter().filter(|c| logic::crafting::is_recipe_valid_with_ingredients(view, c.recipe, &ingredients));
                println!("All valid recipes : {:?}", valid_children.clone().collect_vec());
                let most_specific_valid_child = valid_children.max_by_key(|c| c.depth);

                println!("most specific valid recipe : {:?}", most_specific_valid_child);
            }
        }
    }
}

//pub fn weapon_archetypes() -> ArchetypeLibrary {
//    let mut archetypes_by_name = HashMap::new();
//
//
//    archetypes_by_name.insert(strf("longsword"), EntityBuilder::new()
//        .with_creator(|world| ItemData {
//            attacks: vec![
//                create_attack(world, "stab", vec![&StabbingAttack, &MeleeAttack], Attack {
//                    name: strf("stab"),
//                    verb: None,
//                    attack_type: AttackType::Melee,
//                    ap_cost: 3,
//                    damage_dice: DicePool::of(1, 10),
//                    damage_bonus: 0,
//                    to_hit_bonus: 1,
//                    primary_damage_type: DamageType::Piercing,
//                    secondary_damage_type: None,
//                    range: 1,
//                    min_range: 0,
//                    ammunition_kind: None,
//                    stamina_cost: 0,
//                    pattern: HexPattern::Single,
//                }),
//                create_attack(world, "slash", vec![&SlashingAttack, &MeleeAttack], Attack {
//                    name: strf("slash"),
//                    verb: None,
//                    attack_type: AttackType::Melee,
//                    ap_cost: 4,
//                    damage_dice: DicePool::of(2, 6),
//                    damage_bonus: 1,
//                    to_hit_bonus: 0,
//                    primary_damage_type: DamageType::Slashing,
//                    secondary_damage_type: None,
//                    range: 1,
//                    min_range: 0,
//                    ammunition_kind: None,
//                    stamina_cost: 0,
//                    pattern: HexPattern::Single,
//                })],
//            ..Default::default()
//        })
//        .with(IdentityData::of_kind(Taxon::new(world, "longsword", &taxonomy::weapons::Sword))),
//    );
//
////    archetypes_by_name.insert(strf("shortsword"), EntityBuilder::new()
////        .with(ItemData {
////            attacks: vec![
////                Attack {
////                    name: "stab",
////                    attack_type: AttackType::Melee,
////                    ap_cost: 3,
////                    damage_dice: DicePool::of(1, 8),
////                    damage_bonus: 0,
////                    to_hit_bonus: 1,
////                    primary_damage_type: DamageType::Piercing,
////                    secondary_damage_type: None,
////                    range: 1,
////                    min_range: 0,
////                    ammunition_kind: None,
////                },
////                Attack {
////                    name: "slash",
////                    attack_type: AttackType::Melee,
////                    ap_cost: 3,
////                    damage_dice: DicePool::of(2, 4),
////                    damage_bonus: 0,
////                    to_hit_bonus: 0,
////                    primary_damage_type: DamageType::Slashing,
////                    secondary_damage_type: None,
////                    range: 1,
////                    min_range: 0,
////                    ammunition_kind: None,
////                }],
////            ..Default::default()
////        })
////        .with(IdentityData::of_kind(taxon("shortsword", &taxonomy::Sword))),
////    );
//
//    archetypes_by_name.insert(strf("longspear"), EntityBuilder::new()
//        .with_creator(|world| ItemData {
//            attacks: vec![
//                create_attack(world, "stab", vec![&StabbingAttack, &ReachAttack], Attack {
//                    name: strf("stab"),
//                    verb: None,
//                    attack_type: AttackType::Reach,
//                    ap_cost: 5,
//                    damage_dice: DicePool::of(1, 10),
//                    damage_bonus: 2,
//                    to_hit_bonus: 0,
//                    primary_damage_type: DamageType::Piercing,
//                    secondary_damage_type: None,
//                    range: 2,
//                    min_range: 2,
//                    ammunition_kind: None,
//                    stamina_cost: 0,
//                    pattern: HexPattern::Single,
//                }),
//                create_attack(world, "smack", vec![&BludgeoningAttack, &MeleeAttack], Attack {
//                    name: strf("smack"),
//                    verb: None,
//                    attack_type: AttackType::Melee,
//                    ap_cost: 3,
//                    damage_dice: DicePool::of(1, 4),
//                    damage_bonus: 0,
//                    to_hit_bonus: 1,
//                    primary_damage_type: DamageType::Bludgeoning,
//                    secondary_damage_type: None,
//                    range: 1,
//                    min_range: 0,
//                    ammunition_kind: None,
//                    stamina_cost: 0,
//                    pattern: HexPattern::Single,
//                }),
//                create_attack(world, "throw", vec![&ThrownAttack, &PiercingAttack], Attack {
//                    name: strf("throw"),
//                    verb: Some(strf("throw your spear at")),
//                    attack_type: AttackType::Thrown,
//                    ap_cost: 4,
//                    damage_dice: DicePool::of(1, 12),
//                    damage_bonus: 2,
//                    to_hit_bonus: -1,
//                    primary_damage_type: DamageType::Piercing,
//                    secondary_damage_type: None,
//                    range: 4,
//                    min_range: 2,
//                    ammunition_kind: None,
//                    stamina_cost: 0,
//                    pattern: HexPattern::Single,
//                })],
//            ..Default::default()
//        })
//        .with(IdentityData::of_kind(taxon("longspear", &taxonomy::weapons::Spear))),
//    );
//
//    archetypes_by_name.insert(strf("hatchet"), EntityBuilder::new()
//        .with_creator(|world| ItemData {
//            attacks: vec![
//                create_attack(world, "chop", vec![&SlashingAttack], Attack {
//                    name: strf("chop"),
//                    verb: None,
//                    attack_type: AttackType::Melee,
//                    ap_cost: 4,
//                    damage_dice: DicePool::of(1, 4),
//                    damage_bonus: 2,
//                    to_hit_bonus: 0,
//                    primary_damage_type: DamageType::Slashing,
//                    ..Default::default()
//                })],
//            ..Default::default()
//        })
//        .with(ToolData {
//            tool_harvest_fixed_bonus: 1,
//            tool_speed_bonus: 1,
//            tool_harvest_dice_bonus: DicePool::none(),
//        })
//        .with(IdentityData::of_kind(taxon("hatchet", &taxonomy::tools::ToolAxe))),
//    );
//
//
//    archetypes_by_name.insert(strf("pickaxe"), EntityBuilder::new()
//        .with_creator(|world| ItemData {
//            attacks: vec![
//                create_attack(world, "stab", vec![&StabbingAttack, &ImprovisedAttack], Attack {
//                    name: strf("stab"),
//                    verb: None,
//                    attack_type: AttackType::Melee,
//                    ap_cost: 8,
//                    damage_dice: DicePool::of(1, 10),
//                    damage_bonus: 1,
//                    to_hit_bonus: 0,
//                    primary_damage_type: DamageType::Piercing,
//                    ..Default::default()
//                })],
//            ..Default::default()
//        })
//        .with(ToolData {
//            tool_harvest_fixed_bonus: 0,
//            tool_speed_bonus: 1,
//            tool_harvest_dice_bonus: DicePool::of(1, 2),
//        })
//        .with(IdentityData::of_kind(taxon("pickaxe", &taxonomy::tools::Pickaxe))),
//    );
//
//    let default = EntityBuilder::new()
//        .with_creator(|world| ItemData {
//            attacks: vec![
//                create_attack(world, "default", vec![&taxonomy::Attack], Attack {
//                    name: strf("default"),
//                    verb: None,
//                    attack_type: AttackType::Melee,
//                    ap_cost: 3,
//                    damage_dice: DicePool::of(1, 6),
//                    damage_bonus: 0,
//                    to_hit_bonus: 0,
//                    primary_damage_type: DamageType::Piercing,
//                    secondary_damage_type: None,
//                    range: 1,
//                    min_range: 0,
//                    ammunition_kind: None,
//                    stamina_cost: 0,
//                    pattern: HexPattern::Single,
//                })],
//            ..Default::default()
//        })
//        .with(IdentityData::of_kind(taxon("default", &taxonomy::Weapon)));
//
//    ArchetypeLibrary {
//        archetypes_by_name,
//        default,
//    }
//}