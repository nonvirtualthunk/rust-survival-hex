use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;

//use game::modifiers::ConstantModifier;
use common::reflect::*;
use std::collections::HashMap;
use game::prelude::*;

#[derive(Clone, Default, Debug, Serialize, Deserialize, Fields)]
pub struct TurnData {
    pub turn_number : u32,
    pub active_faction : Entity
}
impl EntityData for TurnData {}

//pub struct SetTurnNumberMod(pub u32);
//impl ConstantModifier<TurnData> for SetTurnNumberMod{
//    fn modify(&self, data: &mut TurnData) {
//        data.turn_number = self.0;
//    }
//}
//
//pub struct SetActiveFactionMod(pub Entity);
//impl ConstantModifier<TurnData> for SetActiveFactionMod{
//    fn modify(&self, data: &mut TurnData) {
//        data.active_faction = self.0;
//    }
//}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}
impl Default for Season {
    fn default() -> Self {
        Season::Spring
    }
}


#[derive(Clone, Debug, Fields, Serialize, Deserialize)]
pub struct TimeData {
    pub moments_since_world_start : u32,
    pub moments_per_time_of_day: HashMap<TimeOfDay, u32>,
    pub days_per_season: HashMap<Season, u32>,
    pub moments_since_day_start : u32
}

impl TimeData {

}

impl Default for TimeData {
    fn default() -> Self {
        let mut durations = HashMap::new();
        durations.insert(TimeOfDay::Dawn, 2);
        durations.insert(TimeOfDay::Dusk, 2);
        durations.insert(TimeOfDay::Daylight, 6);
        durations.insert(TimeOfDay::Night, 6);

        let days_per_season = [
            (Season::Spring, 6u32),
            (Season::Summer, 6u32),
            (Season::Autumn, 6u32),
            (Season::Winter, 6u32)
        ].iter().cloned().collect();

        TimeData {
            moments_since_world_start : 0,
            moments_since_day_start : 0,
            moments_per_time_of_day: durations,
            days_per_season
        }
    }
}

impl EntityData for TimeData {}