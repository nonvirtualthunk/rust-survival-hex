use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;

use game::modifiers::ConstantModifier;
use common::reflect::*;

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
