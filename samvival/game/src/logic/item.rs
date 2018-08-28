use entities::modifiers::EquipItemMod;
use entities::modifiers::ItemHeldByMod;
use events::GameEvent;
use game::prelude::*;
use entities::modify;
use entities::combat::CombatData;
use entities::combat::AttackRef;
use entities::PositionData;
use entities::tile::*;
use game::reflect::*;
use entities::inventory::EquipmentData;
use entities::modifiers::UnequipItemMod;
use entities::item::ItemData;
use common::hex::*;
use entities::inventory::InventoryData;

pub fn put_item_in_inventory(world: &mut World, item : Entity, inventory : Entity) {
    world.modify(item, ItemData::in_inventory_of.set_to(Some(inventory)), None);
    world.modify(inventory, InventoryData::items.append(item), None);
    world.add_event(GameEvent::AddToInventory { item, to_inventory: inventory });
}

pub fn remove_item_from_inventory(world: &mut World, item : Entity, inventory : Entity) {
    world.modify(item, ItemData::in_inventory_of.set_to(None), None);
    world.modify(inventory, InventoryData::items.remove(item), None);
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

    world.modify(character, EquipmentData::equipped.append(item), None);
    if ! is_item_in_inventory_of(world, item, character) {
        put_item_in_inventory(world, item, character);
    }


    if world_view.data::<CombatData>(character).active_attack.is_none() {
        let item_attack_ref = AttackRef::of_primary_from(world.view(), item);
        if item_attack_ref.is_some() {
            world.modify(character, CombatData::active_attack.set_to(item_attack_ref), "item equipped");
        }
    }

    if trigger_event {
        world.add_event(GameEvent::Equip { character, item });
    }
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
        modify(world, from_character, UnequipItemMod(item));

        let active_attack = world.view().data::<CombatData>(from_character).active_attack.attack_entity;
        let active_counter_attack = world.view().data::<CombatData>(from_character).active_counterattack.attack_entity;
        // attack entity is no longer the entity that conatins the attack, that's why this is crashing, we should not be targeting the referenced entity directly
        if world.data::<ItemData>(item).attacks.contains(&active_attack) {
            world.modify(from_character, CombatData::active_attack.set_to(AttackRef::none()), "item unequipped");
        }
        if world.data::<ItemData>(item).attacks.contains(&active_counter_attack) {
            world.modify(from_character, CombatData::active_counterattack.set_to(AttackRef::none()), "item unequipped");
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