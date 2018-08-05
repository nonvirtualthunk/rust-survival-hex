use entities::modifiers::EquipItemMod;
use entities::modifiers::ItemHeldByMod;
use events::GameEvent;
use World;
use world::Entity;
use entities::modify;

pub fn equip_item(world: &mut World, character : Entity, item : Entity) {
    modify(world, character, EquipItemMod(item));
    modify(world, item, ItemHeldByMod(Some(character)));

    world.add_event(GameEvent::Equip { character, item });
}
