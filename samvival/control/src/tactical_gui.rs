use common::prelude::*;
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

use game::entities::reactions::ReactionTypeRef;
use game::logic;
use common::prelude::*;
use graphics::core::GraphicsWrapper;
use common::event_bus::EventBus;
use gui::control_events::TacticalEvents;
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
use gui::action_bar::ActionBar;
use gui::reaction_bar::ReactionBar;
use gui::character_info::CharacterInfoWidget;
use gui::state::GameState;
use gui::state::KeyGameState;
use gui::action_bar::PlayerActionType;
use gui::inventory_widget;
use gui::escape_menu::*;
use std::fs::File;
use gui::state::ControlContext;
use gui::control_events::GameModeEvent;
use gui::harvest_detail_widget::HarvestSummaryWidget;
use graphics::GraphicsResources;
use action_ui_handlers::player_action_handler::PlayerActionHandler;
use action_ui_handlers::harvest_handler::HarvestHandler;
use game::logic::item;
use std::collections::HashSet;
use game::prelude::GameEventWrapper;
use graphics::AnimationElement;
use graphics::WaitAnimationElement;
use gui::character_dialog_widget::CharacterSpeechWidget;

#[derive(PartialEq,Clone,Copy)]
pub enum AuxiliaryWindows {
    Inventory,
}

pub struct TacticalGui {
    main_area : Widget,
    victory_widget : Widget,
    defeat_widget : Widget,
    pub(crate) action_bar : ActionBar,
    reaction_bar : ReactionBar,
    event_bus : EventBus<TacticalEvents>,
    event_bus_handle : ConsumerHandle,
    character_info_widget : CharacterInfoWidget,
    targeting_draw_list : DrawList,
    last_targeting_info : Option<(KeyGameState, PlayerActionType)>,
    messages_display : MessagesDisplay,
    inventory_widget: inventory_widget::InventoryDisplay,
    crafting_widget: crafting_widget::CraftingWidget,
    open_auxiliary_windows : Vec<AuxiliaryWindows>,
    escape_menu : EscapeMenu,
    player_action_handlers : Vec<Box<PlayerActionHandler>>,
    speech_widgets : Vec<CharacterSpeechWidget>,
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

        let victory_text = Widget::text("Victory!", FontSize::Points(30)).centered().parent(&victory_widget).apply(gui);

        let defeat_widget = Widget::window(Color::greyscale(0.9), 2)
            .size(Sizing::ux(50.0), Sizing::ux(30.0))
            .position(Positioning::centered(), Positioning::centered())
            .showing(false)
            .apply(gui);

        let defeat_text = Widget::text("Defeat!", FontSize::Points(30)).color(Color::new(0.6,0.1,0.1,1.0)).centered().parent(&defeat_widget).apply(gui);

        let event_bus = EventBus::new();
        let event_bus_handle = event_bus.register_consumer(true);

        let player_action_handlers : Vec<Box<PlayerActionHandler>> = vec![
            MoveAndAttackHandler::new(),
            HarvestHandler::new(),
        ];

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
            messages_display : MessagesDisplay::new(gui, &main_area),
            inventory_widget : inventory_widget::InventoryDisplay::new(strf("Character Inventory"), &main_area),
            crafting_widget : crafting_widget::CraftingWidget::new(&main_area),
            open_auxiliary_windows: Vec::new(),
            escape_menu : EscapeMenu::new(gui, &main_area),
            main_area,
            player_action_handlers,
            speech_widgets : Vec::new(),
        }
    }

    pub fn selected_player_action(&self, view : &WorldView, game_state : &GameState) -> PlayerActionType {
        if let Some(selected) = game_state.selected_character {
            self.action_bar.selected_action_for(view, selected)
        } else {
            PlayerActionType::None
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

//            if let Some((last_state, last_action_type)) = self.last_targeting_info.as_ref() {
//                if last_state == &game_state.key_state() && last_action_type == &action_type {
//                    return self.targeting_draw_list.clone()
//                }
//            }

            let current_position = cdata.position.hex;

            let visibility = view.world_data::<VisibilityData>().visibility_for(game_state.player_faction);

            let mut draw_list = DrawList::none();
            for handler in &mut self.player_action_handlers {
                draw_list.extend(handler.draw(view, game_state, &action_type));
            }

//            self.last_targeting_info = Some((game_state.key_state(), action_type.clone()));
//            self.targeting_draw_list = draw_list.clone();

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
        if self.open_auxiliary_windows.non_empty() || self.crafting_widget.is_showing() {
            for window in &self.open_auxiliary_windows {
                match window {
                    AuxiliaryWindows::Inventory => self.inventory_widget.set_showing(false).reapply(gui)
                }
            }
            self.crafting_widget.hide().reapply(gui);
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


    pub fn handle_click(&mut self, gui : &mut GUI, world: &mut World, game_state : &GameState, button : MouseButton) -> bool {
        let world_view = world.view();
        let action = self.selected_player_action(world_view, game_state);
        if gui.moused_over_widget() == Some(self.main_area.id()) {
            self.player_action_handlers.iter_mut().any(|pah| pah.handle_click(world, game_state, &action, button))
        } else { // handle any click that isn't going through to the main area
            true
        }
    }

    pub fn handle_key_release(&mut self, world : &mut World, gui : &mut GUI, game_state: &GameState, key : Key) -> bool {
        let world_view = world.view();
        let action = self.selected_player_action(world_view, game_state);

        if key == Key::C {
            self.crafting_widget.toggle(world_view, gui);
            true
        } else {
            self.player_action_handlers.iter_mut().any(|pah| pah.handle_key_release(world, game_state, &action, key))
        }
    }

    pub fn update_gui(&mut self, world: &mut World, world_view : &WorldView, gsrc : &mut GraphicsResources, gui: &mut GUI, frame_id: Option<Wid>, game_state: GameState, game_mode_event_bus : &mut EventBus<GameModeEvent>) {
        self.messages_display.update(gui);

        let selected_action = self.selected_player_action(world_view, &game_state);

        let mut control = ControlContext { event_bus : &mut self.event_bus };
        self.speech_widgets.iter_mut().for_each(|w| w.update(world_view, gui, &game_state, &mut control));

        if let Some(selected) = game_state.selected_character {

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
                actions.push(PlayerActionType::Harvest);
                actions.push(PlayerActionType::Wait);

                self.action_bar.update(gui, world_view, actions, &game_state, &mut control);
//                self.reaction_bar.set_x(Positioning::left_of(self.character_info_widget.as_widget(), 1.ux())).as_widget().reapply(gui);
//                self.reaction_bar.set_y(Positioning::constant(2.ux())).as_widget().reapply(gui);
                let reactions = vec![ReactionTypeRef::Defend, ReactionTypeRef::Dodge, ReactionTypeRef::Block, ReactionTypeRef::Counterattack];
                self.reaction_bar.update(gui, reactions, char.action.active_reaction, &game_state, &mut control);
            } else {
                self.action_bar.set_showing(false).reapply(gui);
                self.reaction_bar.set_showing(false).reapply(gui);
            }


            let inv_data = world_view.data::<InventoryData>(selected);
            let items = &inv_data.items;
            let destacked = item::items_in_inventory(world_view, selected);
            let all_equipped_items : HashSet<Entity> = vec![selected].iter()
                .flat_map(|ent| world.data_opt::<EquipmentData>(*ent).map(|eq| eq.equipped.clone()).unwrap_or(Vec::new()))
                .collect();

            let main_inv = vec![InventoryDisplayData::new(items.clone(), destacked, HashSet::new(), all_equipped_items, "Character Inventory", vec![selected], true, inv_data.inventory_size)];

            let mut ground_items = Vec::new();
            let mut ground_entities = Vec::new();
            let mut destacked_ground_items = Vec::new();

            for ground_coord in vec![char.position.hex].extended_by(char.position.hex.neighbors_vec()) {
                if let Some(ent) = world_view.tile_ent_opt(ground_coord) {
                    ground_entities.push(ent.entity);
                    if let Some(inv) = world_view.data_opt::<InventoryData>(ent.entity) {
                        ground_items.extend(inv.items.clone());
                        destacked_ground_items.extend(item::items_in_inventory(world_view, ent.entity));
                    }
                }
            }
            let other_inv = vec![InventoryDisplayData::new(ground_items, destacked_ground_items, HashSet::new(), HashSet::new(), "Ground", ground_entities, false, None)];
            self.inventory_widget.update(gui, world, main_inv, other_inv, &mut control);
            self.crafting_widget.update(world, gui, &game_state, &mut control);
        } else {
            self.main_area.set_width(Sizing::DeltaOfParent(0.ux())).reapply(gui);

            self.character_info_widget.set_showing(false).reapply(gui);
            self.action_bar.set_showing(false).reapply(gui);
            self.reaction_bar.set_showing(false).reapply(gui);
            self.inventory_widget.set_showing(false).reapply(gui);
            self.crafting_widget.hide().reapply(gui);
        }

        if gui.moused_over_widget() != Some(self.main_area.id()) {
            for handler in &mut self.player_action_handlers {
                handler.hide_widgets(gui);
            }
        } else {
            for handler in &mut self.player_action_handlers {
                handler.update_widgets(gui, gsrc, world, world_view, &game_state, &selected_action);
            }
        }



        for event in gui.events_for(&self.main_area) {
            if let Some((tactical_event,_)) = event.as_custom_event::<TacticalEvents>() {
                self.event_bus.push_event(tactical_event);
            }
        }

        for event in self.event_bus.events_for(&mut self.event_bus_handle) {
            let event : &TacticalEvents = event; // explicit typing so that IDEA can figure out what's going on
            if let Some(selected) = game_state.selected_character {
                match event {
                    TacticalEvents::DisplayMessage(message) => {
                        self.messages_display.add_message(message.clone())
                    },
                    TacticalEvents::ActionSelected(action_type) => {
                        println!("Selected action type : {:?}", action_type);
                    },
                    TacticalEvents::CancelActiveAction => {
                        world.modify(selected, ActionData::active_action.set_to(None));
                        world.add_event(GameEvent::ActionCanceled);
                    },
                    TacticalEvents::AttackSelected(attack_ref) => {
                        println!("Attack selected");
                        world.modify_with_desc(selected, CombatData::active_attack.set_to(attack_ref.clone()), "attack selected");
                        world.add_event(GameEvent::SelectedAttackChanged { entity : selected, attack_ref : attack_ref.clone() });
                    },
                    TacticalEvents::CounterattackSelected(attack_ref) => {
                        println!("Counter selected");
                        if logic::combat::is_valid_counter_attack(world, selected, attack_ref) {
                            world.modify_with_desc(selected, CombatData::active_counterattack.set_to(attack_ref.clone()), "counter-attack selected");
                            world.add_event(GameEvent::SelectedCounterattackChanged { entity : selected, attack_ref : attack_ref.clone() });
                        } else {
                            self.messages_display.add_message(Message::new("Only melee attacks can be used as counter-attacks, reach and ranged cannot."));
                        }
                    },
                    TacticalEvents::ReactionSelected(reaction_type) => {
                        println!("Selected reaction type : {:?}", reaction_type);
                        world.modify_with_desc(selected, ActionData::active_reaction.set_to(reaction_type.clone()), "reaction selected");
                        world.add_event(GameEvent::SelectedReactionChanged { entity : selected, reaction_type : reaction_type.clone() });
                    },
                    TacticalEvents::ItemTransferRequested { item , from, to } => {
                        if let Some(from) = from.find(|ent| world_view.data_opt::<InventoryData>(*ent).map(|inv| inv.items.contains(item)).unwrap_or(false)) {
                            let from_pos = world_view.data_opt::<PositionData>(*from).map(|pd| pd.hex).unwrap_or(AxialCoord::new(0,0));
                            // if these are ground tiles, pick the closest one
                            let single_to = if to.iter().all(|to| world_view.has_data::<TileData>(*to)) {
                                to.iter().min_by_key(|to| world_view.data::<TileData>(**to).position.distance(&from_pos))
                            } else {
                                to.first()
                            };
                            if let Some(single_to) = single_to {
                                if logic::item::transfer_item(world, *item, *from, *single_to) != logic::item::TransferResult::All {
                                    warn!("Could not transfer all of the items from {} to {}", world.view().signifier(*from), world.view().signifier(*single_to));
                                }
                            }
                        } else {
                            error!("none of the from entities actually held the desired item")
                        }
                    },
                    TacticalEvents::EquipItemRequested { item, equip_on } => {
                        if ! logic::item::is_item_equipped_by(world, *item, *equip_on) {
                            logic::item::equip_item(world, *item, *equip_on, true);
                        } else {
                            logic::item::unequip_item(world, *item, *equip_on, true);
                        }
                    },
                    TacticalEvents::SpeechDialogDismissed(character, wid) => {
                        self.speech_widgets.retain(|w| w.id() != *wid);
                    },
                    _ => {}
                }
            }

            match event {
                TacticalEvents::Save => {
                    game_mode_event_bus.push_event(GameModeEvent::Save(strf("savegame")));
                },
                TacticalEvents::MainMenu => {
                    game_mode_event_bus.push_event(GameModeEvent::MainMenu);
                },
                _ => {}
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

    pub fn toggle_escape_menu(&mut self, gui : &mut GUI) {
        self.escape_menu.toggle_showing().reapply(gui)
    }


    pub fn animation_elements_for_new_event(&mut self, world_view: &WorldView, wrapper: &GameEventWrapper<GameEvent>, resources: &mut GraphicsResources) -> Vec<Box<AnimationElement>> {
        use gui::character_dialog_widget::CharacterSpeechWidget;
        match wrapper.if_starting() {
            Some(GameEvent::DialogSpoken { speaker, requires_confirmation, text }) => {
                let widget = CharacterSpeechWidget::new(*speaker, text.clone(), *requires_confirmation)
                    .parent(&self.main_area);

                self.speech_widgets.push(widget);

                vec![box WaitAnimationElement::new(1.0)]
            },
            _ => vec![]
        }
    }
}