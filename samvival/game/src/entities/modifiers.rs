use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::hex::AxialCoord;
use game::core::*;
use game::modifiers::ConstantModifier;
use game::modifiers::LimitedModifier;
use game::modifiers::DynamicModifier;

use entities::*;
use game::world::World;
use entities::combat::CombatData;
use entities::combat::AttackReference;


pub fn modify<T : EntityData, CM : ConstantModifier<T>>(world : &mut World, ent : Entity, modifier : CM) {
    world.add_constant_modifier(ent, modifier);
}

pub struct SkillXPMod(pub Skill, pub u32);
impl ConstantModifier<SkillData> for SkillXPMod {
    fn modify(&self, data: &mut SkillData) {
        let SkillXPMod(skill, xp) = *self;
        data.skill_xp[skill] += xp;
    }
}

pub struct SkillMod(pub Skill, pub u32);
impl ConstantModifier<SkillData> for SkillMod {
    fn modify(&self, data: &mut SkillData) {
        data.skill_bonuses[self.0] += self.1;
    }
}



pub struct ReduceActionsMod(pub u32);
impl ConstantModifier<CharacterData> for ReduceActionsMod {
    fn modify(&self, data: &mut CharacterData) {
        data.action_points.reduce_by(self.0 as i32);
    }
}

pub struct ReduceStaminaMod(pub Sext);
impl ConstantModifier<CharacterData> for ReduceStaminaMod {
    fn modify(&self, data: &mut CharacterData) {
        data.stamina.reduce_by(self.0);
    }
}

pub struct DamageMod(pub i32);
impl ConstantModifier<CharacterData> for DamageMod {
    fn modify(&self, data: &mut CharacterData) {
        data.health.reduce_by(self.0);
    }
}

pub struct ReduceMoveMod(pub Sext);
impl ConstantModifier<CharacterData> for ReduceMoveMod {
    fn modify(&self, data: &mut CharacterData) {
        data.moves = data.moves - self.0;
    }
}

pub struct EndMoveMod;
impl ConstantModifier<CharacterData> for EndMoveMod {
    fn modify(&self, data: &mut CharacterData) {
        data.moves = Sext::zero();
    }
}

pub struct ResetCharacterTurnMod;
impl ConstantModifier<CharacterData> for ResetCharacterTurnMod {
    fn modify(&self, data: &mut CharacterData) {
        data.moves = Sext::zero();
        data.action_points.reset();
        data.stamina.recover_by(data.stamina_recovery);
    }
}
pub struct ResetCombatTurnMod;
impl ConstantModifier<CombatData> for ResetCombatTurnMod {
    fn modify(&self, data: &mut CombatData) {
        data.counters = Reduceable::new(0);
    }
}


pub struct SetHexOccupantMod(pub Option<Entity>);
impl ConstantModifier<TileData> for SetHexOccupantMod {
    fn modify(&self, data: &mut TileData) { data.occupied_by = self.0; }
}

pub struct CarryItemMod(pub Entity);
impl ConstantModifier<InventoryData> for CarryItemMod {
    fn modify(&self, data: &mut InventoryData) {
        data.inventory.push(self.0);
    }
}

pub struct EquipItemMod(pub Entity);
impl ConstantModifier<InventoryData> for EquipItemMod {
    fn modify(&self, data: &mut InventoryData) {
        data.equipped.push(self.0);
    }
}

pub struct ItemHeldByMod(pub Option<Entity>);
impl ConstantModifier<ItemData> for ItemHeldByMod{
    fn modify(&self, data: &mut ItemData) {
        data.held_by = self.0;
    }
}

pub struct SetActiveAttackMod(pub AttackReference);
impl ConstantModifier<CombatData> for SetActiveAttackMod {
    fn modify(&self, data: &mut CombatData) {
        data.active_attack = self.0.clone();
    }
}