use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use enum_map::EnumMap;

#[derive(Clone, Debug, Default, PrintFields)]
pub struct SkillData {
    pub skill_bonuses: EnumMap<Skill, u32>,
    pub skill_xp : EnumMap<Skill, u32>
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
    pub fn skill_level(&self, skill : Skill) -> u32 {
        self.skill_bonuses[skill] + Skill::level_for_xp(self.skill_xp[skill])
    }
    pub fn cur_skill_xp(&self, skill : Skill) -> u32 {
        self.skill_xp[skill]
    }
    pub fn skill_xp_up(&mut self, skill : Skill, xp : u32) {
        self.skill_xp[skill] = self.skill_xp[skill] + xp;
    }

    pub fn skill_levels(&self) -> Vec<(Skill, u32)> {
        let mut res = Vec::new();
        for (skill,xp) in &self.skill_xp {
            res.push((skill, self.skill_level(skill)));
        }
        res
    }
}





#[derive(Enum, Debug, Clone, Copy, PartialEq)]
pub enum Skill {
    Dodge = 0,
    Melee = 1,
    Ranged = 2,
    MountainSurvival = 3,
    ForestSurvival = 4,
    FireMagic = 5,
    IceMagic = 6
}

impl Skill {
    pub fn xp_required_for_level(lvl : u32) -> u32 {
        let lvl = (lvl + 1) as f64; // shift over by 1 so that getting to level 1 doesn't cost 0 xp
        ((0.5 * lvl.powf(2.0) - 0.5 * lvl) * 10.0) as u32
    }

    pub fn level_for_xp(xp : u32) -> u32 {
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

static SKILL_INFO : [SkillInfo ; 7] = [
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
    }
];

pub fn skill_info(for_skill : Skill) -> &'static SkillInfo {
    &SKILL_INFO[for_skill as usize]
}
