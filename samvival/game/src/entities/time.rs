use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;

use game::modifiers::ConstantModifier;
use common::reflect::*;
use std::collections::HashMap;

#[derive(Clone, Default, Debug, PrintFields)]
pub struct TurnData {
    pub turn_number : u32,
    pub active_faction : Entity
}
impl EntityData for TurnData {}

pub struct SetTurnNumberMod(pub u32);
impl ConstantModifier<TurnData> for SetTurnNumberMod{
    fn modify(&self, data: &mut TurnData) {
        data.turn_number = self.0;
    }
}

pub struct SetActiveFactionMod(pub Entity);
impl ConstantModifier<TurnData> for SetActiveFactionMod{
    fn modify(&self, data: &mut TurnData) {
        data.active_faction = self.0;
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TimeOfDay {
    Dawn,
    Daylight,
    Dusk,
    Night
}
impl Default for TimeOfDay {
    fn default() -> Self {
        TimeOfDay::Daylight
    }
}

#[derive(Clone, Debug, PrintFields)]
pub struct TimeData {
    pub moments_since_world_start : u32,
    pub moments_by_time_of_day : HashMap<TimeOfDay, u32>,
    pub moments_since_day_start : u32
}

impl Default for TimeData {
    fn default() -> Self {
        let mut durations = HashMap::new();
        durations.insert(TimeOfDay::Dawn, 2);
        durations.insert(TimeOfDay::Dusk, 2);
        durations.insert(TimeOfDay::Daylight, 6);
        durations.insert(TimeOfDay::Night, 6);

        TimeData {
            moments_since_world_start : 0,
            moments_since_day_start : 0,
            moments_by_time_of_day : durations
        }
    }
}

impl EntityData for TimeData {}