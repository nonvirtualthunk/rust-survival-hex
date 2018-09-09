use common::prelude::*;

use gui::GameState;
use gui::MouseButton;
use gui::PlayerActionType;
use gui::GUI;



pub(crate) trait PlayerActionHandler {
    fn on_click(&mut self, world : &mut World, game_state : &GameState, player_action: &PlayerActionType, mouse_game_pos : Vec2f, button : MouseButton);

    fn draw(&mut self, world_view : &WorldView, game_state : &GameState, player_action: &PlayerActionType) -> DrawList;

    fn update_widgets(&mut self, gui : &mut GUI, grsrc : &mut GraphicsResources, world: &World, world_view : &WorldView, game_state : &GameState, player_action : &PlayerActionType);
}