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
use game::events::*;
use game::logic;
use game::logic::movement;
use game::Entity;
use game::world_util::*;
//use game::ConstantModifier;
use game::entities::modifiers::*;
use game::logic::visibility::VisibilityComputor;
use game;
use game::entities::CharacterData;
use gui;

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

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Cost(pub R32);

impl Eq for Cost {}

impl Ord for Cost {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Zero for Cost {
    fn zero() -> Self {
        Cost(R32::new(0.0))
    }

    fn is_zero(&self) -> bool {
        self.0 == R32::new(0.0)
    }
}

impl Add<Cost> for Cost {
    type Output = Cost;

    fn add(self, rhs: Cost) -> Self::Output {
        Cost(self.0 + rhs.0)
    }
}

impl Cost {
    pub fn new(f: f64) -> Cost {
        Cost(R32::from_f64(f))
    }
}


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
}

impl TacticalMode {
    pub fn new(gui: &mut GUI, world: &World, player_faction : Entity) -> TacticalMode {
        let tile_radius = 35.5;
        let mut camera = Camera2d::new();
        camera.move_speed = tile_radius * 20.0;
        camera.zoom = 1.0;


        let dec = world.events::<GameEvent>().find(|e| e.is_ended() && (if let GameEvent::WorldStart = e.event { true } else { false })).map(|e| e.occurred_at).unwrap_or(0);
        TacticalMode {
            display_world_view: world.view_at_time(dec),
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
            display_event_clock: dec,
            last_realtime_clock: 0.0,
            realtime_clock: 0.0,
            realtime_clock_speed: 1.0,
            time_within_event: 0.0,
            fract_within_event: 0.0,
            event_clock_last_advanced: 0.0,
            last_time: Instant::now(),
            last_update_time: Instant::now(),
            player_faction,
            minimum_active_event_clock: 0,
            event_start_times: vec![0.0],
            victory_time: None,
            defeat_time: None,
            animation_elements: vec![],
            gui: TacticalGui::new(gui),
            show_real_world: false,
            skipped_characters: HashSet::new(),
            visibility_computor: VisibilityComputor::new(),
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

    fn ai_action(&mut self, ai_ref: &Entity, cdata: &CharacterData, world: &mut World, world_view: &WorldView, _all_characters: &Vec<Entity>) {
        let ai = world_view.character(*ai_ref);

        if ai.movement.move_speed == Sext::of(0) {

        } else {
            let closest_enemy = world_view.entities_with_data::<CharacterData>().iter()
                .filter(|&(_, c)| c.is_alive())
                .filter(|&(cref, _)| is_enemy(world_view, *ai_ref, *cref))
                .min_by_key(|t| world_view.data::<PositionData>(*t.0).hex.distance(&ai.position.hex));

            if let Some(closest) = closest_enemy {
                let enemy_ref: &Entity = closest.0;
                let enemy_data = world_view.character(*closest.0);

                if enemy_data.position.distance(&ai.position) >= r32(1.5) {
                    if let Some(path) = logic::movement::path_any_v(world_view, *ai_ref, ai.position.hex, &enemy_data.position.hex.neighbors_vec(), enemy_data.position.hex) {
                        movement::handle_move(world, *ai_ref, path.0.as_slice())
                    } else {
                        println!("No move towards closest enemy, stalling");
                    }
                }

                if enemy_data.position.distance(&ai.position) < r32(1.5) {
                    let all_possible_attacks = possible_attack_refs(world_view, *ai_ref);
                    if let Some(attack) = all_possible_attacks.first() {
                        logic::combat::handle_attack(world, *ai_ref, *enemy_ref, attack);
                    }
                }
            } else {
                if let Some(path) = logic::movement::path(world_view, *ai_ref, ai.position.hex, ai.position.hex.neighbor(0)) {
                    logic::movement::handle_move(world, *ai_ref, path.0.as_slice());
                }
            }
        }
    }

    fn end_turn(&mut self, world: &mut World) {
        let world_view = world.view();
        let current_turn = world_view.world_data::<TurnData>().turn_number;

        let mut prev_faction = self.player_faction;

        for (faction, faction_data) in world_view.entities_with_data::<FactionData>() {
            if faction != &self.player_faction {
                world.modify_world(TurnData::active_faction.set_to(*faction), None);
                world.end_event(GameEvent::FactionTurn { turn_number : current_turn, faction : prev_faction });
                world.start_event(GameEvent::FactionTurn { turn_number : current_turn, faction : *faction });

                prev_faction = *faction;

                let character_refs: Vec<Entity> = world.view().entities_with_data::<CharacterData>().keys().cloned().collect::<Vec<Entity>>();

                for (cref, cur_data) in world_view.entities_with_data::<CharacterData>() {
                    let allegiance = world_view.data::<AllegianceData>(*cref);
                    if &allegiance.faction == faction && cur_data.is_alive() {
                        // these are enemies, now we get to decide what they want to do
                        self.ai_action(&cref, &cur_data, world, world_view, &character_refs);
                    }
                }
            }
        }

        // recompute the character set, some may have been created, hypothetically
        let character_refs = world_view.entities_with_data::<CharacterData>();

        for (cref, cdat) in character_refs {
            world.modify(*cref, MovementData::moves.set_to(Sext::of(0)), None);
            world.modify(*cref, CharacterData::action_points.reset(), None);
            world.modify(*cref, CharacterData::stamina.recover_by(cdat.stamina_recovery), None);
        }

        let turn_number = current_turn + 1;
        world.modify_world(TurnData::turn_number.set_to(turn_number), None);

        world.add_event(GameEvent::TurnStart { turn_number });

        // back to the player's turn
        world.modify_world(TurnData::active_faction.set_to(self.player_faction), None);
        world.end_event(GameEvent::FactionTurn { turn_number : current_turn, faction : prev_faction });
        world.start_event(GameEvent::FactionTurn { turn_number : current_turn, faction : self.player_faction });

        let mut living_enemy = false;
        let mut living_ally = false;
        for cref in character_refs.keys() {
            let char_data = world_view.data::<CharacterData>(*cref);
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

    fn create_move_ui_draw_list(&mut self, world_in: &mut World, game_state : &GameState) -> DrawList {
        let is_animating = !self.at_latest_event(world_in);
        if is_animating {
            DrawList::none()
        } else {
            if let Some(hovered_tile) = self.hovered_tile(world_in.view()) {
                let world_view = world_in.view();
                let hovered_hex = hovered_tile.position;

                let mut draw_list = DrawList::of_quad(Quad::new_cart(String::from("ui/hoverHex"), hovered_hex.as_cart_vec()).centered());

                if let Some(selected) = self.selected_character {
                    let sel_c = world_in.view().character(selected);

                    if let Some(hovered_occupant) = hovered_tile.occupied_by {
                        if logic::faction::is_enemy(world_view, hovered_occupant, selected) {
                            if let Some(attack_ref) = logic::combat::primary_attack_ref(world_view, selected) {
                                let path = logic::combat::path_to_attack(world_view, selected, hovered_occupant, &attack_ref, game_state.mouse_cart_vec()).map(|t| t.0)
                                    .or_else(|| logic::movement::path_adjacent_to(world_view, selected, hovered_occupant).map(|t| t.0));
//                                    .map(|path| logic::movement::portion_of_path_traversable_this_turn(world_view, selected, &path));

                                if let Some(path) = path {
                                    for hex in path {
                                        draw_list = draw_list.with_quad(Quad::new_cart(String::from("ui/feet"), hex.as_cart_vec()).centered());
                                    }
                                }
                            }
                        }
                    } else {
                        if let Some(path_result) = logic::movement::path(world_view, selected, sel_c.position.hex, hovered_hex) {
                            let path = path_result.0;
                            for hex in path {
                                draw_list = draw_list.with_quad(Quad::new_cart(String::from("ui/feet"), hex.as_cart_vec()).centered());
                            }
                        }
                    }
                }

                draw_list
            } else {
                DrawList::none()
            }
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
            mouse_cart_vec: CartVec::new(mouse_game_pos.x / self.tile_radius, mouse_game_pos.y / self.tile_radius)
        }
    }

    fn update_world_view(&mut self, world : &World) {
        world.update_view_to_time(&mut self.display_world_view, self.display_event_clock);
    }
}

impl GameMode for TacticalMode {
    fn enter(&mut self, world: &mut World) {
        world.add_callback(|world, event| {
            for (ent,act_data) in world.view().entities_with_data::<ActionData>() {
                (act_data.active_reaction.on_event)(world, *ent, event);
            }
        });

        game::components::SpawningComponent::register(world);

        VisibilityComputor::register(world);
    }

    fn update(&mut self, _: &mut World, _: f64) {
        let dt = Instant::now().duration_since(self.last_update_time).subsec_nanos() as f64 / 1e9f64;
        self.last_update_time = Instant::now();
        self.camera.position = self.camera.position + self.camera.move_delta * (dt as f32) * self.camera.move_speed;
    }

    fn update_gui(&mut self, world: &mut World, ui: &mut GUI, frame_id: Option<Wid>) {
        self.gui.update_gui(world, &self.display_world_view, ui, frame_id, self.current_game_state(world));
    }

    fn draw(&mut self, world_in: &mut World, g: &mut GraphicsWrapper) {
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

        // draw list for hover hex and foot icons
//        let movement_ui_draw_list = self.create_move_ui_draw_list(world_in, &self.current_game_state(world_in));

        self.render_draw_list(terrain_draw_list, g);
        self.render_draw_list(item_draw_list, g);
        self.render_draw_list(ui_draw_list, g);
//        self.render_draw_list(movement_ui_draw_list, g);
        self.render_draw_list(unit_draw_list, g);
        self.render_draw_list(anim_draw_list, g);

        let gui_camera = Camera2d::new();
        g.context.view = gui_camera.matrix(self.viewport);


        self.display_world_view.clear_overlay();
    }


    fn handle_event(&mut self, world: &mut World, gui: &mut GUI, event: &UIEvent) {
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
//                    let world_view = world.view_at_time(self.display_event_clock);

                    let found = character_at(display_world_view, clicked_coord);
                    if let Some((found_ref, target_data)) = found {
                        match self.selected_character {
                            Some(prev_sel) if prev_sel == found_ref => {
                                self.gui.close_all_auxiliary_windows(gui);
                                self.selected_character = None;
                            },
                            Some(cur_sel) => {
                                let sel_data = main_world_view.character(cur_sel);
                                if target_data.allegiance.faction != self.player_faction &&
                                    sel_data.allegiance.faction == self.player_faction {
                                    if let Some(attack_ref) = logic::combat::primary_attack_ref(main_world_view, cur_sel) {
                                        if let Some((path, cost)) = logic::combat::path_to_attack(main_world_view, cur_sel, found_ref, &attack_ref, self.current_game_state(world).mouse_cart_vec()) {
                                            if path.is_empty() {
                                                println!("no movement needed, attacking");
                                                logic::combat::handle_attack(world, cur_sel, found_ref, &attack_ref);
                                            } else {
                                                logic::movement::handle_move(world, cur_sel, &path);
                                                if let Some(attack) = attack_ref.resolve(main_world_view, cur_sel) {
                                                    if logic::combat::can_attack(main_world_view, cur_sel, found_ref, &attack, None, None) {
                                                        println!("Can attack from new position, attacking");
                                                        logic::combat::handle_attack(world, cur_sel, found_ref, &attack_ref);
                                                    } else {
                                                        warn!("Could not attack from new position :(, but should have been");
                                                    }
                                                }
                                            }
                                        } else {
                                            if let Some((path,cost)) = logic::movement::path_adjacent_to(world, cur_sel, found_ref) {
                                                println!("Moving adjacent, no path to attack could be found");
                                                logic::movement::handle_move(world, cur_sel, &path);
                                            } else {
                                                println!("no adjacent to path");
                                            }
                                        }
                                    } else {
                                        warn!("Cannot attack, there are no attacks to use!");
                                    }
                                } else {
                                    println!("Switching selected character");
                                    self.selected_character = Some(found_ref);
                                }
                            }
                            None => {
                                println!("Switching selected character");
                                self.selected_character = Some(found_ref);
                            }
                        }
                    } else {
                        if let Some(sel_c) = self.selected_character {
                            if let Some(hovered_) = self.hovered_tile(world.view()) {
                                let cur_sel_data = display_world_view.character(sel_c);
                                if cur_sel_data.allegiance.faction == self.player_faction {
                                    let start_pos = display_world_view.character(sel_c).position.hex;
                                    if let Some(path_result) = logic::movement::path(display_world_view, sel_c, start_pos, clicked_coord) {
                                        let path = path_result.0;
                                        logic::movement::handle_move(world, sel_c, path.as_slice());
                                    }
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
                            self.selected_character = None
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
                    _ => ()
                }
            },
            _ => ()
        }
    }


    fn on_event(&mut self, world: &mut World, gui: &mut GUI, event: &UIEvent) {
        gui.handle_ui_event_2(event, self, world);
    }
}


impl TacticalMode {
    fn select_next_character(&mut self, world : &World) {
        let world_view = world.view();
        let mut player_characters : Vec<Entity> = world_view.entities_with_data::<CharacterData>().iter()
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