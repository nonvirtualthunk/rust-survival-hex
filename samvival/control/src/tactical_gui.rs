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
use game::GameEvent;
use game::logic::movement::hexes_in_range;
use game::entities::AttackReference;
use game::logic::faction::is_enemy;
use std::ops;
use std::fmt;
use std;
use gui::*;
use gui::ToGUIUnit;
use common::Color;
use control_gui::*;
use game::entities::actions::*;
use game::entities::reactions::reaction_types;

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
use game::reflect::*;


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

pub struct TacticalEventBundle<'a, 'b> {
    pub tactical: &'a mut TacticalMode,
    pub world: &'b mut World,
}

pub struct TacticalGui {
    victory_widget : Widget,
    defeat_widget : Widget,
    action_bar : ActionBar,
    reaction_bar : ReactionBar,
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

        let defeat_widget = Widget::window(Color::greyscale(0.9), 2)
            .size(Sizing::ux(50.0), Sizing::ux(30.0))
            .position(Positioning::centered(), Positioning::centered())
            .showing(false)
            .apply(gui);

        let defeat_text = Widget::text("Defeat!", 30).color(Color::new(0.6,0.1,0.1,1.0)).centered().parent(&defeat_widget).apply(gui);


        let event_bus = EventBus::new();
        let event_bus_handle = event_bus.register_consumer(true);

        TacticalGui {
            victory_widget,
            defeat_widget,
            event_bus,
            event_bus_handle,
            character_info_widget : CharacterInfoWidget::new(gui),
            action_bar : ActionBar::new(gui),
            reaction_bar : ReactionBar::new(gui),
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


                    let strike_count_colors = [Color::new(0.2, 0.6, 0.2, 0.35), Color::new(0.25, 0.5, 0.15, 0.35), Color::new(0.3, 0.45, 0.1, 0.35), Color::new(0.35,0.35,0.1,0.35)];

                    let color_for_strike_count = |sc : i32| if sc == 0 { Color::new(0.4,0.4,0.4,0.35) } else { strike_count_colors[(max_possible_strikes - sc) as usize] };
                    for (hex,cost) in &hexes {
                        if hex != &current_position{
                            if let Some(tile) = view.entity_by_key(hex) {
                                let strikes_in_this_tile = strikes_at_cost(cost);
                                let neighbors = hex.neighbors_vec();
                                for q in 0 .. 6 {
                                    let mut draw_color = None;
                                    if let Some(neighbor_cost) = hexes.get(&neighbors[q]) {
                                        let neighbor_strikes = strikes_at_cost(neighbor_cost);
                                        if neighbor_strikes < strikes_in_this_tile {
                                            draw_color = Some(color_for_strike_count(strikes_in_this_tile));
                                        }
                                    } else if view.tile_opt(neighbors[q]).map(|h| h.occupied_by.is_none()).unwrap_or(true) || strikes_in_this_tile > 0 {
                                        draw_color = Some(color_for_strike_count(strikes_in_this_tile));
                                    }

                                    if let Some(color) = draw_color {
                                        draw_list = draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(color).centered());
                                    }
                                }
                            }
                        }
                    }

                    for (entity, cdata) in view.entities_with_data::<CharacterData>() {
                        let character = view.character(*entity);
                        if logic::combat::within_range(view, selected, *entity, &attack, None, None)  && logic::combat::is_valid_attack_target(view, selected, *entity, &attack) {
                            if is_enemy(view, *entity, selected) {
                                let hex = character.position.hex;
                                for q in 0 .. 6 {
                                    draw_list = draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(Color::new(0.7,0.1,0.1,0.75)).centered());
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
                        let neighbors = hex.neighbors_vec();
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
        let world_view = world.view_at_time(game_state.display_event_clock);

        if let Some(selected) = game_state.selected_character {
            self.character_info_widget.update(&world_view, gui, &game_state, ControlContext { event_bus : &mut self.event_bus });
            let char = world_view.character(selected);

            if char.faction == game_state.player_faction {
                self.action_bar.update(gui, vec![action_types::MoveAndAttack, action_types::Move, action_types::Run], &game_state, ControlContext { event_bus : &mut self.event_bus });
                self.reaction_bar.set_x(Positioning::left_of(self.character_info_widget.as_widget(), 1.ux())).as_widget().reapply(gui);
//                self.reaction_bar.set_y(Positioning::constant(2.ux())).as_widget().reapply(gui);
                self.reaction_bar.update(gui, vec![reaction_types::Defend, reaction_types::Dodge, reaction_types::Block, reaction_types::Counterattack], char.action.active_reaction.clone(), &game_state, ControlContext { event_bus : &mut self.event_bus });
            } else {
                self.action_bar.set_showing(false).as_widget().reapply(gui);
                self.reaction_bar.set_showing(false).as_widget().reapply(gui);
            }

            self.update_attack_details(world, &world_view, gui, &game_state, selected);
        } else {
            self.character_info_widget.set_showing(false).as_widget().reapply(gui);
            self.action_bar.set_showing(false).as_widget().reapply(gui);
            self.reaction_bar.set_showing(false).as_widget().reapply(gui);
            self.attack_details_widget.hide(gui);
        }

        for event in self.event_bus.events_for(&mut self.event_bus_handle) {
            if let Some(selected) = game_state.selected_character {
                match event {
                    ControlEvents::ActionSelected(action_type) => {
                        println!("Selected action type : {:?}", action_type);
                    },
                    ControlEvents::AttackSelected(attack_ref) => {
                        println!("Attack selected");
                        world.modify(selected, CombatData::active_attack.set_to(attack_ref.clone()), "attack selected");
                        world.add_event(GameEvent::SelectedAttackChanged { entity : selected, attack_ref : attack_ref.clone() })
                    },
                    ControlEvents::ReactionSelected(reaction_type) => {
                        println!("Selected reaction type : {:?}", reaction_type);
                        world.modify(selected, ActionData::active_reaction.set_to(reaction_type.clone()), "reaction selected");
                        world.add_event(GameEvent::SelectedReactionChanged { entity : selected, reaction_type : reaction_type.clone() });
                    }
                }
            }
        }



        if let Some(victorious_time) = game_state.victory_time {
            if victorious_time <= game_state.display_event_clock {
                self.victory_widget.set_showing(true).reapply(gui);
            }
        } else if let Some(defeat_time) = game_state.defeat_time {
            if defeat_time <= game_state.display_event_clock {
                self.defeat_widget.set_showing(true).reapply(gui);
            }
        }
    }

    pub fn update_attack_details(&mut self, world: &World, view : &WorldView, gui : &mut GUI, game_state : &GameState, selected : Entity) {
        use game::logic::*;
        if let Some(hovered_char) = view.tile_opt(game_state.hovered_hex_coord).and_then(|e| e.occupied_by) {
            if ! game_state.animating && faction::is_enemy(view, selected, hovered_char) {
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