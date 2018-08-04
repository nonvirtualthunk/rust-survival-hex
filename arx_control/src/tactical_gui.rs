use common::prelude::*;
use tactical::TacticalMode;
use game::World;
use game::world::Entity;
use game::WorldView;
use std::sync::Mutex;
use game::entities::*;
use game::core::Reduceable;
use game::core::ReduceableType;
use game::core::GameEventClock;
use game::action_execution::movement::hexes_in_range;
use game::EntitySelector;
use game::EntitySelectors;
use game::AttackReference;
use std::ops;
use std::fmt;
use std;
use gui::*;
use gui::ToGUIUnit;
use common::Color;
use control_gui::*;
use game::action_types;
use game::ActionType;

use game::action_execution;
use common::prelude::*;
use arx_graphics::core::GraphicsWrapper;
use common::event_bus::EventBus;
use control_events::ControlEvents;
use common::event_bus::ConsumerHandle;
use arx_graphics::core::DrawList;
use arx_graphics::Quad;
use itertools::Itertools;
use common::AxialCoord;
use game::action_execution::movement;
use game::action_execution::combat;
use game::Oct;
use std::collections::HashMap;


#[derive(PartialEq)]
pub struct GameState {
    pub display_event_clock: GameEventClock,
    pub selected_character: Option<Entity>,
    pub victory: Option<bool>,
}

pub struct TacticalEventBundle<'a, 'b> {
    pub tactical: &'a mut TacticalMode,
    pub world: &'b mut World,
}

//
//#[derive(Default)]
//pub struct SimpleCache2<A : PartialEq + Clone,B : PartialEq + Clone,R> {
//    last_args : Option<(A,B)>,
//    last_ret : Option<R>
//}
//
//impl <A : PartialEq + Clone,B : PartialEq + Clone,R> SimpleCache2<A,B,R> {
//    pub fn new() -> SimpleCache2<A,B,R> {
//        SimpleCache2 {
//            last_args : None,
//            last_ret : None
//        }
//    }
//
//    pub fn call<'a,'b,'s,F : Fn(&A,&B) -> R>(&'s mut self, a : &'a A, b : &'b B, func : F) -> &'s R {
//        if let Some((arg_a, arg_b)) = self.last_args.as_ref() {
//            if (arg_a,arg_b) == (a,b) {
//                if self.last_ret.is_some() {
//                    return self.last_ret.as_ref().unwrap();
//                }
//            }
//        }
//        self.last_args = Some((a.clone(),b.clone()));
//        self.last_ret = Some((func)(a,b));
//        self.last_ret.as_ref().unwrap()
//    }
//}
//

pub struct TacticalGui {
    victory_widget : Widget,
    action_bar : ActionBar,
    event_bus : EventBus<ControlEvents>,
    event_bus_handle : ConsumerHandle,
    character_info_widget : CharacterInfoWidget,
    targeting_draw_list : DrawList,
    last_targeting_info : Option<(GameState, ActionType)>
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
            .with_child(Widget::text("Victory!", 30).centered())
            .apply(gui);


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
        }
    }

    pub fn draw(&mut self, view: & WorldView, game_state : GameState) -> DrawList {
        if let Some(selected) = game_state.selected_character {
            let cdata = view.character(selected);
            let action_type = self.action_bar.selected_action_for(selected);

            if let Some((last_state, last_action_type)) = self.last_targeting_info.as_ref() {
                if last_state == &game_state && last_action_type == action_type {
                    return self.targeting_draw_list.clone()
                }
            }

            let range = cdata.max_moves_remaining(1.0);
            let current_position = cdata.position;

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

//                    let hex_rings : Vec<HashMap<AxialCoord, f64>> = Vec::new();
//                    for i in 0 ..= max_possible_strikes {
//                        hex_rings.push(HashMap::new())
//                    }

                    let strikes_at_cost = |move_cost : &f64| -> i32 {
                        let additional_move_required = *move_cost - cur_move;
                        let additional_ap_required = (additional_move_required / move_speed).ceil() as i32;
                        (ap_remaining - additional_ap_required) / ap_per_strike as i32
                    };

                    for (hex,cost) in &hexes {
                        if hex != &current_position {
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

    fn draw_boundary_at_hex_range(&self, view : &WorldView, selected : Entity, current_position : AxialCoord, range : Oct, mut draw_list : DrawList, selector : &EntitySelector) -> DrawList {
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
            self.action_bar.update(gui, vec![action_types::MoveAndAttack, action_types::Move, action_types::Run], &game_state, ControlContext { event_bus : &mut self.event_bus });
        }

        for event in self.event_bus.events_for(&mut self.event_bus_handle) {
            match event {
                ControlEvents::ActionSelected(action_type) => {
                    println!("Selected action type : {:?}", action_type);
                }
            }
        }


        if let Some(victorious) = game_state.victory {
            self.victory_widget.set_showing(true);
        }
    }
}