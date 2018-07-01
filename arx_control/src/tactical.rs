use game::World;
use game::world::WorldView;
use game::entities::*;
use game::world::EntityBuilder;

use noisy_float::prelude::*;

use piston_window::*;
use common::prelude::*;

use core::GameMode;
use core::normalize_mouse;

use arx_graphics::core::GraphicsWrapper;
use arx_graphics::animation::AnimationElement;
use common::hex::*;
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

use pathfinding::prelude::astar;
use std::cmp::*;
use cgmath::num_traits::Zero;
use std::ops::Add;

use arx_graphics::renderers::TerrainRenderer;
use arx_graphics::renderers::UnitRenderer;

use tactical_gui::TacticalGui;
use tactical_gui;

use conrod::*;
use conrod::widget::primitive::*;
use conrod;
use conrod::Widget;

use std::time::*;

use interpolation::*;

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
    pub animation : Box<AnimationElement>,
    pub start_time : f64
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
    pub time_within_event: f64,
    pub fract_within_event: f64,
    pub event_clock_last_advanced: f64,
    pub last_time: Instant,
    pub player_faction: Entity,
    pub minimum_active_event_clock: GameEventClock,
    event_start_times: Vec<f64>,
    pub victory: Option<bool>,
    animation_elements: Vec<AnimationElementWrapper>,
    gui: TacticalGui
}

impl TacticalMode {
    pub fn new() -> TacticalMode {
        let tile_radius = 32.0;
        let mut camera = Camera2d::new();
        camera.move_speed = tile_radius * 20.0;

        TacticalMode {
            selected_character: None,
            tile_radius,
            mouse_pos: [0.0, 0.0],
            camera,
            viewport: Viewport {
                window_size: [256, 256],
                draw_size: [256, 256],
                rect: [0, 0, 256, 256]
            },
            terrain_renderer: TerrainRenderer {
                tile_radius
            },
            unit_renderer: UnitRenderer {
                tile_radius
            },
            display_event_clock: 0,
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
            gui: TacticalGui::new()
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
    pub fn colored_quad(&self, texture_identifier: String, pos: AxialCoord, color: [f32; 4]) -> Quad {
        Quad::new(texture_identifier, pos.as_cartesian(self.tile_radius)).centered().color(color)
    }
    pub fn colored_quad_cart(&self, texture_identifier: String, pos: Vec2f, color: [f32; 4]) -> Quad {
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

    pub fn advance_event_clock(&mut self, next_event: Option<GameEvent>, max_event_clock: GameEventClock) {
        let dt = Instant::now().duration_since(self.last_time).subsec_nanos() as f64 / 1e9f64;
        //        println!("Advancing event clock by {}", dt);
        self.last_time = Instant::now();
        self.camera.position = self.camera.position + self.camera.move_delta * (dt as f32) * self.camera.move_speed;
        self.realtime_clock += dt;
        let duration = match next_event {
            Some(evt) => self.blocking_time_for_event(evt),
            None => 0.000001
        };
        let full_duration = match next_event {
            Some(evt) => self.time_for_event(evt),
            None => 0.00001
        };
        if self.display_event_clock < max_event_clock - 1 {
            if self.event_start_times.get((self.display_event_clock + 1) as usize).is_none() {
                self.event_start_times.push(self.realtime_clock);
            }

            self.time_within_event += dt;
            if self.time_within_event > duration {
                //                println!("Advancing, realtime: {}, last_advanced: {}, duration: {}", self.realtime_clock, self.event_clock_last_advanced, duration);
                self.display_event_clock += 1;
                self.event_clock_last_advanced = self.realtime_clock;
                self.time_within_event = 0.0;
            }
            self.fract_within_event = (self.time_within_event / full_duration).min(1.0).max(0.0);
            //            println!("Fract within event: {:?}", self.fract_within_event);
        }
    }

    pub fn animating(&self, world: &World) -> bool {
        self.display_event_clock < world.current_time - 1
    }

    fn event_started_at(&self, gec: GameEventClock) -> f64 {
        *self.event_start_times.get(gec as usize).unwrap()
    }

    fn active_events(&mut self, world: &World) -> Vec<(GameEvent, f64, Option<f64>)> {
        let mut ret = vec![];

        // plus 2 because we ant to go up through display_event_clock + 1 inclusive
        for gec in self.minimum_active_event_clock..self.display_event_clock + 2 {
            if let Some(event) = world.event_at(gec) {
                let duration = self.time_for_event(event);

                let fract = if gec < self.display_event_clock + 1 {
                    let start_point = self.event_started_at(gec);
                    (self.realtime_clock - start_point) / duration
                } else {
                    self.fract_within_event
                };

                let blocking_fract = if gec < self.display_event_clock + 1 {
                    None
                } else {
                    let blocking_duration = self.blocking_time_for_event(event);
                    Some(self.time_within_event / blocking_duration)
                };

                if fract < 1.0 {
                    ret.push((event, fract, blocking_fract));
                } else {
                    // if the oldest event is past 1.0, go ahead and advance the minimum
                    if gec == self.minimum_active_event_clock {
                        self.minimum_active_event_clock = gec;
                    }
                }
            }
        }

        ret
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
                let all_possible_attacks = ai.possible_attacks(world_view);
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
}

impl GameMode for TacticalMode {
    fn update(&mut self, _: &mut World, _: f64) {}

    fn draw(&mut self, world_in: &mut World, g: &mut GraphicsWrapper) {
        let next_event = world_in.event_at(self.display_event_clock + 1);
        self.advance_event_clock(next_event, world_in.current_time);
        let active_events = self.active_events(world_in);

        let mut world_view = world_in.view_at_time(self.display_event_clock);


        if let Some(v) = g.context.viewport {
            self.viewport = v;
        }
        g.context.view = self.camera.matrix(self.viewport);

        // Perform any pre-emptive modification of the environment according to the event we're
        // currently animating (if any)
        for &(event, fract, blocking_fract) in &active_events {
            match event {
                GameEvent::Move { character, from, to, .. } => {
                    let from_pos = from.as_cartesian(self.tile_radius);
                    let to_pos = to.as_cartesian(self.tile_radius);
                    let pos = from_pos + (to_pos - from_pos) * fract as f32;
                    if fract > 1.0 { println!("Past 1"); }
                    let mut mover = world_view.data_mut::<CharacterData>(character);
                    mover.graphical_position = Some(pos);
                }
                GameEvent::Attack { defender, damage_done, attacker, .. } => {
                    if let Some(bfract) = blocking_fract {
                        let circ_fract: f32 = Ease::sine_in_out(bfract as f32);
                        let circ_fract = if circ_fract < 0.5 {
                            circ_fract * 2.0
                        } else {
                            1.0 - (circ_fract - 0.5) * 2.0
                        };

                        {
                            let defender = world_view.data_mut::<CharacterData>(defender);
                            //                    let end_alpha = if killing_blow { 0.0 } else { defender.graphical_color[3] };

                            if damage_done > 0 {
                                defender.health.reduce_by((damage_done as f64 * bfract) as i32);
                                let start_color = defender.graphical_color;
                                let end_color = [1f32, 0.1f32, 0.1f32, 1f32];
                                defender.graphical_color = lerp(&start_color, &end_color, &circ_fract);

                                //                            defender.graphical_color = (&[1.0f32, 1.0f32, 1.0f32, 1.0f32], &[1.0f32,0.1f32,0.1f32,1.0f32], bfract as f32);
                            }
                        }
                        let new_attacker_pos = {
                            use cgmath::InnerSpace;

                            let attacker = world_view.character(attacker);
                            let defender = world_view.character(defender);

                            let defender_pos = defender.position.as_cartesian(self.tile_radius);
                            let attacker_pos = attacker.position.as_cartesian(self.tile_radius);
                            let delta: Vec2f = (defender_pos - attacker_pos).normalize() * self.tile_radius;

                            attacker_pos + delta * circ_fract * 0.5
                        };
                        {
                            let attacker = world_view.data_mut::<CharacterData>(attacker);
                            attacker.graphical_position = Some(new_attacker_pos);
                        }

                        //                    let cur_alpha = defender.graphical_color[3];
                        //                    defender.graphical_color[3] = lerp(&cur_alpha, &end_alpha, &(fract as f32));
                    }
                }
                _ => ()
            }
        }

        let is_animating = self.animating(world_in);

        self.terrain_renderer.render_tiles(&world_view, g, self.display_event_clock);

        let hovered_hex = AxialCoord::from_cartesian(&self.mouse_game_pos(), self.tile_radius);

        if !is_animating {
            g.draw_quad(self.quad(String::from("ui/hoverHex"), hovered_hex));
        }


        if let Some(selected) = self.selected_character {
            let sel_c = world_in.view().data::<CharacterData>(selected);
            if let Some(path_result) = self.path(sel_c.position, hovered_hex) {
                if !is_animating {
                    let path = path_result.0;
                    for hex in path {
                        g.draw_quad(self.quad(String::from("ui/feet"), hex));
                    }
                }
            }
        }

        self.unit_renderer.render_units(&world_view, g, self.display_event_clock, self.selected_character);



        // perform any explicit drawing due to the events we're animating, if any
        for &(event, fract, _blocking_fract) in &active_events {
            match event {
                GameEvent::Attack { defender, damage_done, hit, .. } => {
                    let (msg, color) = if hit {
                        (format!("{}", damage_done), [0.9, 0.2, 0.2, 1.0 - fract.powf(2.0)])
                    } else {
                        (String::from("miss"), [0.1, 0.0, 0.0, 1.0 - fract.powf(2.0)])
                    };
                    let defender = world_view.character(defender);
                    let pos = defender.position.as_cartesian(self.tile_radius);
                    g.draw_text(Text::new(msg.as_str(), 20)
                        .offset(v2(pos.x, pos.y + self.tile_radius * 0.75 + fract as f32 * 15.0f32))
                        .colord(color));
                }
                _ => ()
            }
        }
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
            Widget::Click(click) if !self.animating(world) => match click.button {
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
                                        let all_possible_attacks = sel_data.possible_attacks(&world_view);
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

    fn enter(&mut self, world: &mut World) {
        // -------- entity data --------------
        world.register::<TileData>();
        world.register::<CharacterData>();
        world.register::<ItemData>();
        world.register::<FactionData>();
        // -------- world data ---------------
        world.register::<MapData>();
        world.register::<TimeData>();

        world.register_index::<AxialCoord>();

        world.attach_world_data(&MapData {
            min_tile_bound : AxialCoord::new(-10,-10),
            max_tile_bound : AxialCoord::new(10,10)
        });
        world.attach_world_data(&TimeData {
            turn_number : 0
        });

        for x in -10..10 {
            for y in -10..10 {
                let coord = AxialCoord::new(x, y);
                let tile = EntityBuilder::new()
                    .with(TileData {
                        position: coord,
                        name: "grass",
                        move_cost: 1,
                        cover: 0.0
                    }).create(world);
                world.index_entity(tile, coord);
            }
        }

        let player_faction = EntityBuilder::new()
            .with(FactionData {
                name: String::from("Player"),
                color: [1.1, 0.3, 0.3, 1.0]
            }).create(world);
        self.player_faction = player_faction;

        let enemy_faction = EntityBuilder::new()
            .with(FactionData {
                name: String::from("Enemy"),
                color: [0.3, 0.3, 0.9, 1.0],

            }).create(world);


        let bow = EntityBuilder::new()
            .with(ItemData {
                primary_attack: Some(Attack {
                    speed: 2.0,
                    damage_dice: DicePool {
                        die: 8,
                        count: 2
                    },
                    damage_bonus: 1,
                    relative_accuracy: 0.5,
                    primary_damage_type: DamageType::Piercing,
                    secondary_damage_type: None,
                    range: 10,
                    min_range: 2
                }),
                ..Default::default()
            }).create(world);

        let archer = EntityBuilder::new()
            .with(CharacterData {
                faction: player_faction,
                position: AxialCoord::new(0, 0),
                sprite: String::from("elf/archer"),
                name: String::from("Archer"),
                moves: Reduceable::new(10.0),
                health: Reduceable::new(25),
                ..Default::default()
            }).create(world);

        actions::equip_item(world, archer, bow);

        let create_monster_at = |world_in: &mut World, pos: AxialCoord| {
            EntityBuilder::new()
                .with(CharacterData {
                faction: enemy_faction,
                position: pos,
                sprite: String::from("void/monster"),
                name: String::from("Monster"),
                moves: Reduceable::new(5.0),
                health: Reduceable::new(22),
                natural_attacks: vec![Attack {
                    damage_dice: DicePool {
                        count: 1,
                        die: 4
                    },
                    ..Default::default()
                }],
                ..Default::default()
            }).create(world_in);
        };

        create_monster_at(world, AxialCoord::new(4, 0));
        create_monster_at(world, AxialCoord::new(0, 4));
    }

    fn update_gui(&mut self, world: &mut World, ui: &mut conrod::UiCell, frame_id: conrod::widget::Id) {
        self.gui.draw_gui(world, ui, frame_id, tactical_gui::GameState {
            display_event_clock: self.display_event_clock,
            selected_character: self.selected_character,
            victory: self.victory
        });
    }
}

