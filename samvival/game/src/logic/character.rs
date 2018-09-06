use prelude::*;
use data::entities::character::CharacterData;
use data::entities::combat::DamageType;

use logic;


/// so the question here is...what do we want to do with sub-events, i.e. a character is being attacked, we could get an event
/// every time damage is dealt, and then when a strike is made. But really we want the damage being dealt to be a sub-aspect of
/// the strike. We might want something to trigger any time damage is done, but we don't want the mainline animations and stuff
/// to have to care about individual damage events...probably. Coordinating that would be hard, primarily, the strike animation comes
/// before the damage animation. But...if we had the strike event occur, then the damage event occur, that would actually work
/// out pretty well

/// should be called after a character has taken damage
pub fn apply_damage_to_character(world : &mut World, character : Entity, damage_amount : u32, damage_types : &[DamageType]) {
    world.modify_with_desc(character, CharacterData::health.reduce_by(damage_amount as i32), "attack damage");

    world.add_event(GameEvent::DamageTaken { entity : character, damage_taken : damage_amount, damage_types : Vec::from(damage_types) });

    if ! world.view().character(character).is_alive() {
        logic::movement::remove_entity_from_world(world, character);

        world.add_event(GameEvent::EntityDied { entity : character });
    }
}