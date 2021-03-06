use common::prelude::*;
use prelude::*;

use entities::SkillData;
use entities::Skill;
use data::entities::common_entities::LookupSignifier;

pub fn skill_level(view : &WorldView, entity : Entity, skill : Skill) -> i32 {
    if let Some(skill_data) = view.data_opt::<SkillData>(entity) {
        skill_level_from(skill_data, skill)
    } else { warn!("Requesting skill level ({:?}) for non-skilled entity: {:?}", skill, view.signifier(entity)); 0 }
}

pub fn skill_level_from(skill_data: &SkillData, skill : Skill) -> i32 {
    skill_data.skill_bonuses.get(&skill).unwrap_or(&0) + level_for_xp(skill_data.cur_skill_xp(skill))
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

pub fn xp_required_for_level(lvl : i32) -> i32 {
    let lvl = (lvl + 1) as f64; // shift over by 1 so that getting to level 1 doesn't cost 0 xp
    ((0.5 * lvl.powf(2.0) - 0.5 * lvl) * 10.0) as i32
}

pub fn level_for_xp(xp : i32) -> i32 {
    for i in 0 .. 100 {
        if xp_required_for_level(i) > xp {
            return i - 1;
        }
    }
    100
}