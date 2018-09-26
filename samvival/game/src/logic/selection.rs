use prelude::CharacterData;
use entity_util::position_of;
use logic;
use prelude::IdentityData;
use data::entities::selectors::EntitySelector;
use data::entities::selectors::EntitySelector::*;
use data::entities::*;
use game::prelude::*;

pub trait SelectorMatches {
    fn matches(&self, world: &WorldView, entity: Entity) -> bool;

    fn matches_attack(&self, view : &WorldView, attack: &Attack) -> bool;

    fn matches_identity(&self, view : &WorldView, ident: &IdentityData) -> bool;

    fn matches_any(&self, world: &WorldView, entities: &Vec<Entity>) -> bool {
        entities.iter().any(|e| self.matches(world, *e))
    }
}


use common::ExtendedCollection;
impl SelectorMatches for EntitySelector {
    fn matches(&self, world: &WorldView, entity: Entity) -> bool {
        match self {
            Is(other_entity) => &entity == other_entity,
            IsEquivalentTo(other_entity) => { warn!("IsEquivalentTo selector not fully defined yet"); &entity == other_entity },
            IsCharacter => world.has_data::<CharacterData>(entity),
            IsTile => world.has_data::<TileData>(entity),
            Friend { of } =>
                IsCharacter.matches(world, *of) &&
                    IsCharacter.matches(world, entity) &&
                    world.character(*of).allegiance.faction == world.character(entity).allegiance.faction,
            Enemy { of } =>
                IsCharacter.matches(world, *of) &&
                    IsCharacter.matches(world, entity) &&
                    world.character(*of).allegiance.faction == world.character(entity).allegiance.faction,
            InMoveRange { hex_range, of } => {
                if let Some(end_point) = position_of(entity, world) {
                    if let Some((_, cost)) = logic::movement::path_to(world, *of, end_point) {
                        return cost < *hex_range as f64
                    }
                }
                false
            },
            HasInventory => world.has_data::<InventoryData>(entity),
            HasStamina(stam) => world.data_opt::<CharacterData>(entity).map(|c| c.stamina.cur_value() >= *stam).unwrap_or(false),
            HasAP(ap) => world.data_opt::<CharacterData>(entity).map(|c| c.action_points.cur_value() >= *ap).unwrap_or(false),
            HasEquipmentKind(taxon) => {
                let sub_selector = EntitySelector::IsA(taxon.clone());
                world.data_opt::<EquipmentData>(entity).map(|ed| ed.equipped.iter().any(|eq| sub_selector.matches(world, *eq))).unwrap_or(false)
            },
            HasAttackKind(taxon) => {
                let sub_selector = EntitySelector::IsA(taxon.clone());
                logic::combat::possible_attack_refs(world, entity).iter().any(|eq| sub_selector.matches(world, eq.attack_entity))
            },
            HasSkillLevel(skill, level) => logic::skill::skill_level(world, entity, *skill) >= *level,
            IsA(taxon) => world.data_opt::<IdentityData>(entity).filter(|i| i.kinds.any_match(|k| k.is_a(world, &taxon))).is_some(),
            And(a,b) => a.matches(world, entity) && b.matches(world, entity),
            Or(a,b) => a.matches(world, entity) || b.matches(world, entity),
            Any => true,
            None => false,
        }
    }

    fn matches_attack(&self, view : &WorldView, attack: &Attack) -> bool {
        match self {
            And(a,b) => a.matches_attack(view, attack) && b.matches_attack(view, attack),
            Or(a,b) => a.matches_attack(view, attack) || b.matches_attack(view, attack),
            Any => true,
            _ => false
        }
    }

    fn matches_identity(&self, view : &WorldView, ident: &IdentityData) -> bool {
        match self {
            IsA(taxon) => ident.kinds.any_match(|k| k.is_a(view, taxon)),
            And(a,b) => a.matches_identity(view, ident) && b.matches_identity(view, ident),
            Or(a,b) => a.matches_identity(view, ident) || b.matches_identity(view, ident),
            Any => true,
            _ => false,
        }
    }
}