use common::prelude::*;
use prelude::*;
use entities::reactions::*;
use entities::actions::*;
use entities::common_entities::*;
use entities::AllegianceData;
use entities::combat::CombatData;
use game::ModifierReference;
use logic;
use prelude::GameEvent;


pub fn can_use_reaction(world: &WorldView, ent: Entity, reaction_type: ReactionTypeRef) -> bool {
    (reaction_type.resolve().condition)().matches(world, ent)
}

pub fn trigger_reactions_for_event(world: &mut World, event: &GameEventWrapper<GameEvent>) {
    if let GameEvent::FactionTurn { faction, .. } = event.event {
        let view = world.view();

        for (ent, ent_allegiance) in view.entities_with_data::<AllegianceData>() {
            if ent_allegiance.faction == faction {
                if let Some(reaction) = view.data_opt::<ActionData>(*ent).map(|ad: &ActionData| ad.active_reaction) {
                    let ent = *ent;
                    if can_use_reaction(view, ent, reaction) {
                        let modifier_key = format!("{:?}-reaction-modifier", reaction);

                        if event.is_ended() {
                            let modifier_applied = match reaction {
                                ReactionTypeRef::Dodge => {
                                    let increase_dodge_by = (world.view().data::<CombatData>(ent).dodge_bonus * 2).max(2);
                                    world.modify_with_desc(ent, CombatData::dodge_bonus.add(increase_dodge_by), "dodge reaction")
                                }
                                ReactionTypeRef::Counterattack => {
                                    if let Some(counter_attack) = logic::combat::counter_attack_ref_to_use(view, ent) {
                                        if let Some(counter_attack) = counter_attack.resolve(view, ent) {
                                            let increase_counters_by = view.data::<CharacterData>(ent).action_points.max_value() / counter_attack.ap_cost as i32;
                                            world.modify_with_desc(ent, CombatData::counters_remaining.increase_by(increase_counters_by), "counterattack reaction")
                                        } else { ModifierReference::sentinel() }
                                    } else { ModifierReference::sentinel() }
                                }
                                ReactionTypeRef::Defend => {
                                    world.modify_with_desc(ent, CombatData::defense_bonus.add(1), "defense reaction")
                                }
                                ReactionTypeRef::Block => {
                                    world.modify_with_desc(ent, CombatData::defense_bonus.add(1), "block reaction")
                                }
                            };

                            if let Some(modifier) = modifier_applied.as_opt() {
                                world.modify_with_desc(ent, ModifierTrackingData::modifiers_by_key.set_key_to(modifier_key, modifier.clone()), None);
                                world.add_event(GameEvent::ReactionEffectApplied { entity: ent });
                            }
                        } else if event.is_starting() {
                            if let Some(prev_modifier) = view.data::<ModifierTrackingData>(ent).modifiers_by_key.get(&modifier_key) {
                                world.disable_modifier(prev_modifier.clone());
                                world.add_event(GameEvent::EffectEnded { entity: None });
                            }
                        }
                    }
                }
            }
        }
    }
}