use entities::modifiers::EquipItemMod;
use entities::modifiers::ItemHeldByMod;
use events::GameEvent;
use game::World;
use game::Entity;
use entities::modify;
use entities::combat::CombatData;
use entities::combat::AttackReference;
use entities::PositionData;
use game::reflect::*;
use entities::inventory::InventoryData;
use entities::modifiers::UnequipItemMod;
use entities::item::ItemData;
use common::hex::*;

pub fn equip_item(world: &mut World, character : Entity, item : Entity, trigger_event : bool) {
    modify(world, character, EquipItemMod(item));
    modify(world, item, ItemHeldByMod(Some(character)));

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


pub fn unequip_item(world: &mut World, character : Entity, item : Entity, trigger_event : bool) {
    if world.view().data::<InventoryData>(character).equipped.contains(&item) {
        modify(world, character, UnequipItemMod(item));
        world.modify(item, ItemData::held_by.set_to(None), None);

        if world.view().data::<CombatData>(character).active_attack.entity == item {
            world.modify(character, CombatData::active_attack.set_to(AttackReference::none()), "item unequipped");
        }

        if trigger_event {
            world.add_event(GameEvent::Unequip { character, item });
        }
    } else {
        warn!("Attempted to unequip non-equipped item");
    }
}

pub fn place_item_in_world(world: &mut World, item : Entity, at_pos : AxialCoord) {
    //TODO: any sort of checking, make sure it's not held

    if ! world.view().has_data::<PositionData>(item) {
        world.attach_data(item, &PositionData { hex : at_pos });
    } else {
        world.modify(item, PositionData::hex.set_to(at_pos), "placed in world");
    }

    world.add_event(GameEvent::EntityAppears { entity : item, at : at_pos });
}