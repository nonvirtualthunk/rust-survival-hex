use game::prelude::*;
use game::entities::*;

use noisy_float::prelude::*;
use itertools::Itertools;

use piston_window::*;
use common::prelude::*;

use core::GameMode;
use core::normalize_screen_pos;

use graphics::core::GraphicsWrapper;
use graphics::animation::AnimationElement;
use graphics::interpolation::*;
use common::hex::*;
use common::color::Color;
use graphics::core::Quad;
use graphics::core::Text;
use graphics::camera::*;
use piston_window::math;
use vecmath;
use cgmath::InnerSpace;
use game::core::*;
use game::logic;
use game::logic::movement;
use game::Entity;
use game::world_util::*;
//use game::ConstantModifier;
use game::entities::modifiers::*;
use game::logic::visibility::VisibilityComputor;
use game;
use game::universe::*;
use game::entities::CharacterData;
use gui;
use common::EventBus;

use game::logic::combat::*;

use pathfinding::prelude::astar;
use std::cmp::*;
use cgmath::num_traits::Zero;
use std::ops::Add;

use graphics::renderers::*;

use tactical_gui::TacticalGui;
use tactical_gui;

use tactical_event_handler;
use common::Rect;
use ron;
use bincode;

use std::time::*;

use interpolation::*;
use graphics::core::DrawList;
use graphics::core::GraphicsResources;

use gui::UIEvent;
use gui::UIEventType;
use gui::GUI;
use gui::Wid;
use gui::MouseButton;
use gui::Key;

use game::logic::faction::is_enemy;
use std::collections::HashSet;
use gui::state::GameState;
use game::entities::time::TurnData;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use gui::control_events::GameModeEvent;

pub struct AnimationElementWrapper {
    pub animation: Box<AnimationElement>,
    pub start_time: Option<f64>,
}

impl AnimationElementWrapper {
    fn pcnt_elapsed(&self, realtime_clock: f64) -> f64 {
        ((realtime_clock - self.start_time.unwrap_or(realtime_clock)) / self.animation.raw_duration()) as f64
    }

    fn blocking_pcnt_elapsed(&self, realtime_clock: f64) -> f64 {
        ((realtime_clock - self.start_time.unwrap_or(realtime_clock)) / self.animation.blocking_duration()) as f64
    }
}

pub struct TacticalMode {
    pub world_ref: WorldRef,
    pub selected_character: Option<Entity>,
    pub tile_radius: f32,
    pub mouse_pos: Vec2f,
    pub camera: Camera2d,
    pub viewport: Viewport,
    pub terrain_renderer: TerrainRenderer,
    pub unit_renderer: UnitRenderer,
    pub item_renderer: ItemRenderer,
    pub display_event_clock: GameEventClock,
    pub realtime_clock: f64,
    pub realtime_clock_speed: f64,
    pub last_realtime_clock: f64,
    pub time_within_event: f64,
    pub fract_within_event: f64,
    pub event_clock_last_advanced: f64,
    pub last_time: Instant,
    pub last_update_time: Instant,
    pub player_faction: Entity,
    pub visibility_computor: VisibilityComputor,
    pub minimum_active_event_clock: GameEventClock,
    event_start_times: Vec<f64>,
    pub victory_time: Option<GameEventClock>,
    pub defeat_time: Option<GameEventClock>,
    animation_elements: Vec<AnimationElementWrapper>,
    gui: TacticalGui,
    display_world_view: WorldView,
    show_real_world: bool,
    skipped_characters: HashSet<Entity>,
    start_at_beginning: bool,
}

impl TacticalMode {
    pub fn new(gui: &mut GUI, world_ref: WorldRef, start_at_beginning: bool) -> TacticalMode {
        let tile_radius = 35.5;
        let mut camera = Camera2d::new();
        camera.move_speed = tile_radius * 20.0;
        camera.zoom = 1.0;

        TacticalMode {
            world_ref,
            display_world_view: WorldView::default(),
            selected_character: None,
            tile_radius,
            mouse_pos: v2(0.0, 0.0),
            camera,
            viewport: Viewport {
                window_size: [256, 256],
                draw_size: [256, 256],
                rect: [0, 0, 256, 256],
            },
            terrain_renderer: TerrainRenderer::default(),
            unit_renderer: UnitRenderer {},
            item_renderer: ItemRenderer::new(),
            display_event_clock: 0,
            last_realtime_clock: 0.0,
            realtime_clock: 0.0,
            realtime_clock_speed: 1.0,
            time_within_event: 0.0,
            fract_within_event: 0.0,
            event_clock_last_advanced: 0.0,
            last_time: Instant::now(),
            last_update_time: Instant::now(),
            player_faction : Entity::sentinel(),
            minimum_active_event_clock: 0,
            event_start_times: vec![0.0],
            victory_time: None,
            defeat_time: None,
            animation_elements: vec![],
            gui: TacticalGui::new(gui),
            show_real_world: false,
            skipped_characters: HashSet::new(),
            visibility_computor: VisibilityComputor::new(),
            start_at_beginning,
        }
    }

    pub fn selected_player_character(&self, world: &WorldView) -> Option<Entity> {
        if let Some(sel_ref) = self.selected_character {
            if world.character(sel_ref).allegiance.faction == self.player_faction {
                Some(sel_ref)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn screen_pos_to_game_pos(&self, screen_pos : Vec2f) -> Vec2f {
        let camera_mat = self.camera.matrix(self.viewport);
        let inverse_mat = vecmath::mat2x3_inv(camera_mat);
        let norm_screen_pos = normalize_screen_pos(screen_pos, &self.viewport);
        let transformed_pos = math::transform_pos(inverse_mat, [norm_screen_pos.x as f64, norm_screen_pos.y as f64]);
        let transformed_pos = v2(transformed_pos[0] as f32, transformed_pos[1] as f32);
        transformed_pos
    }

    pub fn mouse_game_pos(&self) -> Vec2f {
        self.screen_pos_to_game_pos(self.mouse_pos)
    }

    pub fn quad(&self, texture_identifier: String, pos: AxialCoord) -> Quad {
        Quad::new(texture_identifier, pos.as_cartesian(self.tile_radius)).centered()
    }
    pub fn colored_quad(&self, texture_identifier: String, pos: AxialCoord, color: Color) -> Quad {
        Quad::new(texture_identifier, pos.as_cartesian(self.tile_radius)).centered().color(color)
    }
    pub fn colored_quad_cart(&self, texture_identifier: String, pos: Vec2f, color: Color) -> Quad {
        Quad::new(texture_identifier, pos).centered().color(color)
    }

    fn blocking_animations_active(&self) -> bool {
        self.animation_elements.iter().any(|e| e.blocking_pcnt_elapsed(self.realtime_clock) < 1.0)
    }

    pub fn advance_event_clock(&mut self, max_event_clock: GameEventClock) -> bool {
        let dt = Instant::now().duration_since(self.last_time).subsec_nanos() as f64 / 1e9f64;
        //        println!("Advancing event clock by {}", dt);
        self.last_time = Instant::now();
        self.last_realtime_clock = self.realtime_clock;
        self.realtime_clock += dt * self.realtime_clock_speed;

        // max_event_clock          the world time the next event will be added at when made
        // max_event_clock-1        the world time of the latest event in the world so far
        // display_event_clock      the world time used as a base-point for display
        // display_event_clock-1    the world time of the event that is currently animating
        if self.display_event_clock < max_event_clock - 1 {
            let still_blocking = self.blocking_animations_active();
            if !still_blocking {
                let realtime_clock = self.realtime_clock;
                // if we're going to advance the DEC we need to clear out all of the finished animations, the only
                // ones remaining will be non-blocking animations that are presumed to be ok to continue
                self.animation_elements.retain(|e| e.pcnt_elapsed(realtime_clock) < 1.0);
                // advance the DEC
                self.display_event_clock += 1;
                return true;
            }
        }
        false
    }

    pub fn at_latest_event(&self, world: &World) -> bool {
        self.display_event_clock >= world.current_time()
    }

    fn event_started_at(&self, gec: GameEventClock) -> f64 {
        *self.event_start_times.get(gec as usize).unwrap()
    }

    fn end_turn(&mut self, world: &mut World) {
        let world_view = world.view();
        let current_turn = world_view.world_data::<TurnData>().turn_number;


        loop {
            logic::turn::end_faction_turn(world);

            let newly_active_faction = world_view.world_data::<TurnData>().active_faction;
            let newly_active_faction_data = world_view.data::<FactionData>(newly_active_faction);
            if ! newly_active_faction_data.player_faction {
                ::ai::ai::take_ai_actions(world, newly_active_faction);
            }
            if world_view.world_data::<TurnData>().active_faction == self.player_faction {
                break;
            }
        }

        let mut living_enemy = false;
        let mut living_ally = false;
        for (cref,char_data) in world_view.entities_with_data::<CharacterData>() {
            if char_data.is_alive() {
                let allegiance = world_view.data::<AllegianceData>(*cref);
                if allegiance.faction != self.player_faction {
                    living_enemy = true;
                } else {
                    living_ally = true;
                }
            }
        }
        if !living_enemy {
            self.victory_time = self.victory_time.or(Some(world.next_time -1));
        } else if !living_ally {
            self.defeat_time = self.defeat_time.or(Some(world.next_time -1));
        }

        self.skipped_characters.clear();
        if self.selected_character.is_none() {
            println!("End turn complete, selecting a character");
            self.select_next_character(world);
        }
    }


    fn add_animation_element(&mut self, elem: Box<AnimationElement>) {
        self.animation_elements.push(AnimationElementWrapper { animation: elem, start_time: None });
    }

    fn render_draw_list(&self, mut draw_list: DrawList, g: &mut GraphicsWrapper) {
        for quad in draw_list.quads.iter_mut() {
            quad.offset *= self.tile_radius;
            quad.size = match quad.size {
                Some(specific_size) => Some(specific_size * self.tile_radius),
                None => None
            };
//            g.draw_quad(quad);
        }

        g.draw_quads(&draw_list.quads,"test");

        for mut text in draw_list.text {
            text.offset *= self.tile_radius;
            g.draw_text(text);
        }
    }

    fn create_animations_if_necessary(&mut self, world_in: &World, resources : &mut GraphicsResources) {
        if !self.blocking_animations_active() && !self.at_latest_event(world_in) {
            if let Some(event_wrapper) = world_in.event_at(self.display_event_clock + 1) {
                trace!("Advanced event, new event is {:?}", event_wrapper.event);
                for elem in tactical_event_handler::animation_elements_for_new_event(&self.display_world_view, event_wrapper, resources) {
                    self.add_animation_element(elem)
                }
            };
        }
    }

    fn hovered_tile<'a, 'b>(&'a self, world: &'b WorldView) -> Option<TileEntity<'b>> {
        let hovered_hex = AxialCoord::from_cartesian(&self.mouse_game_pos(), self.tile_radius);
        world.tile_ent_opt(hovered_hex)
    }

    fn current_game_state(&self, world: &World) -> gui::state::GameState {
        let mouse_game_pos = self.screen_pos_to_game_pos(self.mouse_pos);

        gui::state::GameState {
            display_event_clock: self.display_event_clock,
            selected_character: self.selected_character,
            victory_time: self.victory_time,
            defeat_time: self.defeat_time,
            player_faction: self.player_faction,
            hovered_hex_coord : AxialCoord::from_cartesian(&self.mouse_game_pos(), self.tile_radius),
            animating: !self.at_latest_event(world),
            mouse_pixel_pos: self.mouse_pos,
            mouse_game_pos,
            mouse_cart_vec: CartVec::new(mouse_game_pos.x / self.tile_radius, mouse_game_pos.y / self.tile_radius),
            player_faction_active: self.display_world_view.world_data::<TurnData>().active_faction == self.player_faction,
        }
    }

    fn update_world_view(&mut self, world : &World) {
        world.update_view_to_time(&mut self.display_world_view, self.display_event_clock);
    }

    fn active_world<'a, 'b>(&'a self, universe : &'b mut Universe) -> &'b mut World {
        universe.world(self.world_ref)
    }
}

impl GameMode for TacticalMode {
    fn enter(&mut self, gui: &mut GUI, universe: &mut Universe, event_bus: &mut EventBus<GameModeEvent>) {
        let world = self.active_world(universe);

        self.player_faction = *world.view().entities_with_data::<FactionData>().find(|(ent,faction_data)| faction_data.player_faction).unwrap().0;

        let dec = if self.start_at_beginning {
            world.events::<GameEvent>().find(|e| e.is_ended() && (if let GameEvent::WorldStart = e.event { true } else { false })).map(|e| e.occurred_at).unwrap_or(0)
        } else {
            world.next_time-1
        };
        self.display_event_clock = dec;
        self.display_world_view = world.view_at_time(dec);

        world.add_callback(|world, event| {
            logic::reaction::trigger_reactions_for_event(world, event);
        });

        game::components::SpawningComponent::register(world);

        VisibilityComputor::register(world);
    }

    fn update(&mut self, universe: &mut Universe, _: f64, event_bus: &mut EventBus<GameModeEvent>) {
        let world = self.active_world(universe);
        let dt = Instant::now().duration_since(self.last_update_time).subsec_nanos() as f64 / 1e9f64;
        self.last_update_time = Instant::now();
        self.camera.position = self.camera.position + self.camera.move_delta * (dt as f32) * self.camera.move_speed;
    }

    fn update_gui(&mut self, universe: &mut Universe, gsrc : &mut GraphicsResources, ui: &mut GUI, frame_id: Option<Wid>, event_bus: &mut EventBus<GameModeEvent>) {
        let world = self.active_world(universe);
        let game_state = self.current_game_state(world);
        self.gui.update_gui(world, &self.display_world_view, gsrc, ui, frame_id, game_state, event_bus);
    }

    fn draw(&mut self, universe: &mut Universe, g: &mut GraphicsWrapper, event_bus: &mut EventBus<GameModeEvent>) {
        let world_in = self.active_world(universe);
//        let active_events = self.active_events(world_in);
//        let mut world_view = world_in.view_at_time(self.display_event_clock);

//        let world_view : &mut WorldView = &mut self.world_view;

        loop {
            self.create_animations_if_necessary(world_in, g.resources);

            if self.advance_event_clock(world_in.next_time) {
                self.update_world_view(world_in);
            } else {
                break;
            }
        }

        if let Some(v) = g.context.viewport {
            self.viewport = v;
        }
        g.context.view = self.camera.matrix(self.viewport);

        // draw list of all of the drawing requests from all currently active animations
        let mut anim_draw_list = DrawList::default();

        let realtime_clock = self.realtime_clock;
        for elem in &mut self.animation_elements {
            elem.start_time = elem.start_time.or(Some(self.last_realtime_clock));
            let pcnt = elem.pcnt_elapsed(realtime_clock).min(1.0);
            anim_draw_list.append(&mut elem.animation.draw(&mut self.display_world_view, pcnt));

            let blocking_pcnt = elem.blocking_pcnt_elapsed(realtime_clock);
            if blocking_pcnt < 1.0 {
                break;
            }
        }

        let world_view = if ! self.show_real_world {
            &self.display_world_view
        } else {
            world_in.view()
        };

        let min_game_pos = self.screen_pos_to_game_pos(v2(0.0,0.0));
        let max_game_pos = self.screen_pos_to_game_pos(v2(self.viewport.window_size[0] as f32, self.viewport.window_size[1] as f32));

        let culling_rect = Rect::from_corners(min_game_pos.x / self.tile_radius, max_game_pos.y / self.tile_radius, max_game_pos.x / self.tile_radius, min_game_pos.y / self.tile_radius);

        // draw list for the map tiles
        let terrain_draw_list = self.terrain_renderer.render_tiles(world_view, self.player_faction, self.display_event_clock, culling_rect);
        // draw list for items on the map
        let item_draw_list = self.item_renderer.render_items(world_view, g.resources, culling_rect);
        // draw list for the units and built-in unit UI elements
        let unit_draw_list = self.unit_renderer.render_units(world_view, self.display_event_clock, self.player_faction, self.selected_character);

        let ui_draw_list = self.gui.draw(world_view, self.current_game_state(world_in));

        self.render_draw_list(terrain_draw_list, g);
        self.render_draw_list(item_draw_list, g);
        self.render_draw_list(ui_draw_list, g);
        self.render_draw_list(unit_draw_list, g);
        self.render_draw_list(anim_draw_list, g);

        self.display_world_view.clear_overlay();
    }


    fn handle_event(&mut self, universe: &mut Universe, gui: &mut GUI, event: &UIEvent, event_bus: &mut EventBus<GameModeEvent>) {
        let world = self.active_world(universe);

        match event {
            UIEvent::MouseMove { pos, .. } => {
                self.mouse_pos = pos.pixel_pos
            }
            UIEvent::MouseRelease { pos, button } if self.at_latest_event(world) => match button {
                MouseButton::Left => {
                    let mouse_pos = self.mouse_game_pos();
                    let clicked_coord = AxialCoord::from_cartesian(&mouse_pos, self.tile_radius);


                    self.update_world_view(world);
                    let display_world_view = &self.display_world_view;
                    let main_world_view = world.view();

                    let game_state = self.current_game_state(world);
                    if !game_state.animating && game_state.player_faction_active {
                        if ! self.gui.handle_click(world, &game_state, *button) {
                            let found = character_at(display_world_view, clicked_coord);
                            if let Some((found_char, found_data)) = found {
                                match self.selected_character {
                                    Some(cur_sel) => {
                                        if cur_sel == found_char {
                                            self.gui.close_all_auxiliary_windows(gui);
                                            self.selected_character = None;
                                        } else if main_world_view.data::<AllegianceData>(cur_sel).faction == found_data.allegiance.faction {
                                            self.selected_character = Some(found_char)
                                        }
                                    },
                                    None => { self.selected_character = Some(found_char); }
                                }
                            }
                        }
                    }
                }
                _ => ()
            },
            UIEvent::KeyPress { key } => {
                match key {
                    Key::Left => self.camera.move_delta.x = -1.0,
                    Key::Right => self.camera.move_delta.x = 1.0,
                    Key::Up => self.camera.move_delta.y = 1.0,
                    Key::Down => self.camera.move_delta.y = -1.0,
                    Key::Z => self.camera.zoom += 0.1,
                    Key::LShift => self.realtime_clock_speed = 3.0,
                    Key::LAlt => self.realtime_clock_speed = 0.3,
                    Key::LCtrl => self.show_real_world = true,
                    _ => ()
                }
            },
            UIEvent::KeyRelease { key } => {
                match key {
                    Key::A => gui.mark_all_modified(),
                    Key::Left => self.camera.move_delta.x = 0.0,
                    Key::Right => self.camera.move_delta.x = 0.0,
                    Key::Up => self.camera.move_delta.y = 0.0,
                    Key::Down => self.camera.move_delta.y = 0.0,
                    Key::Return => {
                        self.gui.close_all_auxiliary_windows(gui);
                        self.end_turn(world);
                    },
                    Key::Escape => {
                        // close auxiliary windows if any are open, otherwise cancel character selection
                        if ! self.gui.close_all_auxiliary_windows(gui) {
                            if self.selected_character.is_some() {
                                self.selected_character = None
                            } else {
                                self.gui.toggle_escape_menu(gui);
                            }
                        }
                    },
                    Key::LShift => self.realtime_clock_speed = 1.0,
                    Key::LAlt => self.realtime_clock_speed = 1.0,
                    Key::D9 => self.realtime_clock_speed = if self.realtime_clock_speed < 100.0 { 100.0 } else  { 1.0 },
                    Key::I => self.gui.toggle_inventory(gui),
                    Key::LCtrl => self.show_real_world = false,
                    Key::N => self.select_next_character(world),
                    Key::Space => {
                        if let Some(sel) = self.selected_character {
                            if self.skipped_characters.contains(&sel) {
                                self.skipped_characters.remove(&sel);
                            } else {
                                self.skipped_characters.insert(sel);
                            }
                        }
                        println!("Skip, selecting a character");
                        self.select_next_character(world);
                        if self.selected_character.is_none() {
                            self.end_turn(world);
                        }
                    },
                    Key::Y => {
                        use std::io::Write;
                        let string = ron::ser::to_string_pretty(&world, ron::ser::PrettyConfig::default()).expect("couldn't deserialize");
                        File::create(Path::new("/tmp/world.ron")).expect("couldn't create").write_all(string.as_bytes()).expect("couldn't write");
                    },
                    Key::U => {
                        use std::io::Write;
                        let bytes = ron::ser::to_string_pretty(&world.view().effective_data, ron::ser::PrettyConfig::default()).expect("couldn't deserialize");
                        File::create(Path::new("/tmp/view.ron")).expect("couldn't create").write_all(&bytes.as_bytes()).expect("couldn't write");
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }


    fn on_event(&mut self, universe: &mut Universe, gui: &mut GUI, event: &UIEvent, event_bus: &mut EventBus<GameModeEvent>) {
        let world = self.active_world(universe);
        gui.handle_ui_event_2(event, self, world);
    }
}


impl TacticalMode {
    fn select_next_character(&mut self, world : &World) {
        let world_view = world.view();
        let mut player_characters : Vec<Entity> = world_view.entities_with_data::<CharacterData>()
            .filter(|(k,v)| world_view.data::<AllegianceData>(**k).faction == self.player_faction)
            .map(|(k,v)| *k)
            .sorted();

        // eliminate everything that isn't alive, and everything that has been skipped (unless it's the selected char right now, in which case we need it in order to switch past it)
        player_characters.retain(|e| {
            let character = world_view.character(*e);
            let skip = self.skipped_characters.contains(e) && self.selected_character.as_ref() != Some(e);
            character.action_points.cur_value() > 0 && character.is_alive() && ! skip
        });

        let cur_index = if let Some(sel) = self.selected_character {
            player_characters.iter().position(|e| e == &sel).unwrap_or(0)
        } else {
            player_characters.len() - 1
        };

        let new_index = if player_characters.len() > 0 { (cur_index + 1) % player_characters.len() } else { 0 };
        if new_index < player_characters.len() {
            // if the next one to be selected has been skipped, that means everyone has, because the only skipped on allowed ot remain in the list was the cur selected one
            // so that means _everyone_ has been skipped
            let next_selected = player_characters[new_index];
            self.selected_character = if self.skipped_characters.contains(&next_selected) {
                None
            } else {
                Some(next_selected)
            };
        } else {
            println!("No one to select?");
            self.selected_character = None;
        }
    }
}