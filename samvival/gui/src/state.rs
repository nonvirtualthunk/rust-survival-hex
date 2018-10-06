use common::prelude::*;
use control_events::TacticalEvents;
use common::hex::*;
use game::prelude::*;
use common::EventBus;
use vecmath::Matrix2x3;
use piston_window::Viewport;

#[derive(Clone)]
pub struct GameState {
    pub display_event_clock: GameEventClock,
    pub selected_character: Option<Entity>,
    pub victory_time: Option<GameEventClock>,
    pub defeat_time: Option<GameEventClock>,
    pub player_faction: Entity,
    pub hovered_hex_coord: AxialCoord,
    pub animating: bool,
    pub player_faction_active: bool,
    pub mouse_pixel_pos: Vec2f,
    pub mouse_game_pos: Vec2f,
    pub mouse_cart_vec: CartVec,
    pub view_matrix: Matrix2x3<f64>,
    pub viewport: Viewport,
}
impl GameState {
    pub fn mouse_cart_vec(&self) -> CartVec { self.mouse_cart_vec }

    pub fn key_state(&self) -> KeyGameState {
        KeyGameState {
            display_event_clock : self.display_event_clock,
            selected_character : self.selected_character,
            hovered_hex_coord : self.hovered_hex_coord,
            animating : self.animating,
        }
    }
}

/// game state suitable for use as a key or for caching, only includes relevant, not-quickly-changing fields
#[derive(PartialEq, Clone)]
pub struct KeyGameState {
    pub display_event_clock: GameEventClock,
    pub selected_character: Option<Entity>,
    pub hovered_hex_coord: AxialCoord,
    pub animating: bool,
}


pub struct ControlContext<'a> {
    pub event_bus : &'a mut EventBus<TacticalEvents>
}

impl <'a> ControlContext<'a> {
    pub fn trigger_event(&mut self, evt : TacticalEvents) {
        self.event_bus.push_event(evt);
    }
}