use common::prelude::*;
use prelude::*;

use entities::SkillData;
use entities::Skill;

pub fn skill_level(view : &WorldView, entity : Entity, skill : Skill) -> i32 {
    if let Some(skill_data) = view.data_opt::<SkillData>(entity) {
        skill_level_from(skill_data, skill)
    } else { warn!("Requesting skill level ({:?}) for non-skilled entity: {:?}", skill, view.signifier(entity)); 0 }
}

pub fn skill_level_from(skill_data: &SkillData, skill : Skill) -> i32 {
    skill_data.skill_bonuses.get(&skill).unwrap_or(&0) + Skill::level_for_xp(skill_data.cur_skill_xp(skill))
}

pub fn skill_levels(view : &WorldView, entity: Entity) -> Vec<(Skill, i32)> {
    if let Some(skill_data) = view.data_opt::<SkillData>(entity) {
        let mut res = Vec::new();
        for (skill,xp) in &skill_data.skill_xp {
            res.push((*skill, skill_level_from(skill_data, *skill)));
        }
        res
    } else { warn!("Requesting all skills for non-skilled entity: {:?}", view.signifier(entity)); Vec::new() }
}