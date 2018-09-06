//use data::entities::modifiers::EquipItemMod;
//use data::entities::modifiers::ItemHeldByMod;
use data::events::GameEvent;
use game::prelude::*;
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

/// clone_on_add is used when creating an item into an inventory that may be able to stack, in that case you don't
/// want to clone-create the entity ahead of time, because then you pay the cost even if it stacks, rather you want
/// to clone only if you have to create a new stack
pub fn put_item_in_inventory(world: &mut World, item : Entity, inventory : Entity, clone_on_add : bool) {
    world.modify(item, ItemData::in_inventory_of.set_to(Some(inventory)));
    world.modify(inventory, InventoryData::items.append(item));
    world.add_event(GameEvent::AddToInventory { item, to_inventory: inventory });
}

pub fn remove_item_from_inventory(world: &mut World, item : Entity, inventory : Entity) {
    world.modify_with_desc(item, ItemData::in_inventory_of.set_to(None), None);
    world.modify_with_desc(inventory, InventoryData::items.remove(item), None);
    world.add_event(GameEvent::RemoveFromInventory { item, from_inventory: inventory });
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
        put_item_in_inventory(world, item, character, false);
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
pub fn is_item_in_inventory_of(world: &WorldView, item : Entity, character : Entity) -> bool {
    if let Some(inv) = world.data_opt::<InventoryData>(character) {
        inv.items.contains(&item)
    } else {
        false
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
        put_item_in_inventory(world, item, tile_ent, false);

        world.add_event(GameEvent::EntityAppears { entity : item, at : at_pos });
    } else {
        warn!("Attempted to place an item in the world at an invalid location: {:?}", at_pos);
    }
}

pub fn inventory_limit_remaining_for(world: &WorldView, inventory_entity : Entity, to_hold : Entity) -> Option<i32> {
    if let Some(inv) = world.data_opt::<InventoryData>(inventory_entity) {
        if let Some(size) = inv.inventory_size {
            let mut stack_capacity = 0;

            let to_hold_cloned_from = world.data::<EntityMetadata>(to_hold).cloned_from;
            // if there's a fixed inventory size, check for any stacks that can be added to
            for item in inv.items {
                // if this slot is used by a stack of items that are the same as, or cloned from, the stack entity
                if let Some(stack) = world.data_opt::<StackData>(item) {
                    if stack.stack_of == to_hold || to_hold_cloned_from == Some(stack.stack_of) {
                        stack_capacity += stack.stack_limit - stack.stack_size
                    }
                }
            }

            let effective_stack_size = if let Some(item_data) = world.data_opt::<ItemData>(to_hold) {
                item_data.stack_limit.max(1)
            } else {
                1
            };

            let free_slots = (size - inv.items.len()).max(0);

            stack_capacity + free_slots * effective_stack_size
        } else {
            None
        }
    } else {
        Some(0)
    }
}