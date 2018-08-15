use entities::modifiers::EquipItemMod;
use entities::modifiers::ItemHeldByMod;
use events::GameEvent;
use game::World;
use game::Entity;
use entities::modify;
use entities::combat::CombatData;
use entities::combat::AttackReference;
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
}

pub fn remove_item_from_inventory(world: &mut World, item : Entity, inventory : Entity) {
    world.modify(item, ItemData::in_inventory_of.set_to(None), None);
    world.modify(inventory, InventoryData::items.remove(item), None);
}

pub fn equip_item(world: &mut World, character : Entity, item : Entity, trigger_event : bool) {
    world.modify(character, EquipmentData::equipped.append(item), None);
    put_item_in_inventory(world, item, character);


    if world.view().data::<CombatData>(character).active_attack.is_none() {
        let item_attack_ref = AttackReference::of_primary_from(world.view(), item);
        if item_attack_ref.is_some() {
            world.modify(character, CombatData::active_attack.set_to(item_attack_ref), "item equipped");
        }
    }

    if trigger_event {
        world.add_event(GameEvent::Equip { character, item });
    }
}


pub fn unequip_item(world: &mut World, item : Entity, from_character : Entity, trigger_event : bool) {
    if world.view().data::<EquipmentData>(from_character).equipped.contains(&item) {
        modify(world, from_character, UnequipItemMod(item));

        if world.view().data::<CombatData>(from_character).active_attack.entity == item {
            world.modify(from_character, CombatData::active_attack.set_to(AttackReference::none()), "item unequipped");
        }
        if world.view().data::<CombatData>(from_character).active_counterattack.entity == item {
            world.modify(from_character, CombatData::active_counterattack.set_to(AttackReference::none()), "item unequipped");
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