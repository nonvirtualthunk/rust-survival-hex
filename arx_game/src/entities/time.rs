use world::Entity;
use world::EntityData;
use world::WorldView;

use world::ConstantModifier;

#[derive(Clone, Default, Debug)]
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
