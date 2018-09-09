use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use enum_map::EnumMap;
use std::collections::HashMap;
use game::entity;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Fields)]
pub struct SkillData {
    pub skill_bonuses: HashMap<Skill, i32>,
    pub skill_xp : HashMap<Skill, i32>
}
impl EntityData for SkillData {}


pub trait SkillDataStore {
    fn skills(&self, ent : Entity) -> &SkillData;
}
impl SkillDataStore for WorldView {
    fn skills(&self, ent: Entity) -> &SkillData {
        self.data::<SkillData>(ent)
    }
}

impl SkillData {
    pub fn cur_skill_xp(&self, skill : Skill) -> i32 {
        *self.skill_xp.get(&skill).unwrap_or(&0)
    }
}





#[derive(Enum, Debug, Clone, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
pub enum Skill {
    Dodge = 0,
    Melee = 1,
    Ranged = 2,
    MountainSurvival = 3,
    ForestSurvival = 4,
    FireMagic = 5,
    IceMagic = 6,
    Farming = 7,
    Axe = 8,
    Spear = 9,
    Sword = 10,
    Mining = 11,
}

impl Skill {
    pub fn xp_required_for_level(lvl : i32) -> i32 {
        let lvl = (lvl + 1) as f64; // shift over by 1 so that getting to level 1 doesn't cost 0 xp
        ((0.5 * lvl.powf(2.0) - 0.5 * lvl) * 10.0) as i32
    }

    pub fn level_for_xp(xp : i32) -> i32 {
        for i in 0 .. 100 {
            if Skill::xp_required_for_level(i) > xp {
                return i - 1;
            }
        }
        100
    }
}

#[derive(Debug)]
pub struct SkillInfo {
    pub name : &'static str,
    pub skill_type : Skill
}

static SKILL_INFO : [SkillInfo ; 12] = [
    SkillInfo {
        name : "Dodge",
        skill_type : Skill::Dodge
    },
    SkillInfo {
        name : "Melee",
        skill_type : Skill::Melee
    },
    SkillInfo {
        name : "Ranged",
        skill_type : Skill::Ranged
    },
    SkillInfo {
        name : "Mountain Survival",
        skill_type : Skill::MountainSurvival
    },
    SkillInfo {
        name : "Forest Survival",
        skill_type : Skill::ForestSurvival
    },
    SkillInfo {
        name : "Fire Magic",
        skill_type : Skill::FireMagic
    },
    SkillInfo {
        name : "Ice Magic",
        skill_type : Skill::IceMagic
    },
    SkillInfo {
        name : "Farming",
        skill_type : Skill::Farming
    },
    SkillInfo {
        name : "Axe",
        skill_type : Skill::Axe
    },
    SkillInfo {
        name : "Spear",
        skill_type : Skill::Spear
    },
    SkillInfo {
        name : "Sword",
        skill_type : Skill::Sword
    },
    SkillInfo {
        name : "Mining",
        skill_type : Skill::Mining
    }
];

pub fn skill_info(for_skill : Skill) -> &'static SkillInfo {
    &SKILL_INFO[for_skill as usize]
}
