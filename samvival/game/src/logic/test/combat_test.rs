use common::prelude::*;
use prelude::*;

use archetypes::*;
use logic::test::testbed::in_testbed;
use spectral::prelude::*;
use logic;
use data::entities::combat::CombatData;
use data::entities::item::ItemData;
use data::entities::combat::AttackRef;
use game::events::CoreEvent;
use data::entities::combat::DerivedAttackData;
use data::entities::selectors::EntitySelector;
use data::entities::taxonomy;
use data::entities::combat::*;


#[test]
pub fn test_basic_equip_and_select_attack() {
    in_testbed(|world, testbed| {
        let view = world.view();
        let character = character_archetypes().with_name("human").create(world);

        { // by default the only possible attacks should be natural ones
            let possible_attacks = logic::combat::possible_attack_refs(world, character);
            let natural_attacks = &view.data::<CombatData>(character).natural_attacks;
            assert_that(&possible_attacks.len()).is_equal_to(&natural_attacks.len());

            // and the default attack should be one of those
            let default_attack = logic::combat::default_attack(view, character);
            assert_that(&default_attack.is_some()).is_true();
            assert_that(natural_attacks).contains(&default_attack.attack_entity);

            // and the primary should be as well, it hasn't been overridden so it should just use the default
            let primary_attack = logic::combat::primary_attack_ref(view, character);
            assert_that(&primary_attack).matching_contains(|p| natural_attacks.contains(&p.attack_entity));
        }


        // give the character a spear, now that should be part of the attacks
        let spear = weapon_archetypes().with_name("longspear").create(world);
        logic::item::put_item_in_inventory(world, spear, character, false);
        logic::item::equip_item(world, spear, character, true);

        { // the spear should now also be included in the possible attacks
            let possible_attacks = logic::combat::possible_attack_refs(world, character);
            let natural_attacks = &view.data::<CombatData>(character).natural_attacks;
            let spear_attacks = &view.data::<ItemData>(spear).attacks;
            assert_that(&possible_attacks.len()).is_equal_to(&(natural_attacks.len() + spear_attacks.len()));

            // and the default attack should be one of the spear attacks now
            let default_attack = logic::combat::default_attack(view, character);
            assert_that(&default_attack.is_some()).is_true();
            assert_that(spear_attacks).contains(&default_attack.attack_entity);
            let weapon_for_default = default_attack.resolve_weapon(world, character);
            assert_that(&weapon_for_default).contains(&spear);

            // and the primary should be as well, it hasn't been overridden so it should just use the default
            let primary_attack = logic::combat::primary_attack_ref(view, character);
            assert_that(&primary_attack).contains(&default_attack);

            // explicitly override the active attack to point at a natural attack
            world.modify_with_desc(character, CombatData::active_attack.set_to(AttackRef::new(natural_attacks[0], character)), None);
            world.add_event(CoreEvent::TimePassed);

            // now the primary should be the natural attack instead
            let primary_attack = logic::combat::primary_attack_ref(view, character);
            assert_that(&primary_attack).is_some();
            assert_that(natural_attacks).contains(&primary_attack.unwrap().attack_entity);

            // now clear it
            world.modify_with_desc(character, CombatData::active_attack.set_to(AttackRef::none()), None);
            world.add_event(CoreEvent::TimePassed);
        }
    })
}

#[test]
pub fn test_derived_attack() {
    in_testbed(|world, testbed| {
        let view = world.view();
        let character = character_archetypes().with_name("human").create(world);

        // give the character a spear, now that should be part of the attacks
        let spear = weapon_archetypes().with_name("longspear").create(world);
        logic::item::put_item_in_inventory(world, spear, character, false);
        logic::item::equip_item(world, spear, character, true);


        let special_attack = EntityBuilder::new()
            .with(DerivedAttackData {
                character_condition: EntitySelector::Any,
                weapon_condition: EntitySelector::is_a(&taxonomy::weapons::ReachWeapon),
                attack_condition: EntitySelector::is_a(&taxonomy::attacks::StabbingAttack).and(EntitySelector::is_a(&taxonomy::attacks::ReachAttack)),
                kind: DerivedAttackKind::PiercingStrike,
            }).create(world);

        world.modify_with_desc(character, CombatData::special_attacks.append(special_attack), None);
        world.add_event(CoreEvent::TimePassed);

        {
            let possible_attacks = logic::combat::possible_attack_refs(world, character);
            let natural_attacks = &view.data::<CombatData>(character).natural_attacks;
            let spear_attacks = &view.data::<ItemData>(spear).attacks;
            let special_attacks = &view.data::<CombatData>(character).special_attacks;

            // there should now be one special attack, and there should be one spear attack that matches its criteria, which means
            // there should now be one more attack besides those derived from the spear and the natural
            assert_that(&special_attacks.len()).is_equal_to(1);
            assert_that(&possible_attacks.len()).is_equal_to(&(natural_attacks.len() + spear_attacks.len() + 1));

            let mut derived_attack_refs = possible_attacks.clone();
            // remove all of the natural attacks and the spear attacks. What remains should be the derived attack
            derived_attack_refs.retain(|sa| !natural_attacks.contains(&sa.attack_entity) && !spear_attacks.contains(&sa.attack_entity));
            assert_that(&derived_attack_refs.len()).is_equal_to(1);
            let derived_attack_ref = derived_attack_refs.first().unwrap();

            let derived_attack_opt = derived_attack_ref.resolve_attack_and_weapon(view, character);
            assert_that(&derived_attack_opt).is_some();
            if let Some((derived_attack, weapon)) = derived_attack_opt {
                assert_that(&weapon).is_equal_to(&spear);
                // the derived attack makes the base attack have line-2 pattern
                assert_that(&derived_attack.pattern).is_equal_to(&HexPattern::Line(0, 2));
            }
        }
    });
}