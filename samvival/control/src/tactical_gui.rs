use common::prelude::*;
use tactical::TacticalMode;
use game::World;
use game::Entity;
use game::WorldView;
use std::sync::Mutex;
use game::entities::*;
use game::core::Reduceable;
use game::core::ReduceableType;
use game::core::GameEventClock;
use game::logic::movement::hexes_in_range;
use game::actions::EntitySelector;
use game::actions::EntitySelectors;
use game::entities::AttackReference;
use game::logic::factions::is_enemy;
use std::ops;
use std::fmt;
use std;
use gui::*;
use gui::ToGUIUnit;
use common::Color;
use control_gui::*;
use game::action_types;
use game::ActionType;

use game::logic;
use common::prelude::*;
use graphics::core::GraphicsWrapper;
use common::event_bus::EventBus;
use control_events::ControlEvents;
use common::event_bus::ConsumerHandle;
use graphics::core::DrawList;
use graphics::Quad;
use itertools::Itertools;
use common::AxialCoord;
use game::logic::movement;
use game::logic::combat;
use game::Sext;
use std::collections::HashMap;
use noisy_float::types::r32;
use gui::WidgetContainer;
use control_gui::attack_descriptions::AttackDetailsWidget;


#[derive(PartialEq)]
pub struct GameState {
    pub display_event_clock: GameEventClock,
    pub selected_character: Option<Entity>,
    pub victory: Option<bool>,
    pub player_faction: Entity,
    pub hovered_hex_coord: AxialCoord,
    pub animating: bool,
    pub mouse_pixel_pos: Vec2f,
    pub mouse_game_pos: Vec2f
}

pub struct TacticalEventBundle<'a, 'b> {
    pub tactical: &'a mut TacticalMode,
    pub world: &'b mut World,
}

pub struct TacticalGui {
    victory_widget : Widget,
    action_bar : ActionBar,
    event_bus : EventBus<ControlEvents>,
    event_bus_handle : ConsumerHandle,
    character_info_widget : CharacterInfoWidget,
    targeting_draw_list : DrawList,
    last_targeting_info : Option<(GameState, ActionType)>,
    attack_details_widget : AttackDetailsWidget
}


pub struct ControlContext<'a> {
    pub event_bus : &'a mut EventBus<ControlEvents>
}


impl TacticalGui {
    pub fn new(gui: &mut GUI) -> TacticalGui {
        let victory_widget = Widget::window(Color::greyscale(0.9), 2)
            .size(Sizing::ux(50.0), Sizing::ux(30.0))
            .position(Positioning::centered(), Positioning::centered())
            .showing(false)
            .apply(gui);

        let victory_text = Widget::text("Victory!", 30).centered().parent(&victory_widget).apply(gui);


        let event_bus = EventBus::new();
        let event_bus_handle = event_bus.register_consumer(true);

        TacticalGui {
            victory_widget,
            action_bar : ActionBar::new(gui),
            event_bus,
            event_bus_handle,
            character_info_widget : CharacterInfoWidget::new(gui),
            targeting_draw_list : DrawList::none(),
            last_targeting_info : None,
            attack_details_widget : AttackDetailsWidget::new().draw_layer_for_all(GUILayer::Overlay)
        }
    }

    pub fn draw(&mut self, view: & WorldView, game_state : GameState) -> DrawList {
        if let Some(selected) = game_state.selected_character {
            let cdata = view.character(selected);
            // if it's not the player's turn, don't display UI
            if view.world_data::<TurnData>().active_faction != cdata.faction {
                return DrawList::none();
            }

            let action_type = self.action_bar.selected_action_for(selected);

            if let Some((last_state, last_action_type)) = self.last_targeting_info.as_ref() {
                if last_state == &game_state && last_action_type == action_type {
                    return self.targeting_draw_list.clone()
                }
            }

            let range = cdata.max_moves_remaining(1.0);
            let current_position = cdata.position.hex;

            let mut draw_list = DrawList::none();

            if action_type == &action_types::MoveAndAttack {
                let main_attack = combat::primary_attack(view, selected);
                if let Some(attack) = main_attack {
                    let hexes = hexes_in_range(view, selected, cdata.max_moves_remaining(1.0));

                    let ap_per_strike = attack.ap_cost;
                    let ap_remaining = cdata.action_points.cur_value();
                    let cur_move = cdata.moves.as_f64();
                    let move_speed = cdata.move_speed.as_f64();
                    let max_possible_strikes = ap_remaining / ap_per_strike as i32;

                    let strikes_at_cost = |move_cost : &f64| -> i32 {
                        let additional_move_required = *move_cost - cur_move;
                        let additional_ap_required = (additional_move_required / move_speed).ceil() as i32;
                        (ap_remaining - additional_ap_required) / ap_per_strike as i32
                    };

                    for (hex,cost) in &hexes {
                        if hex != &current_position{
                            if let Some(tile) = view.entity_by_key(hex) {
                                let strikes_in_this_tile = strikes_at_cost(cost);
                                let neighbors = hex.neighbors();
                                for q in 0 .. 6 {
                                    if let Some(neighbor_cost) = hexes.get(&neighbors[q]) {
                                        let neighbor_strikes = strikes_at_cost(neighbor_cost);
                                        if neighbor_strikes < strikes_in_this_tile {
                                            draw_list = draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(Color::new(0.2,0.6,0.2,0.35)).centered());
                                        }
                                    } else {
                                        draw_list = draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(Color::new(0.4,0.4,0.4,0.35)).centered());
                                    }
                                }
                            }
                        }
                    }

                    if attack.range > 1 {
                        for (entity, cdata) in view.entities_with_data::<CharacterData>() {
                            let character = view.character(*entity);
                            if cdata.is_alive() && character.position.hex.distance(&current_position) < r32(attack.range as f32) {
                                if is_enemy(view, *entity, selected) {
                                    let hex = character.position.hex;
                                    for q in 0 .. 6 {
                                        draw_list = draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(Color::new(0.7,0.1,0.1,0.75)).centered());
                                    }
                                }
                            }
                        }
                    }
                } else {
                    warn!("No attack possible, whatsoever, for entity {}", selected);
                }
            } else {
                let entity_selectors = (action_type.target)(selected, view);
                for selector in entity_selectors {
                    let EntitySelector(pieces) = selector;
                    if pieces.contains(&EntitySelectors::IsTile) {
                        let range_limiter = pieces.iter().find(|s| { if let EntitySelectors::InMoveRange {..} = s { true } else { false } });
                        if let Some(range_limit) = range_limiter {
                            // trim the selector down to remove the range limiter, we've pre-emptively taken care of that
                            let selector = EntitySelector(pieces.iter().cloned().filter(|s| if let EntitySelectors::InMoveRange {..} = s { false } else { true } ).collect_vec());

                            draw_list = self.draw_boundary_at_hex_range(view, selected, current_position, range, draw_list, &selector);
                        } else {
                            warn!("Tile selector without range limiter, this should not generally be the case")
                        }
                    }
                }
            }

            self.last_targeting_info = Some((game_state, action_type.clone()));
            self.targeting_draw_list = draw_list.clone();

            draw_list
        } else {
            DrawList::none()
        }
    }

    fn draw_boundary_at_hex_range(&self, view : &WorldView, selected : Entity, current_position : AxialCoord, range : Sext, mut draw_list : DrawList, selector : &EntitySelector) -> DrawList {
        let hexes = hexes_in_range(view, selected, range);
        for (hex,cost) in &hexes {
            if hex != &current_position {
                if let Some(tile) = view.entity_by_key(hex) {
                    if selector.matches(tile, view) {
                        let neighbors = hex.neighbors();
                        for q in 0 .. 6 {
                            if !hexes.contains_key(&neighbors[q]) {
                                draw_list = draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(Color::new(0.1,0.6,0.1,0.6)).centered());
                            }
                        }
                    }
                }
            }
        }

        draw_list
    }


    pub fn update_gui(&mut self, world: &mut World, gui: &mut GUI, frame_id: Option<Wid>, game_state: GameState) {
        if let Some(selected) = game_state.selected_character {
            let world_view = world.view_at_time(game_state.display_event_clock);

            self.character_info_widget.update(&world_view, gui, &game_state);
            if world_view.character(selected).faction == game_state.player_faction {
                self.action_bar.update(gui, vec![action_types::MoveAndAttack, action_types::Move, action_types::Run], &game_state, ControlContext { event_bus : &mut self.event_bus });
            } else {
                self.action_bar.set_showing(false).as_widget().reapply(gui);
            }

            self.update_attack_details(world, &world_view, gui, &game_state, selected);
        } else {
            self.character_info_widget.set_showing(false).as_widget().reapply(gui);
            self.action_bar.set_showing(false).as_widget().reapply(gui);
            self.attack_details_widget.hide(gui);
        }

        for event in self.event_bus.events_for(&mut self.event_bus_handle) {
            match event {
                ControlEvents::ActionSelected(action_type) => {
                    println!("Selected action type : {:?}", action_type);
                }
            }
        }



        if let Some(victorious) = game_state.victory {
            self.victory_widget.set_showing(true).reapply(gui);
        }
    }

    pub fn update_attack_details(&mut self, world: &World, view : &WorldView, gui : &mut GUI, game_state : &GameState, selected : Entity) {
        use game::logic::*;
        if let Some(hovered_char) = view.tile_opt(game_state.hovered_hex_coord).and_then(|e| e.occupied_by) {
            if ! game_state.animating && factions::is_enemy(view, selected, hovered_char) {
                if let Some(attack) = combat::primary_attack(view, selected) {
                    self.attack_details_widget.set_position(Positioning::constant((game_state.mouse_pixel_pos.x + 20.0).px()), Positioning::constant((game_state.mouse_pixel_pos.y + 20.0).px()));
                    self.attack_details_widget.set_showing(true);
                    self.attack_details_widget.update(gui, world, view, selected, hovered_char, &attack);
                }
                return;
            }
        }
        self.attack_details_widget.hide(gui);
    }
}