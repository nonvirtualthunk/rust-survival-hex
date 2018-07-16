use game::World;
use game::world::WorldView;
use game::entities::*;
use game::world::EntityBuilder;

use noisy_float::prelude::*;
use itertools::Itertools;

use piston_window::*;
use common::prelude::*;

use core::GameMode;
use core::normalize_mouse;

use arx_graphics::core::GraphicsWrapper;
use arx_graphics::animation::AnimationElement;
use arx_graphics::interpolation::*;
use common::hex::*;
use common::color::Color;
use arx_graphics::core::Quad;
use arx_graphics::core::Text;
use arx_graphics::camera::*;
use piston_window::math;
use vecmath;
use game::core::*;
use game::events::*;
use game::actions;
use game::world::Entity;
use game::world_util::*;
use game::world::ConstantModifier;

use game::combat::*;

use pathfinding::prelude::astar;
use std::cmp::*;
use cgmath::num_traits::Zero;
use std::ops::Add;

use arx_graphics::renderers::TerrainRenderer;
use arx_graphics::renderers::UnitRenderer;

use tactical_gui::TacticalGui;
use tactical_gui;

use tactical_event_handler;

use conrod::*;
use conrod::widget::primitive::*;
use conrod;
use conrod::Widget;

use std::time::*;

use interpolation::*;
use arx_graphics::core::DrawList;
use arx_graphics::core::GraphicsResources;

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
    pub mouse_pos: [f64; 2],
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
    pub player_faction: Entity,
    pub minimum_active_event_clock: GameEventClock,
    event_start_times: Vec<f64>,
    pub victory: Option<bool>,
    animation_elements: Vec<AnimationElementWrapper>,
    gui: TacticalGui,
}

impl TacticalMode {
    pub fn new() -> TacticalMode {
        let tile_radius = 32.0;
        let mut camera = Camera2d::new();
        camera.move_speed = tile_radius * 20.0;
        camera.zoom = 1.0;

        TacticalMode {
            selected_character: None,
            tile_radius,
            mouse_pos: [0.0, 0.0],
            camera,
            viewport: Viewport {
                window_size: [256, 256],
                draw_size: [256, 256],
                rect: [0, 0, 256, 256],
            },
            terrain_renderer: TerrainRenderer {},
            unit_renderer: UnitRenderer {},
            display_event_clock: 0,
            last_realtime_clock: 0.0,
            realtime_clock: 0.0,
            time_within_event: 0.0,
            fract_within_event: 0.0,
            event_clock_last_advanced: 0.0,
            last_time: Instant::now(),
            player_faction: Entity::sentinel(),
            minimum_active_event_clock: 0,
            event_start_times: vec![0.0],
            victory: None,
            animation_elements: vec![],
            gui: TacticalGui::new(),
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

    pub fn mouse_game_pos(&self) -> Vec2f {
        let camera_mat = self.camera.matrix(self.viewport);
        let inverse_mat = vecmath::mat2x3_inv(camera_mat);
        let norm_mouse = normalize_mouse(self.mouse_pos, &self.viewport);
        let transformed_mouse = math::transform_pos(inverse_mat, norm_mouse);
        let mouse_pos = v2(transformed_mouse[0] as f32, transformed_mouse[1] as f32);
        mouse_pos
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

    pub fn blocking_time_for_event(&self, event: GameEvent) -> f64 {
        match event {
            GameEvent::Attack { .. } => 0.75,
            _ => self.time_for_event(event)
        }
    }

    pub fn time_for_event(&self, event: GameEvent) -> f64 {
        match event {
            GameEvent::WorldStart => 0.0,
            GameEvent::Move { .. } => 0.3,
            GameEvent::Attack { .. } => 3.0,
            GameEvent::Equip { .. } => 0.0,
            GameEvent::TurnStart { .. } => 0.0
        }
    }

    fn blocking_animations_active(&self) -> bool {
        self.animation_elements.iter().any(|e| e.blocking_pcnt_elapsed(self.realtime_clock) < 1.0)
    }

    pub fn advance_event_clock(&mut self, max_event_clock: GameEventClock) {
        let dt = Instant::now().duration_since(self.last_time).subsec_nanos() as f64 / 1e9f64;
        //        println!("Advancing event clock by {}", dt);
        self.last_time = Instant::now();
        self.camera.position = self.camera.position + self.camera.move_delta * (dt as f32) * self.camera.move_speed;
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

//        let duration = match next_event {
//            Some(evt) => self.blocking_time_for_event(evt),
//            None => 0.000001
//        };
//        let full_duration = match next_event {
//            Some(evt) => self.time_for_event(evt),
//            None => 0.00001
//        };
//        if self.display_event_clock < max_event_clock - 1 {
//            if self.event_start_times.get((self.display_event_clock + 1) as usize).is_none() {
//                self.event_start_times.push(self.realtime_clock);
//            }
//
//            self.time_within_event += dt;
//            let advanced = if self.time_within_event > duration {
//                //                println!("Advancing, realtime: {}, last_advanced: {}, duration: {}", self.realtime_clock, self.event_clock_last_advanced, duration);
//                self.display_event_clock += 1;
//                self.event_clock_last_advanced = self.realtime_clock;
//                self.time_within_event = 0.0;
//                true
//            } else {
//                false
//            };
//            self.fract_within_event = (self.time_within_event / full_duration).min(1.0).max(0.0);
//            //            println!("Fract within event: {:?}", self.fract_within_event);
//            advanced
//        } else {
//            false
//        }
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
            .filter(|&(_, c)| c.faction != ai.faction)
            .min_by_key(|t| t.1.position.distance(&ai.position));

        if let Some(closest) = closest_enemy {
            let enemy_ref: &Entity = closest.0;
            let enemy_data: &CharacterData = closest.1;

            if enemy_data.position.distance(&ai.position) >= r32(1.5) {
                if let Some(path) = path_any_v(world_view, ai.position, &enemy_data.position.neighbors(), enemy_data.position) {
                    actions::handle_move(world, *ai_ref, path.0.as_slice())
                } else {
                    println!("No move towards closest enemy, stalling");
                }
            }

            if enemy_data.position.distance(&ai.position) < r32(1.5) {
                let all_possible_attacks = possible_attacks(world_view, *ai_ref);
                if let Some(attack) = all_possible_attacks.first() {
                    actions::handle_attack(world, *ai_ref, *enemy_ref, attack);
                }
            }
        } else {
            if let Some(path) = self.path(cdata.position, cdata.position.neighbor(0)) {
                actions::handle_move(world, *ai_ref, path.0.as_slice());
            }
        }
    }

    fn end_turn(&mut self, world: &mut World) {
        let character_refs: Vec<Entity> = world.view().entities_with_data::<CharacterData>().keys().cloned().collect::<Vec<Entity>>();
        let world_view = world.view();

        for (cref, cur_data) in world_view.entities_with_data::<CharacterData>() {
            if cur_data.faction != self.player_faction && cur_data.is_alive() {
                // these are enemies, now we get to decide what they want to do
                self.ai_action(&cref, &cur_data, world, world_view, &character_refs);
            }
        }

        // recompute the character set, some may have been created, hypothetically
        let character_refs = world_view.entities_with_data::<CharacterData>().keys();

        for cref in character_refs.clone() {
            modify(world, *cref, ResetCharacterTurnMod);
        }

        let turn_number = world_view.world_data::<TimeData>().turn_number + 1;
        SetTurnNumberMod(turn_number).apply_to_world(world);

        world.add_event(GameEvent::TurnStart { turn_number });

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
            let hovered_hex = AxialCoord::from_cartesian(&self.mouse_game_pos(), self.tile_radius);

            let mut draw_list = DrawList::of_quad(Quad::new_cart(String::from("ui/hoverHex"), hovered_hex.as_cart_vec()).centered());

            if let Some(selected) = self.selected_character {
                let sel_c = world_in.view().data::<CharacterData>(selected);
                if let Some(path_result) = self.path(sel_c.position, hovered_hex) {
                    let path = path_result.0;
                    for hex in path {
                        draw_list = draw_list.add_quad(Quad::new_cart(String::from("ui/feet"), hex.as_cart_vec()).centered());
                    }
                }
            }

            draw_list
        }
    }
}

impl GameMode for TacticalMode {
    fn enter(&mut self, world: &mut World) {
        // -------- entity data --------------
        world.register::<TileData>();
        world.register::<CharacterData>();
        world.register::<CombatData>();
        world.register::<InventoryData>();
        world.register::<SkillData>();
        world.register::<ItemData>();
        world.register::<FactionData>();
        // -------- world data ---------------
        world.register::<MapData>();
        world.register::<TimeData>();

        world.register_index::<AxialCoord>();

        world.attach_world_data(&MapData {
            min_tile_bound: AxialCoord::new(-10, -10),
            max_tile_bound: AxialCoord::new(10, 10),
        });
        world.attach_world_data(&TimeData {
            turn_number: 0
        });

        for x in -10..10 {
            for y in -10..10 {
                let coord = AxialCoord::new(x, y);
                let tile = EntityBuilder::new()
                    .with(TileData {
                        position: coord,
                        name: "grass",
                        move_cost: Oct::of(1),
                        cover: 0.0,
                    }).create(world);
                world.index_entity(tile, coord);
            }
        }

        let player_faction = EntityBuilder::new()
            .with(FactionData {
                name: String::from("Player"),
                color: Color::new(1.1, 0.3, 0.3, 1.0),
            }).create(world);
        self.player_faction = player_faction;

        let enemy_faction = EntityBuilder::new()
            .with(FactionData {
                name: String::from("Enemy"),
                color: Color::new(0.3, 0.3, 0.9, 1.0),

            }).create(world);


        let bow = EntityBuilder::new()
            .with(ItemData {
                primary_attack: Some(Attack {
                    ap_cost: 4,
                    damage_dice: DicePool {
                        die: 8,
                        count: 2,
                    },
                    damage_bonus: 1,
                    relative_accuracy: 0.5,
                    primary_damage_type: DamageType::Piercing,
                    secondary_damage_type: None,
                    range: 10,
                    min_range: 2,
                }),
                ..Default::default()
            }).create(world);

        let archer = EntityBuilder::new()
            .with(CharacterData {
                faction: player_faction,
                position: AxialCoord::new(0, 0),
                sprite: String::from("elf/archer"),
                name: String::from("Archer"),
                move_speed: Oct::of_parts(1, 2), // one and 2 eights
                health: Reduceable::new(25),
                ..Default::default()
            })
            .with(CombatData::default())
            .with(SkillData::default())
            .with(InventoryData::default())
            .create(world);

        actions::equip_item(world, archer, bow);

        let create_monster_at = |world_in: &mut World, pos: AxialCoord| {
            EntityBuilder::new()
                .with(CharacterData {
                    faction: enemy_faction,
                    position: pos,
                    sprite: String::from("void/monster"),
                    name: String::from("Monster"),
                    move_speed: Oct::of_rounded(0.75),
                    action_points: Reduceable::new(6),
                    health: Reduceable::new(22),
                    ..Default::default()
                })
                .with(CombatData {
                    natural_attacks: vec![Attack {
                        damage_dice: DicePool {
                            count: 1,
                            die: 4,
                        },
                        ..Default::default()
                    }],
                    ..Default::default()
                })
                .with(SkillData::default())
                .with(InventoryData::default())
                .create(world_in);
        };

        create_monster_at(world, AxialCoord::new(4, 0));
        create_monster_at(world, AxialCoord::new(0, 4));

        world.add_event(GameEvent::WorldStart);
    }

    fn update(&mut self, _: &mut World, _: f64) {}

    fn update_gui_2(&mut self, world: &mut World, resources : &mut GraphicsResources) {

    }

    fn update_gui(&mut self, world: &mut World, ui: &mut conrod::UiCell, frame_id: conrod::widget::Id) {
        self.gui.draw_gui(world, ui, frame_id, tactical_gui::GameState {
            display_event_clock: self.display_event_clock,
            selected_character: self.selected_character,
            victory: self.victory,
        });
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

        // draw list for the map tiles
        let terrain_draw_list = self.terrain_renderer.render_tiles(&world_view, self.display_event_clock);
        // draw list for the units and built-in unit UI elements
        let unit_draw_list = self.unit_renderer.render_units(&world_view, self.display_event_clock, self.selected_character);
        // draw list for hover hex and foot icons
        let movement_ui_draw_list = self.create_move_ui_draw_list(world_in);

        self.render_draw_list(terrain_draw_list, g);
        self.render_draw_list(movement_ui_draw_list, g);
        self.render_draw_list(unit_draw_list, g);
        self.render_draw_list(anim_draw_list, g);

        let gui_camera = Camera2d::new();
//        gui_camera.zoom = 0.5;
        g.context.view = gui_camera.matrix(self.viewport);

        self.gui.draw(world_in, g)

//        g.draw_text(Text::new(String::from("This is some example text, iiiiiiiiiiiiiii"), 20).offset(v2(0.0,0.0)).font("NotoSerif-Regular.ttf"));
    }

    fn on_event(&mut self, world: &mut World, event: conrod::event::Widget) {
        use conrod::event::Widget;

        match event {
            Widget::Motion(motion) => match motion.motion {
                conrod::input::Motion::MouseCursor { x, y } => {
                    self.mouse_pos = [x + self.viewport.window_size[0] as f64 / 2.0, self.viewport.window_size[1] as f64 / 2.0 - y];
                }
                _ => ()
            },
            Widget::Click(click) if self.at_latest_event(world) => match click.button {
                conrod::input::MouseButton::Left => {
                    let mouse_pos = self.mouse_game_pos();
                    let clicked_coord = AxialCoord::from_cartesian(&mouse_pos, self.tile_radius);

                    let world_view = world.view_at_time(self.display_event_clock);

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
                                            actions::handle_attack(world, cur_sel, found_ref, attack);
                                        } else {
                                            println!("Cannot attack, there are no attacks to use!");
                                        }
                                    } else {
                                        println!("Cannot attack, no actions remaining");
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
                            let cur_sel_data = world_view.character(sel_c);
                            if cur_sel_data.faction == self.player_faction {
                                let start_pos = world_view.character(sel_c).position;
                                if let Some(path_result) = self.path(start_pos, clicked_coord) {
                                    let path = path_result.0;
                                    actions::handle_move(world, sel_c, path.as_slice());
                                }
                            }
                        }
                    }
                }
                _ => ()
            },
            Widget::Press(press) => {
                match press.button {
                    conrod::event::Button::Keyboard(key) => {
                        use conrod::input::Key;

                        match key {
                            Key::Left => self.camera.move_delta.x = -1.0,
                            Key::Right => self.camera.move_delta.x = 1.0,
                            Key::Up => self.camera.move_delta.y = 1.0,
                            Key::Down => self.camera.move_delta.y = -1.0,
                            Key::Z => self.camera.zoom += 0.1,
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
            Widget::Release(release) => {
                match release.button {
                    conrod::event::Button::Keyboard(key) => {
                        use conrod::input::Key;

                        match key {
                            Key::Left => self.camera.move_delta.x = 0.0,
                            Key::Right => self.camera.move_delta.x = 0.0,
                            Key::Up => self.camera.move_delta.y = 0.0,
                            Key::Down => self.camera.move_delta.y = 0.0,
                            Key::Return => self.end_turn(world),
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
            _ => ()
        }
    }
}
