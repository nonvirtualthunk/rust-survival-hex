//use data::entities::modifiers::EquipItemMod;
//use data::entities::modifiers::ItemHeldByMod;
use data::events::GameEvent;
//use data::entities::modify;
use data::entities::combat::CombatData;
use data::entities::combat::AttackRef;
use data::entities::PositionData;
use data::entities::tile::*;
use game::reflect::*;
use data::entities::inventory::EquipmentData;
//use data::entities::modifiers::UnequipItemMod;
use data::entities::item::ItemData;
use common::hex::*;
use data::entities::inventory::*;
use data::entities::{StackWith, EntityMetadata};
//use game::EntityMetadata;
use prelude::*;
use common::prelude::*;
use data::entities::common_entities::LookupSignifier;

pub fn put_item_in_inventory(world: &mut World, item : Entity, inventory : Entity) -> bool {
    let view = world.view();
    if let Some(stack) = view.data_opt::<StackData>(item) {
        stack.entities.iter().all(|e| put_item_in_inventory(world, item, inventory))
    } else {
        let inv_data = view.data::<InventoryData>(inventory);
        let item_data = view.data::<ItemData>(item);

        let stack_to_add_to = if item_data.stack_limit > 1 { inv_data.items.find(|i| can_stack_entity_in_existing_stack(view, *i, item)) } else { None };
        println!("Stack to add to {:?}", stack_to_add_to);

        if let Some(stack) = stack_to_add_to {
            world.modify(item, ItemData::in_inventory_of.set_to(Some(inventory)));
            world.modify(*stack, StackData::entities.append(item));
            world.add_event(GameEvent::AddToInventory { item, to_inventory: inventory });
            true
        } else {
            let inv_limit = inv_data.inventory_size.unwrap_or(1000000);
            if inv_limit > inv_data.items.len() as u32 {
                let item_to_add = if item_data.stack_limit > 1 {
                    EntityBuilder::new().with(StackData { entities : vec![item], stack_limit : item_data.stack_limit }).create(world)
                } else {
                    item
                };

                world.modify(item, ItemData::in_inventory_of.set_to(Some(inventory)));
                world.modify(inventory, InventoryData::items.append(item_to_add));
                world.add_event(GameEvent::AddToInventory { item, to_inventory: inventory });
                true
            } else {
                trace!("Could not put in inventory of {}, inventory size was only {}", view.signifier(inventory), inv_limit);
                false
            }
        }
    }
}

#[derive(PartialEq,Debug)]
pub enum TransferResult {
    None,
    Some,
    All,
}

pub fn transfer_item(world: &mut World, item : Entity, from : Entity, to : Entity) -> TransferResult {
    let view = world.view();
    let items = match view.data_opt::<StackData>(item) {
        Some(stack) => stack.entities.clone(),
        None => vec![item],
    };

    let mut transferred = 0;
    let total_possible = items.len();
    for item in items {
        if is_item_in_inventory_of(world, item, from) {
            if put_item_in_inventory(world, item, to) {
                remove_item_from_inventory(world, item, from);
                transferred += 1;
            }
        } else {
            warn!("Attempted to transfer {} from {} to {}, but was not in source inventory to transfer", view.signifier(item), view.signifier(from), view.signifier(to));
        }
    }

    if transferred == 0 {
        TransferResult::None
    } else if transferred == total_possible {
        TransferResult::All
    } else {
        TransferResult::Some
    }
}

/// checks whether the given entity is a stack that can hold the to_hold entity
pub fn can_stack_entity_in_existing_stack(view: &WorldView, stack_entity : Entity, to_hold : Entity) -> bool {
    if let Some(stack) = view.data_opt::<StackData>(stack_entity) {
        if stack.stack_limit > stack.entities.len() as i32 {
            if let Some(first_ent) = stack.entities.first() {
                if can_items_stack_together(view, *first_ent, to_hold) {
                    return true;
                }
            } else { warn!("Entity-less stack encountered") }
        }
    }
    false
}

/// checks whether the base_item can have to_be_stacked stacked with it. Does not check stack limits
pub fn can_items_stack_together(view : &WorldView, base_item : Entity, to_be_stacked: Entity) -> bool {
    let base_item = item_or_first_in_stack(view, base_item);
    if let Some(base_item_data) = view.data_opt::<ItemData>(base_item) {
        match &base_item_data.stack_with {
            StackWith::SameArchetype => {
                let base_arch = view.data::<EntityMetadata>(base_item).archetype;
                let other_arch = view.data::<EntityMetadata>(to_be_stacked).archetype;
                if ! base_arch.is_sentinel() && ! other_arch.is_sentinel() {
                    base_arch == other_arch
                } else { false }
            },
            StackWith::Custom(sel) => sel.matches(view, to_be_stacked)
        }
    } else {
        warn!("Checking item stackability of non-item entity: {}", view.signifier(base_item));
        false
    }
}

pub fn item_stacks_in_inventory(view: &WorldView, inventory : Entity) -> Vec<(Entity, StackData)> {
    view.data::<InventoryData>(inventory).items.iter()
        .filter(|i| view.has_data::<StackData>(**i))
        .map(|i| (*i, view.data::<StackData>(*i).clone()))
        .collect_vec()
}


/// removes the item indicated from the inventory specified, if possible. Accounts for equipped as well
pub fn remove_item_from_inventory(world: &mut World, item : Entity, inventory : Entity) {
    let view = world.view();

    if is_item_equipped_by(view, item, inventory) {
        unequip_item(world, item, inventory, true);
    }

    let inv_data = view.data::<InventoryData>(inventory);
    if inv_data.items.contains(&item) {
        if view.has_data::<ItemData>(item) { world.modify(item, ItemData::in_inventory_of.set_to(None)); }
        world.modify(inventory, InventoryData::items.remove(item));
        world.add_event(GameEvent::RemoveFromInventory { item, from_inventory: inventory });
    } else {
        for (stack_ent, stack_d) in item_stacks_in_inventory(view, inventory) {
            if stack_d.entities.contains(&item) {
                let remove_stack = stack_d.entities.len() == 1;
                world.modify(item, ItemData::in_inventory_of.set_to(None));
                world.modify(stack_ent, StackData::entities.remove(item));
                if remove_stack {
                    world.modify(inventory, InventoryData::items.remove(stack_ent));
                }
                world.add_event(GameEvent::RemoveFromInventory { item, from_inventory: inventory });
            }
        }
    }
}

pub fn equip_item(world: &mut World, item : Entity, character : Entity, trigger_event : bool) {
    let world_view = world.view();
    let (item,character) = if world_view.has_data::<EquipmentData>(item) && ! world_view.has_data::<EquipmentData>(character) {
        warn!("Equip item called with an item on the right hand and a character on the left, swapping");
        (character, item)
    } else {
        (item, character)
    };

    if ! is_item_in_inventory_of(world, item, character) {
        error!("Attempting to equip an item that is not already in inventory, item {}, character {}", world_view.signifier(item), world_view.signifier(character));
    } else {
        world.modify_with_desc(character, EquipmentData::equipped.append(item), None);

        if world_view.data::<CombatData>(character).active_attack.is_none() {
            let item_attack_ref = AttackRef::of_primary_from(world.view(), item);
            if item_attack_ref.is_some() {
                world.modify_with_desc(character, CombatData::active_attack.set_to(item_attack_ref), "item equipped");
            }
        }

        if trigger_event {
            world.add_event(GameEvent::Equip { character, item });
        }
    }
}

pub fn equipped_items(world: &WorldView, character : Entity) -> Vec<Entity> {
    world.data_opt::<EquipmentData>(character).map(|e| &e.equipped).cloned().unwrap_or_else(||Vec::new())
}

pub fn is_item_equipped_by(world: &WorldView, item : Entity, character : Entity) -> bool {
    if let Some(equip) = world.data_opt::<EquipmentData>(character) {
        equip.equipped.contains(&item)
    } else {
        false
    }
}


pub fn items_in_inventory(world: &WorldView, inventory : Entity) -> Vec<Entity> {
    let inv_data = world.data::<InventoryData>(inventory);
    let mut res = Vec::new();

    for item in &inv_data.items {
        match world.data_opt::<StackData>(*item) {
            Some(stack) => res.extend(&stack.entities),
            None => res.push(*item)
        }
    }

    res
}

pub fn is_item_in_inventory_of(world: &WorldView, item : Entity, character : Entity) -> bool {
    if world.has_data::<StackData>(item) {
        world.data::<InventoryData>(character).items.contains(&item)
    } else {
        items_in_inventory(world, character).contains(&item)
    }
}


pub fn unequip_item(world: &mut World, item : Entity, from_character : Entity, trigger_event : bool) {
    if is_item_equipped_by(world, item, from_character) {
        world.modify_with_desc(from_character, EquipmentData::equipped.remove(item), None);

        let active_attack = world.view().data::<CombatData>(from_character).active_attack.attack_entity;
        let active_counter_attack = world.view().data::<CombatData>(from_character).active_counterattack.attack_entity;
        // attack entity is no longer the entity that conatins the attack, that's why this is crashing, we should not be targeting the referenced entity directly
        if world.data::<ItemData>(item).attacks.contains(&active_attack) {
            world.modify_with_desc(from_character, CombatData::active_attack.set_to(AttackRef::none()), "item unequipped");
        }
        if world.data::<ItemData>(item).attacks.contains(&active_counter_attack) {
            world.modify_with_desc(from_character, CombatData::active_counterattack.set_to(AttackRef::none()), "item unequipped");
        }

        if trigger_event {
            world.add_event(GameEvent::Unequip { character : from_character, item });
        }
    } else {
        warn!("Attempted to unequip non-equipped item");
    }
}

pub fn place_item_in_world(world: &mut World, item : Entity, at_pos : AxialCoord) {
    //TODO: any sort of checking, make sure it's not held

    let tile_ent = world.tile_ent_opt(at_pos).map(|e| e.entity);
    if let Some(tile_ent) = tile_ent {
        world.ensure_data::<InventoryData>(tile_ent);
        put_item_in_inventory(world, item, tile_ent);

        world.add_event(GameEvent::EntityAppears { entity : item, at : at_pos });
    } else {
        warn!("Attempted to place an item in the world at an invalid location: {:?}", at_pos);
    }
}

pub fn inventory_limit_remaining_for(world: &WorldView, inventory_entity : Entity, to_hold : Entity) -> Option<i32> {
    if let Some(inv) = world.data_opt::<InventoryData>(inventory_entity) {
        if let Some(size) = inv.inventory_size {
            let mut stack_capacity = 0;

            let to_hold_stack_limit = world.data_opt::<ItemData>(to_hold).map(|id| id.stack_limit).unwrap_or(1);

            if to_hold_stack_limit > 1 {
                // if there's a fixed inventory size, check for any stacks that can be added to
                for item in &inv.items {
                    // if this slot is used by a stack of items that have the same kind as the item to hold, then we can add it
                    if let Some(stack) = world.data_opt::<StackData>(*item) {
                        if let Some(first_ent) = stack.entities.first() {
                            let slots_remaining_in_stack = stack.stack_limit - stack.entities.len() as i32;
                            if slots_remaining_in_stack > 0 && can_items_stack_together(world, *first_ent, to_hold) {
                                stack_capacity += slots_remaining_in_stack;
                            }
                        } else { warn!("stack of entities with no entities in it, this is not allowed"); }
                    }
                }
            }

            let free_slots = (size as i32 - inv.items.len() as i32).max(0);

            Some(stack_capacity + free_slots * to_hold_stack_limit)
        } else {
            None
        }
    } else {
        Some(0)
    }
}


pub fn destroy_item(world: &mut World, item : Entity) -> bool {
    let view = world.view();
    if let Some(item_data) = view.data_opt::<ItemData>(item) {
        if let Some(in_inventory) = item_data.in_inventory_of {
            remove_item_from_inventory(world, item, in_inventory);
        }

        world.destroy_entity(item);

        true
    } else { error!("Attempted to destroy an item that was not an item: {}", view.signifier(item)); false }
}

/// returns the item given or, if it is a stack of items, the first item in the stack
pub fn item_or_first_in_stack(view : &WorldView, item : Entity) -> Entity {
    if let Some(stack_data) = view.data_opt::<StackData>(item) {
        if let Some(first) = stack_data.entities.first() {
            *first
        } else {
            error!("Empty stack encountered when checking item usability");
            Entity::sentinel()
        }
    } else {
        item
    }
}