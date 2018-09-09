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
use game::EntityMetadata;
use prelude::*;
use common::prelude::*;

/// clone_on_add is used when creating an item into an inventory that may be able to stack, in that case you don't
/// want to clone-create the entity ahead of time, because then you pay the cost even if it stacks, rather you want
/// to clone only if you have to create a new stack
pub fn put_item_in_inventory(world: &mut World, item : Entity, inventory : Entity) -> bool {
    let view = world.view();
    let inv_data = view.data::<InventoryData>(inventory);
    let item_data = view.data::<ItemData>(item);

    let stack_to_add_to = if item_data.stack_limit > 1 { inv_data.items.find(|i| can_stack_entity_in(view, *i, item)) } else { None };

    if let Some(stack) = stack_to_add_to {
        world.modify(item, ItemData::in_inventory_of.set_to(Some(inventory)));
        world.modify(*stack, StackData::entities.append(item));
        world.add_event(GameEvent::AddToInventory { item, to_inventory: inventory });
        true
    } else {
        if inv_data.inventory_size.unwrap_or(1000000) > inv_data.items.len() as u32 {
            let item_to_add = if item_data.stack_limit > 1 {
                EntityBuilder::new().with(StackData { entities : vec![item], stack_limit : item_data.stack_limit }).create(world)
            } else {
                item
            };

            world.modify(item, ItemData::in_inventory_of.set_to(Some(inventory)));
            world.modify(inventory, InventoryData::items.append(item_to_add));
            world.add_event(GameEvent::AddToInventory { item, to_inventory: inventory });
            true
        } else { false }
    }
}

/// checks whether the given entity is a stack that can hold the to_hold entity
pub fn can_stack_entity_in(view: &WorldView, stack_entity : Entity, to_hold : Entity) -> bool {
    if let Some(stack) = view.data_opt::<StackData>(stack_entity) {
        if stack.stack_limit > stack.entities.len() as i32 {
            if let Some(first_ent) = stack.entities.first() {
                if view.data::<ItemData>(*first_ent).stack_with.matches(view, to_hold) {
                    return true;
                }
            } else { warn!("Entity-less stack encountered") }
        }
    }
    false
}

pub fn item_stacks_in_inventory(view: &WorldView, inventory : Entity) -> Vec<(Entity, StackData)> {
    view.data::<InventoryData>(inventory).items.iter()
        .filter(|i| view.has_data::<StackData>(**i))
        .map(|i| (*i, view.data::<StackData>(*i).clone()))
        .collect_vec()
}

pub fn remove_item_from_inventory(world: &mut World, item : Entity, inventory : Entity) {
    let view = world.view();
    let inv_data = view.data::<InventoryData>(inventory);
    if inv_data.items.contains(&item) {
        world.modify_with_desc(item, ItemData::in_inventory_of.set_to(None), None);
        world.modify_with_desc(inventory, InventoryData::items.remove(item), None);
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

    world.modify_with_desc(character, EquipmentData::equipped.append(item), None);
    if ! is_item_in_inventory_of(world, item, character) {
        put_item_in_inventory(world, item, character);
    }


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
    items_in_inventory(world, character).contains(&item)
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
                            if slots_remaining_in_stack > 0 && world.data::<ItemData>(*first_ent).stack_with.matches(world, to_hold) {
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