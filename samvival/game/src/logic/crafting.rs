use common::prelude::*;
use prelude::*;

//use data::entities::ItemArchetype;
use data::entities::recipes::*;
use data::entities::{ItemArchetype, ItemData, Attack, WorthData, StackData, DamageType, Material, MaterialEffectSelector, MaterialEffect, MaterialEffectType};
use data::archetype::EntityArchetype;
use logic;
use std::collections::HashMap;


pub struct CraftingBreakdown {
    pub recipe : Entity,
    pub result_identity : IdentityData,
    pub effective_archetype : ItemArchetype
}


pub fn is_recipe_valid_with_ingredients(view : &WorldView, recipe : Entity, ingredients : &HashMap<Taxon, Vec<Entity>>) -> bool {
    let requirements = view.data::<Recipe>(recipe).effective_ingredients_by_kind(view);

    for (kind, ingredient) in &requirements {
        if let Some(entities_for_kind) = ingredients.get(kind) {
            if let Some(first) = entities_for_kind.first() {
                if ingredient.ingredient_selector.matches(view, *first) && entities_for_kind.len() >= ingredient.amount_required as usize {
                    // seems fine
                } else { return false; }
            } else { return false; }
        } else { return false; }
    }
    true
}

pub fn can_entity_craft_recipe(view: &WorldView, crafter : Entity, recipe : Entity) -> Result<(), String> {
    let equipped = logic::item::equipped_items(view, crafter);

    let recipe_dat = view.data::<Recipe>(recipe);
    for skill_used in &recipe_dat.skills_used {
        let lvl = logic::skill::skill_level(view, crafter, skill_used.skill);
        let req = skill_used.required_level.unwrap_or(-10000);
        if lvl < req {
            return Err(format!("requires level {} {}", req, skill_used.skill.to_string_infinitive()))
        }
    }

    for (tool_sel, tool_use) in &recipe_dat.tools_used {
        if let RecipeToolUse::Required = tool_use {
            if ! tool_sel.matches_any(view, &equipped) {
                return Err(format!("requires {}", tool_sel.to_string_with_article(view)))
            }
        }
    }

    Ok(())
}

pub fn compute_crafting_breakdown(world : &mut World, view : &WorldView, crafter: Entity, ingredients : &HashMap<Taxon, Vec<Entity>>, base_recipe : Entity) -> Result<CraftingBreakdown, String> {
    let recipe_catalog = RecipeCatalogView::of(view);
    let children_with_depth = recipe_catalog.self_and_child_recipes_of(base_recipe);
    let valid_children = children_with_depth.into_iter().filter(|c| is_recipe_valid_with_ingredients(view, c.recipe, &ingredients));
    let most_specific_valid_child = valid_children.max_by_key(|c| c.depth);
    if let Some(recipe) = most_specific_valid_child.map(|c| c.recipe) {
        can_entity_craft_recipe(view, crafter, recipe)?;

        let recipe_dat = view.data::<Recipe>(recipe);

        if let EntityArchetype::Archetype(archetype_entity) = recipe_dat.result {
            if let Some(item_arch) = view.data_opt::<ItemArchetype>(archetype_entity) {
                let mut arch = item_arch.clone();
                let mut ident = view.data::<IdentityData>(archetype_entity).clone();
                if let Some(name_from) = &recipe_dat.name_from_ingredient {
                    if let Taxon::ConstTaxonRef { reference } = ident.main_kind() {
                        if let Some(ingredient) = ingredients.get(&name_from).and_then(|e| e.first().cloned()) {
                            let ingredient_name = view.identity(ingredient).main_kind().name();
                            let new_main_kind = Taxon::new(world, format!("{} {}", ingredient_name, reference.name()), *reference);
                            ident.replace_main_kind(new_main_kind);
                        }
                    } else {
                        warn!("Encountered a situation in which we want to create a child of a runtime taxon, time to figure that out");
                    }
                }


                for (ingredient_type, ingredient_list) in ingredients {
                    if let Some(ingredient) = ingredient_list.first() {
                        let material_info = view.data_opt::<Material>(*ingredient);
                        if let Some(material_info) = material_info {
                            for (_,attack) in &mut arch.attacks {
                                if ingredient_type.is_a(view, &taxonomy::ingredient_types::WeaponHeadIngredient) {
                                    match attack.primary_damage_type {
                                        DamageType::Slashing => attack.damage_bonus += material_info.edge - 5,
                                        DamageType::Bludgeoning => attack.damage_bonus += (material_info.hardness + material_info.density)/2 - 5,
                                        DamageType::Piercing => attack.damage_bonus += material_info.point - 5,
                                        _ => () // do nothing
                                    }
                                } else if ingredient_type.is_a(view, &taxonomy::ingredient_types::HandleIngredient) {
                                    // if it's strong relative to its weight, give a bonus to hit, otherwise a penalty
                                    attack.to_hit_bonus += (material_info.strength - material_info.density) / 2;
                                }
                            }

                            for MaterialEffect(condition, effect) in &material_info.material_effects {
                                if let MaterialEffectSelector::IngredientType(sel_ingredient_type) = condition {
                                    if ingredient_type.is_a(view, sel_ingredient_type) {
                                        apply_material_effect(view, &mut arch, effect)
                                    }
                                } else if let MaterialEffectSelector::ItemType(sel_item_type) = condition {
                                    if ident.kinds.any_match(|k| k.is_a(view, sel_item_type)) {
                                        apply_material_effect(view, &mut arch, effect);
                                    }
                                }
                            }
                        }
                    }
                }



                let breakdown = CraftingBreakdown { recipe, effective_archetype : arch, result_identity : ident };
                Ok(breakdown)
            } else { Err(strf("non-item archetype based recipes not yet supported")) }
        } else { Err(strf("non-archetype based recipes not yet supported")) }

    } else {
        let recipe_dat = view.data::<Recipe>(base_recipe);
        if ! recipe_dat.ingredients_by_kind.keys().all(|kind| ingredients.contains_key(kind)) {
            Err(strf("Not all ingredients have been supplied"))
        } else {
            Err(strf("No valid recipes"))
        }
    }
}

fn apply_material_effect(view: &WorldView, arch : &mut ItemArchetype, effect : &MaterialEffectType) {
    match effect {
        MaterialEffectType::WeaponAttribute(attribute_type, amount) => {
            arch.attributes.increase_attribute(attribute_type, *amount)
        },
        MaterialEffectType::ToHitBonus(attack_selector, bonus) => {
            for (ident, attack) in &mut arch.attacks {
                if attack_selector.matches_identity(view, ident) || attack_selector.matches_attack(view, attack) {
                    attack.to_hit_bonus += *bonus
                }
            }
        },
        MaterialEffectType::DamageBonus(attack_selector, bonus) => {
            for (ident, attack) in &mut arch.attacks {
                if attack_selector.matches_identity(view, ident) || attack_selector.matches_attack(view, attack) {
                    attack.damage_bonus += *bonus
                }
            }
        },
    }
}

pub fn craft(world : &mut World, crafter : Entity, ingredients : &HashMap<Taxon, Vec<Entity>>, base_recipe : Entity) -> Result<Entity, String> {
    let breakdown = compute_crafting_breakdown(world, world.view(), crafter, ingredients, base_recipe)?;

    let crafted = create_item_from_archetype(world, &breakdown.effective_archetype, &breakdown.result_identity);

    for ingredient_list in ingredients.values() {
        for ingredient in ingredient_list {
            logic::item::destroy_item(world, *ingredient);
        }
    }


    Ok(crafted)
}

pub fn create_item_from_archetype(world : &mut World, archetype : &ItemArchetype, ident : &IdentityData) -> Entity {
    EntityBuilder::new()
        .with(ItemData {
            attacks : archetype.attacks.iter().cloned().map(|(ident,attack)| {
                EntityBuilder::new()
                    .with(attack)
                    .with(ident)
                    .create(world)
            }).collect(),
            stack_limit : archetype.stack_limit,
            stack_with : archetype.stack_with.clone(),
            in_inventory_of : None
        })
        .with(ident.clone())
        .with(WorthData::new(archetype.worth))
        .with_opt(archetype.tool_data.clone())
        .create(world)
}

pub fn craft_without_materials(world : &mut World, archetype : Entity) -> Entity {
    let view = world.view();
    create_item_from_archetype(world, view.data::<ItemArchetype>(archetype), view.data::<IdentityData>(archetype))
}

/// checks whether a new item can be used in a partially specified crafting operation in the given ingredient slot. Checks whether
/// the item in question is valid for that slot at all and if there are already some existing entities specified for that slot
/// verifies that it can be stacked with them
pub fn can_item_be_used_in_craft(view : &WorldView, base_recipe : Entity, item : Entity, kind : &Taxon, existing_item_assignments : &HashMap<Taxon, Vec<Entity>>) -> bool {
    let item = logic::item::item_or_first_in_stack(view, item);

    let requirements = &view.data::<Recipe>(base_recipe).ingredients_by_kind;
    if let Some(requirement) = requirements.get(kind) {
        if requirement.ingredient_selector.matches(view, item) {
            let existing_head = existing_item_assignments.get(kind).map(|v| v.first().cloned()).unwrap_or(None);

            if let Some(existing_head) = existing_head {
                logic::item::can_items_stack_together(view, existing_head, item)
            } else {
                true
            }
        } else { false }
    } else {
        error!("Attempted to check item usability in craft with invalid kind: {:?}", kind);
        false
    }
}

pub fn is_craft_fully_specified(view : &WorldView, base_recipe : Entity, item_assignments : &HashMap<Taxon, Vec<Entity>>) -> bool {
    let recipe = view.data::<Recipe>(base_recipe);
    recipe.ingredients_by_kind.iter().all(|(kind,ingredient)| item_assignments.get(kind).map(|v|v.len()).unwrap_or(0) as i32 >= ingredient.amount_required)
}