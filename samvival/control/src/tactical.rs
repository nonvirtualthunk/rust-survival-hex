use game::World;
use game::world::WorldView;
use game::entities::*;
use game::EntityBuilder;

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
use game::ConstantModifier;
use game::entities::modifiers::*;

use game::logic::combat::*;

use pathfinding::prelude::astar;
use std::cmp::*;
use cgmath::num_traits::Zero;
use std::ops::Add;

use graphics::renderers::TerrainRenderer;
use graphics::renderers::UnitRenderer;

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

use game::logic::factions::is_enemy;

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
    pub display_event_clock: GameEventClock,
    pub realtime_clock: f64,
    pub last_realtime_clock: f64,
    pub time_within_event: f64,
    pub fract_within_event: f64,
    pub event_clock_last_advanced: f64,
    pub last_time: Instant,
    pub last_update_time: Instant,
    pub player_faction: Entity,
    pub minimum_active_event_clock: GameEventClock,
    event_start_times: Vec<f64>,
    pub victory: Option<bool>,
    animation_elements: Vec<AnimationElementWrapper>,
    gui: TacticalGui,
    world_view: WorldView
}

impl TacticalMode {
    pub fn new(gui: &mut GUI, world: &World, player_faction : Entity) -> TacticalMode {
        let tile_radius = 35.5;
        let mut camera = Camera2d::new();
        camera.move_speed = tile_radius * 20.0;
        camera.zoom = 1.0;

        TacticalMode {
            world_view: world.view_at_time(0),
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
            display_event_clock: 0,
            last_realtime_clock: 0.0,
            realtime_clock: 0.0,
            time_within_event: 0.0,
            fract_within_event: 0.0,
            event_clock_last_advanced: 0.0,
            last_time: Instant::now(),
            last_update_time: Instant::now(),
            player_faction,
            minimum_active_event_clock: 0,
            event_start_times: vec![0.0],
            victory: None,
            animation_elements: vec![],
            gui: TacticalGui::new(gui),
        }
    }

    pub fn selected_player_character(&self, world: &WorldView) -> Option<Entity> {
        if let Some(sel_ref) = self.selected_character {
            if world.character(sel_ref).faction == self.player_faction {
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

    pub fn path(&self, from: AxialCoord, to: AxialCoord) -> Option<(Vec<AxialCoord>, Cost)> {
        astar(&from, |c| c.neighbors().into_iter().map(|c| (c, Cost::new(1.0))), |c| Cost(c.distance(&to)), |c| *c == to)
    }

    fn blocking_animations_active(&self) -> bool {
        self.animation_elements.iter().any(|e| e.blocking_pcnt_elapsed(self.realtime_clock) < 1.0)
    }

    pub fn advance_event_clock(&mut self, max_event_clock: GameEventClock) {
        let dt = Instant::now().duration_since(self.last_time).subsec_nanos() as f64 / 1e9f64;
        //        println!("Advancing event clock by {}", dt);
        self.last_time = Instant::now();
        self.last_realtime_clock = self.realtime_clock;
        self.realtime_clock += dt;

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
            }
        }
    }

    pub fn at_latest_event(&self, world: &World) -> bool {
        self.display_event_clock >= world.current_time - 1
    }

    fn event_started_at(&self, gec: GameEventClock) -> f64 {
        *self.event_start_times.get(gec as usize).unwrap()
    }

    fn ai_action(&mut self, ai_ref: &Entity, cdata: &CharacterData, world: &mut World, world_view: &WorldView, _all_characters: &Vec<Entity>) {
        let ai = world_view.character(*ai_ref);

        let closest_enemy = world_view.entities_with_data::<CharacterData>().iter()
            .filter(|&(_, c)| c.is_alive())
            .filter(|&(cref, _)| is_enemy(world_view, *ai_ref, *cref))
            .min_by_key(|t| world_view.data::<PositionData>(*t.0).hex.distance(&ai.position.hex));

        if let Some(closest) = closest_enemy {
            let enemy_ref: &Entity = closest.0;
            let enemy_data = world_view.character(*closest.0);

            if enemy_data.position.distance(&ai.position) >= r32(1.5) {
                if let Some(path) = path_any_v(world_view, *ai_ref, ai.position.hex, &enemy_data.position.hex.neighbors(), enemy_data.position.hex) {
                    movement::handle_move(world, *ai_ref, path.0.as_slice())
                } else {
                    println!("No move towards closest enemy, stalling");
                }
            }

            if enemy_data.position.distance(&ai.position) < r32(1.5) {
                let all_possible_attacks = possible_attacks(world_view, *ai_ref);
                if let Some(attack) = all_possible_attacks.first() {
                    logic::combat::handle_attack(world, *ai_ref, *enemy_ref, attack);
                }
            }
        } else {
            if let Some(path) = self.path(ai.position.hex, ai.position.hex.neighbor(0)) {
                logic::movement::handle_move(world, *ai_ref, path.0.as_slice());
            }
        }
    }

    fn end_turn(&mut self, world: &mut World) {
        let world_view = world.view();
        let current_turn = world_view.world_data::<TurnData>().turn_number;

        let mut prev_faction = self.player_faction;

        for (faction, faction_data) in world_view.entities_with_data::<FactionData>() {
            if faction != &self.player_faction {
                SetActiveFactionMod(*faction).apply_to_world(world);
                world.add_event(GameEvent::FactionTurnEnd { turn_number : current_turn, faction : prev_faction });
                world.add_event(GameEvent::FactionTurnStart { turn_number : current_turn, faction : *faction });

                prev_faction = *faction;

                let character_refs: Vec<Entity> = world.view().entities_with_data::<CharacterData>().keys().cloned().collect::<Vec<Entity>>();

                for (cref, cur_data) in world_view.entities_with_data::<CharacterData>() {
                    if &cur_data.faction == faction && cur_data.is_alive() {
                        // these are enemies, now we get to decide what they want to do
                        self.ai_action(&cref, &cur_data, world, world_view, &character_refs);
                    }
                }
            }
        }

        // recompute the character set, some may have been created, hypothetically
        let character_refs = world_view.entities_with_data::<CharacterData>().keys();

        for cref in character_refs.clone() {
            modify(world, *cref, ResetCharacterTurnMod);
        }

        let turn_number = current_turn + 1;
        SetTurnNumberMod(turn_number).apply_to_world(world);

        world.add_event(GameEvent::TurnStart { turn_number });

        // back to the player's turn
        SetActiveFactionMod(self.player_faction).apply_to_world(world);
        world.add_event(GameEvent::FactionTurnEnd { turn_number : current_turn, faction : prev_faction });
        world.add_event(GameEvent::FactionTurnStart { turn_number : current_turn, faction : self.player_faction });

        let mut living_enemy = false;
        let mut living_ally = false;
        for cref in character_refs.clone() {
            let char_data = world_view.data::<CharacterData>(*cref);
            if char_data.is_alive() {
                if char_data.faction != self.player_faction {
                    living_enemy = true;
                } else {
                    living_ally = true;
                }
            }
        }
        if !living_enemy {
            self.victory = Some(true);
        } else if !living_ally {
            self.victory = Some(false);
        }
    }


    fn add_animation_element(&mut self, elem: Box<AnimationElement>) {
        self.animation_elements.push(AnimationElementWrapper { animation: elem, start_time: None });
    }

    fn render_draw_list(&self, draw_list: DrawList, g: &mut GraphicsWrapper) {
        for mut quad in draw_list.quads {
            quad.offset *= self.tile_radius;
            quad.size = match quad.size {
                Some(specific_size) => Some(specific_size * self.tile_radius),
                None => None
            };
            g.draw_quad(quad);
        }
        for mut text in draw_list.text {
            text.offset *= self.tile_radius;
            g.draw_text(text);
        }
    }

    fn create_animations_if_necessary(&mut self, world_in: &World, world_view: &WorldView) {
        if !self.blocking_animations_active() && !self.at_latest_event(world_in) {
            if let Some(new_event) = world_in.event_at(self.display_event_clock + 1) {
                trace!("Advanced event, new event is {:?}", new_event);
                for elem in tactical_event_handler::animation_elements_for_new_event(&world_view, new_event) {
                    self.add_animation_element(elem)
                }
            };
        }
    }

    fn create_move_ui_draw_list(&mut self, world_in: &mut World) -> DrawList {
        let is_animating = !self.at_latest_event(world_in);
        if is_animating {
            DrawList::none()
        } else {
            if let Some(hovered_tile) = self.hovered_tile(world_in.view()) {
                let hovered_hex = hovered_tile.position;

                let mut draw_list = DrawList::of_quad(Quad::new_cart(String::from("ui/hoverHex"), hovered_hex.as_cart_vec()).centered());

                if let Some(selected) = self.selected_character {
                    let sel_c = world_in.view().character(selected);
                    if let Some(path_result) = self.path(sel_c.position.hex, hovered_hex) {
                        let path = path_result.0;
                        for hex in path {
                            draw_list = draw_list.add_quad(Quad::new_cart(String::from("ui/feet"), hex.as_cart_vec()).centered());
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

    fn current_game_state(&self, world: &World) -> tactical_gui::GameState {
        tactical_gui::GameState {
            display_event_clock: self.display_event_clock,
            selected_character: self.selected_character,
            victory: self.victory,
            player_faction: self.player_faction,
            hovered_hex_coord : AxialCoord::from_cartesian(&self.mouse_game_pos(), self.tile_radius),
            animating: !self.at_latest_event(world),
            mouse_pixel_pos: self.mouse_pos,
            mouse_game_pos: self.screen_pos_to_game_pos(self.mouse_pos)
        }
    }

    fn update_world_view(&mut self, world : &World) {
        world.update_view_to_time(&mut self.world_view, self.display_event_clock);
    }
}

impl GameMode for TacticalMode {
    fn enter(&mut self, world: &mut World) {

    }

    fn update(&mut self, _: &mut World, _: f64) {
        let dt = Instant::now().duration_since(self.last_update_time).subsec_nanos() as f64 / 1e9f64;
        self.last_update_time = Instant::now();
        self.camera.position = self.camera.position + self.camera.move_delta * (dt as f32) * self.camera.move_speed;
    }

    fn update_gui(&mut self, world: &mut World, ui: &mut GUI, frame_id: Option<Wid>) {
        self.gui.update_gui(world, ui, frame_id, self.current_game_state(world));
    }

    fn draw(&mut self, world_in: &mut World, g: &mut GraphicsWrapper) {
//        let active_events = self.active_events(world_in);
        let mut world_view = world_in.view_at_time(self.display_event_clock);

        self.create_animations_if_necessary(&world_in, &world_view);

        self.advance_event_clock(world_in.current_time);

        world_in.update_view_to_time(&mut world_view, self.display_event_clock);

        self.create_animations_if_necessary(&world_in, &world_view);

        if let Some(v) = g.context.viewport {
            self.viewport = v;
        }
        g.context.view = self.camera.matrix(self.viewport);

        // draw list of all of the drawing requests from all currently active animations
        let mut anim_draw_list = DrawList::default();

        let realtime_clock = self.realtime_clock;
        for elem in &mut self.animation_elements {
            elem.start_time = match elem.start_time {
                Some(existing) => Some(existing),
                None => Some(self.last_realtime_clock)
            };
            let pcnt = elem.pcnt_elapsed(realtime_clock).min(1.0);
            anim_draw_list.append(&mut elem.animation.draw(&mut world_view, pcnt));

            let blocking_pcnt = elem.blocking_pcnt_elapsed(realtime_clock);
            if blocking_pcnt < 1.0 {
                break;
            }
        }

        let min_game_pos = self.screen_pos_to_game_pos(v2(0.0,0.0));
        let max_game_pos = self.screen_pos_to_game_pos(v2(self.viewport.window_size[0] as f32, self.viewport.window_size[1] as f32));

        let culling_rect = Rect::from_corners(min_game_pos.x / self.tile_radius, max_game_pos.y / self.tile_radius, max_game_pos.x / self.tile_radius, min_game_pos.y / self.tile_radius);

        // draw list for the map tiles
        let terrain_draw_list = self.terrain_renderer.render_tiles(&world_view, self.display_event_clock, culling_rect);
        // draw list for the units and built-in unit UI elements
        let unit_draw_list = self.unit_renderer.render_units(&world_view, self.display_event_clock, self.selected_character);
        // draw list for hover hex and foot icons
        let movement_ui_draw_list = self.create_move_ui_draw_list(world_in);

        let ui_draw_list = self.gui.draw(&world_view, self.current_game_state(world_in));

        self.render_draw_list(terrain_draw_list, g);
        self.render_draw_list(ui_draw_list, g);
        self.render_draw_list(movement_ui_draw_list, g);
        self.render_draw_list(unit_draw_list, g);
        self.render_draw_list(anim_draw_list, g);

        let gui_camera = Camera2d::new();
        g.context.view = gui_camera.matrix(self.viewport);
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
                    let world_view = &self.world_view;
//                    let world_view = world.view_at_time(self.display_event_clock);

                    let found = character_at(&world_view, clicked_coord);
                    if let Some((found_ref, target_data)) = found {
                        match self.selected_character {
                            Some(prev_sel) if prev_sel == found_ref => self.selected_character = None,
                            Some(cur_sel) => {
                                let sel_data = world.view().data::<CharacterData>(cur_sel);
                                if target_data.faction != self.player_faction &&
                                    sel_data.faction == self.player_faction {
                                    if sel_data.can_act() {
                                        let all_possible_attacks = possible_attacks(&world_view, cur_sel);
                                        if let Some(attack) = all_possible_attacks.first() {
                                            logic::combat::handle_attack(world, cur_sel, found_ref, attack);
                                        } else {
                                            warn!("Cannot attack, there are no attacks to use!");
                                        }
                                    } else {
                                        warn!("Cannot attack, no actions remaining");
                                    }
                                } else {
                                    self.selected_character = Some(found_ref);
                                }
                            }
                            None => {
                                self.selected_character = Some(found_ref);
                            }
                        }
                    } else {
                        if let Some(sel_c) = self.selected_character {
                            if let Some(hovered_) = self.hovered_tile(world.view()) {
                                let cur_sel_data = world_view.character(sel_c);
                                if cur_sel_data.faction == self.player_faction {
                                    let start_pos = world_view.character(sel_c).position.hex;
                                    if let Some(path_result) = self.path(start_pos, clicked_coord) {
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
                    _ => ()
                }
            },
            UIEvent::KeyRelease { key } => {
                match key {
                    Key::Left => self.camera.move_delta.x = 0.0,
                    Key::Right => self.camera.move_delta.x = 0.0,
                    Key::Up => self.camera.move_delta.y = 0.0,
                    Key::Down => self.camera.move_delta.y = 0.0,
                    Key::Return => self.end_turn(world),
                    Key::Escape => self.selected_character = None,
                    _ => ()
                }
            },
            _ => ()
        }
    }


    fn on_event<'a, 'b, 'c>(&'a mut self, world: &'b mut World, gui: &'c mut GUI, event: &UIEvent) {
        gui.handle_ui_event_2(event, self, world);
    }
}
