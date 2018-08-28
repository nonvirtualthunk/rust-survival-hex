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
use game::entities::AttackRef;
use game::logic::faction::is_enemy;
use std::ops;
use std::fmt;
use std;
use gui::*;
use gui::ToGUIUnit;
use common::Color;
use game::entities::actions::*;
use game::entities::reactions::reaction_types;

use game::logic;
use common::prelude::*;
use graphics::core::GraphicsWrapper;
use common::event_bus::EventBus;
use gui::control_events::ControlEvents;
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
use gui::attack_descriptions::AttackDetailsWidget;
use game::reflect::*;
use gui::messages_widget::MessagesDisplay;
use gui::messages_widget::Message;
use gui::inventory_widget::*;
use game::entities::inventory::InventoryData;
use action_ui_handlers::*;


pub struct TacticalEventBundle<'a, 'b> {
    pub tactical: &'a mut TacticalMode,
    pub world: &'b mut World,
}

#[derive(PartialEq,Clone,Copy)]
pub enum AuxiliaryWindows {
    Inventory
}

pub struct TacticalGui {
    main_area : Widget,
    victory_widget : Widget,
    defeat_widget : Widget,
    action_bar : ActionBar,
    reaction_bar : ReactionBar,
    event_bus : EventBus<ControlEvents>,
    event_bus_handle : ConsumerHandle,
    character_info_widget : CharacterInfoWidget,
    targeting_draw_list : DrawList,
    last_targeting_info : Option<(GameState, PlayerActionType)>,
    attack_details_widget : AttackDetailsWidget,
    messages_display : MessagesDisplay,
    inventory_widget: inventory_widget::InventoryDisplay,
    open_auxiliary_windows : Vec<AuxiliaryWindows>,

}


impl TacticalGui {
    pub fn new(gui: &mut GUI) -> TacticalGui {
        let main_area = Widget::div()
            .size(Sizing::DeltaOfParent(0.ux()), Sizing::DeltaOfParent(0.ux()))
            .position(Positioning::origin(), Positioning::origin())
            .named("main tactical gui area")
            .apply(gui);

        let victory_widget = Widget::window(Color::greyscale(0.9), 2)
            .size(Sizing::ux(50.0), Sizing::ux(30.0))
            .position(Positioning::centered(), Positioning::centered())
            .showing(false)
            .parent(&main_area)
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
            action_bar : ActionBar::new(gui, &main_area),
            reaction_bar : ReactionBar::new(gui, &main_area),
            targeting_draw_list : DrawList::none(),
            last_targeting_info : None,
            attack_details_widget : AttackDetailsWidget::new().draw_layer_for_all(GUILayer::Overlay),
            messages_display : MessagesDisplay::new(gui, &main_area),
            inventory_widget : inventory_widget::InventoryDisplay::new(strf("Character Inventory"), &main_area),
            main_area,
            open_auxiliary_windows: Vec::new()
        }
    }


    pub fn draw_selected_character_overlay(&mut self, view: & WorldView, game_state : &GameState) -> DrawList {
        if let Some(selected) = game_state.selected_character {
            let cdata = view.character(selected);
            // if it's not the player's turn, don't display UI
            if view.world_data::<TurnData>().active_faction != cdata.allegiance.faction {
                return DrawList::none();
            }

            let action_type = self.action_bar.selected_action_for(view, selected);

            if let Some((last_state, last_action_type)) = self.last_targeting_info.as_ref() {
                if last_state == game_state && last_action_type == &action_type {
                    return self.targeting_draw_list.clone()
                }
            }

            let current_position = cdata.position.hex;

            let visibility = view.world_data::<VisibilityData>().visibility_for(game_state.player_faction);


            let draw_list = if let PlayerActionType::MoveAndAttack(_,attack_ref) = &action_type {
                move_and_attack_handler::draw_move_and_attack_overlay(view, &game_state, attack_ref.clone())
            } else {
                DrawList::none()
//                let entity_selectors = (action_type.target)(selected, view);
//                for selector in entity_selectors {
//                    let EntitySelector(pieces) = selector;
//                    if pieces.contains(&EntitySelectors::IsTile) {
//                        let range_limiter = pieces.iter().find(|s| { if let EntitySelectors::InMoveRange {..} = s { true } else { false } });
//                        if let Some(range_limit) = range_limiter {
//                            // trim the selector down to remove the range limiter, we've pre-emptively taken care of that
//                            let selector = EntitySelector(pieces.iter().cloned().filter(|s| if let EntitySelectors::InMoveRange {..} = s { false } else { true } ).collect_vec());
//
//                            draw_list = self.draw_boundary_at_hex_range(view, selected, current_position, range, draw_list, &selector);
//                        } else {
//                            warn!("Tile selector without range limiter, this should not generally be the case")
//                        }
//                    }
//                }
            };

            self.last_targeting_info = Some((game_state.clone(), action_type.clone()));
            self.targeting_draw_list = draw_list.clone();

            draw_list
        } else {
            DrawList::none()
        }
    }

    pub fn draw(&mut self, view: & WorldView, game_state : GameState) -> DrawList {
        let main_draw_list = self.draw_selected_character_overlay(view, &game_state);

        let draw_list = main_draw_list.with_quad(Quad::new_cart(String::from("ui/hoverHex"), game_state.hovered_hex_coord.as_cart_vec()).centered());

        draw_list
    }

    pub fn toggle_inventory(&mut self, gui : &mut GUI) {
        if self.open_auxiliary_windows.contains(&AuxiliaryWindows::Inventory) {
            self.inventory_widget.set_showing(false).reapply(gui);
            self.open_auxiliary_windows.retain(|window| window != &AuxiliaryWindows::Inventory);
        } else {
            self.inventory_widget.set_showing(true).reapply(gui);
            self.open_auxiliary_windows.push(AuxiliaryWindows::Inventory);
        }
    }

    pub fn close_all_auxiliary_windows(&mut self, gui : &mut GUI) -> bool {
        if self.open_auxiliary_windows.non_empty() {
            for window in &self.open_auxiliary_windows {
                match window {
                    AuxiliaryWindows::Inventory => self.inventory_widget.set_showing(false).reapply(gui)
                }
            }
            self.open_auxiliary_windows.clear();
            true
        } else {
            false
        }
    }

//    fn draw_boundary_at_hex_range(&self, view : &WorldView, selected : Entity, current_position : AxialCoord, range : Sext, mut draw_list : DrawList, selector : &EntitySelectors) -> DrawList {
//        let hexes = hexes_in_range(view, selected, range);
//        for (hex,cost) in &hexes {
//            if hex != &current_position {
//                if let Some(tile) = view.entity_by_key(hex) {
//                    if selector.matches(tile, view) {
//                        let neighbors = hex.neighbors_vec();
//                        for q in 0 .. 6 {
//                            if !hexes.contains_key(&neighbors[q]) {
//                                draw_list = draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(Color::new(0.1,0.6,0.1,0.6)).centered());
//                            }
//                        }
//                    }
//                }
//            }
//        }
//
//        draw_list
//    }


    pub fn update_gui(&mut self, world: &mut World, world_view : &WorldView, gui: &mut GUI, frame_id: Option<Wid>, game_state: GameState) {
//        let world_view = world.view_at_time(game_state.display_event_clock);

        self.messages_display.update(gui);

        if let Some(selected) = game_state.selected_character {
            let mut control = ControlContext { event_bus : &mut self.event_bus };

            self.main_area.set_width(Sizing::DeltaOfParent(-40.ux())).reapply(gui);
            self.character_info_widget.update(&world_view, gui, &game_state, &mut control);
            let char = world_view.character(selected);

            if char.allegiance.faction == game_state.player_faction {
                let mut actions = Vec::new();
                if let Some(attack_ref) = combat::primary_attack_ref(world_view, selected) {
                    if let Some(move_ref) = movement::default_movement_type(world_view, selected) {
                        actions.push(PlayerActionType::MoveAndAttack(move_ref, attack_ref));
                    }
                }
                actions.push(PlayerActionType::InteractWithInventory);
                actions.push(PlayerActionType::Wait);

                self.action_bar.update(gui, world_view, actions, &game_state, &mut control);
//                self.reaction_bar.set_x(Positioning::left_of(self.character_info_widget.as_widget(), 1.ux())).as_widget().reapply(gui);
//                self.reaction_bar.set_y(Positioning::constant(2.ux())).as_widget().reapply(gui);
                let reactions = vec![reaction_types::Defend, reaction_types::Dodge, reaction_types::Block, reaction_types::Counterattack];
                self.reaction_bar.update(gui, reactions, char.action.active_reaction.clone(), &game_state, &mut control);
            } else {
                self.action_bar.set_showing(false).reapply(gui);
                self.reaction_bar.set_showing(false).reapply(gui);
            }

            if let PlayerActionType::MoveAndAttack(_, attack_ref) = self.action_bar.selected_action_for(&world_view, selected) {
                update_move_and_attack_widgets(&mut self.attack_details_widget, world, &world_view, gui, &game_state, selected, attack_ref);
            }


            let inv_data = world_view.data::<InventoryData>(selected);
            let items = &inv_data.items;
            let main_inv = vec![InventoryDisplayData::new(items.clone(), "Character Inventory", vec![selected], true, inv_data.inventory_size)];

            let mut ground_items = Vec::new();
            let mut ground_entities = Vec::new();
            for ground_coord in vec![char.position.hex].extended_by(char.position.hex.neighbors_vec()) {
                if let Some(ent) = world_view.tile_ent_opt(ground_coord) {
                    ground_entities.push(ent.entity);
                    if let Some(inv) = world_view.data_opt::<InventoryData>(ent.entity) {
                        ground_items.extend(inv.items.clone());
                    }
                }
            }
            let other_inv = vec![InventoryDisplayData::new(ground_items, "Ground", ground_entities, false, None)];
            self.inventory_widget.update(gui, world, main_inv, other_inv, &mut control);
        } else {
            self.main_area.set_width(Sizing::DeltaOfParent(0.ux())).reapply(gui);

            self.character_info_widget.set_showing(false).reapply(gui);
            self.action_bar.set_showing(false).reapply(gui);
            self.reaction_bar.set_showing(false).reapply(gui);
            self.attack_details_widget.hide(gui);
            self.inventory_widget.set_showing(false).reapply(gui);
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
                        world.add_event(GameEvent::SelectedAttackChanged { entity : selected, attack_ref : attack_ref.clone() });
                    },
                    ControlEvents::CounterattackSelected(attack_ref) => {
                        println!("Counter selected");
                        if logic::combat::is_valid_counter_attack(world, selected, attack_ref) {
                            world.modify(selected, CombatData::active_counterattack.set_to(attack_ref.clone()), "counter-attack selected");
                            world.add_event(GameEvent::SelectedCounterattackChanged { entity : selected, attack_ref : attack_ref.clone() });
                        } else {
                            self.messages_display.add_message(Message::new("Only melee attacks can be used as counter-attacks, reach and ranged cannot."));
                        }
                    },
                    ControlEvents::ReactionSelected(reaction_type) => {
                        println!("Selected reaction type : {:?}", reaction_type);
                        world.modify(selected, ActionData::active_reaction.set_to(reaction_type.clone()), "reaction selected");
                        world.add_event(GameEvent::SelectedReactionChanged { entity : selected, reaction_type : reaction_type.clone() });
                    },
                    ControlEvents::ItemTransferRequested { item , from, to } => {
                        if let Some(from) = from.find(|ent| world_view.data_opt::<InventoryData>(*ent).map(|inv| inv.items.contains(item)).unwrap_or(false)) {
                            let from_pos = world_view.data_opt::<PositionData>(*from).map(|pd| pd.hex).unwrap_or(AxialCoord::new(0,0));
                            // if these are ground tiles, pick the closest one
                            let single_to = if to.iter().all(|to| world_view.has_data::<TileData>(*to)) {
                                to.iter().min_by_key(|to| world_view.data::<TileData>(**to).position.distance(&from_pos))
                            } else {
                                to.first()
                            };
                            if let Some(single_to) = single_to {
                                if logic::item::is_item_in_inventory_of(world, *item, *from) {
                                    if logic::item::is_item_equipped_by(world, *item, *from) {
                                        logic::item::unequip_item(world, *item, *from, true);
                                    }
                                    logic::item::remove_item_from_inventory(world, *item, *from);
                                    logic::item::put_item_in_inventory(world, *item, *single_to);
                                } else {
                                    error!("Could not transfer, item no longer in source");
                                }
                            }
                        } else {
                            error!("none of the from entities actually held the desired item")
                        }
                    },
                    ControlEvents::EquipItemRequested { item, equip_on } => {
                        if ! logic::item::is_item_equipped_by(world, *item, *equip_on) {
                            logic::item::equip_item(world, *item, *equip_on, true);
                        } else {
                            logic::item::unequip_item(world, *item, *equip_on, true);
                        }
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
}