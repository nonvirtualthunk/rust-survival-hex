use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use enum_map::EnumMap;
use std::collections::HashMap;
use game::entity;
use common::prelude::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Fields)]
pub struct SkillData {
    pub skill_bonuses: HashMap<Skill, i32>,
    pub skill_xp: HashMap<Skill, i32>,
}

impl EntityData for SkillData {}


pub trait SkillDataStore {
    fn skills(&self, ent: Entity) -> &SkillData;
}

impl SkillDataStore for WorldView {
    fn skills(&self, ent: Entity) -> &SkillData {
        self.data::<SkillData>(ent)
    }
}

impl SkillData {
    pub fn cur_skill_xp(&self, skill: Skill) -> i32 {
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
    Sentinel = 12,
}

impl Skill {
    pub fn name(&self) -> Str {
        match self {
            Skill::Dodge => "Dodge",
            Skill::Melee => "Melee",
            Skill::Ranged => "Ranged",
            Skill::MountainSurvival => "Mountain Survival",
            Skill::ForestSurvival => "Forest Survival",
            Skill::FireMagic => "Fire Magic",
            Skill::IceMagic => "Ice Magic",
            Skill::Farming => "Farming",
            Skill::Axe => "Axe",
            Skill::Spear => "Spear",
            Skill::Sword => "Sword",
            Skill::Mining => "Mining",
            Skill::Sentinel => "Sentinel",
        }
    }

    pub fn to_string_infinitive(&self) -> Str {
        self.name()
    }
}

impl Default for Skill {
    fn default() -> Self {
        Skill::Sentinel
    }
}