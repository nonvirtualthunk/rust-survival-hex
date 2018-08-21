use common::prelude::*;
use control_events::ControlEvents;
use common::hex::*;
use game::prelude::*;
use common::EventBus;

#[derive(PartialEq)]
pub struct GameState {
    pub display_event_clock: GameEventClock,
    pub selected_character: Option<Entity>,
    pub victory_time: Option<GameEventClock>,
    pub defeat_time: Option<GameEventClock>,
    pub player_faction: Entity,
    pub hovered_hex_coord: AxialCoord,
    pub animating: bool,
    pub mouse_pixel_pos: Vec2f,
    pub mouse_game_pos: Vec2f
}



pub struct ControlContext<'a> {
    pub event_bus : &'a mut EventBus<ControlEvents>
}

impl <'a> ControlContext<'a> {
    pub fn trigger_event(&mut self, evt : ControlEvents) {
        self.event_bus.push_event(evt);
    }
}