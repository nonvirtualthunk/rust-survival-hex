use common::prelude::*;

use gui::GameState;
use gui::MouseButton;
use gui::PlayerActionType;
use gui::GUI;

use game::prelude::*;

use graphics::GraphicsResources;
use graphics::prelude::*;



pub(crate) trait PlayerActionHandler {
    fn handle_click(&mut self, world : &mut World, game_state : &GameState, player_action: &PlayerActionType, button : MouseButton) -> bool;

    fn draw(&mut self, world_view : &WorldView, game_state : &GameState, player_action: &PlayerActionType) -> DrawList;

    fn update_widgets(&mut self, gui : &mut GUI, grsrc : &mut GraphicsResources, world: &World, world_view : &WorldView, game_state : &GameState, player_action : &PlayerActionType);

    fn hide_widgets(&mut self, gui : &mut GUI);
}