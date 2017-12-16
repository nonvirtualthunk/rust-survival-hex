use game::World;
use game::entities::*;

use piston_window::*;
use common::prelude::*;

use core::GameMode;

use arx_graphics::core::GraphicsWrapper;
use common::hex::*;
use arx_graphics::core::Quad;
use arx_graphics::camera::*;
use piston_window::math;
use vecmath;
use game::core::*;
use game::events::*;

use pathfinding::astar;
use std::cmp::*;
use cgmath::num_traits::Zero;
use std::ops::Add;

use conrod::widget::Id as Wid;

use arx_graphics::renderers::TerrainRenderer;

use conrod::*;
use conrod::widget::primitive::*;
use conrod;
use conrod::Widget;

use std::time::*;

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Cost(pub f32);

impl Eq for Cost {}

impl Ord for Cost {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Zero for Cost {
    fn zero() -> Self {
        Cost(0.0f32)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0.0f32
    }
}

impl Add<Cost> for Cost {
    type Output = Cost;

    fn add(self, rhs: Cost) -> Self::Output {
        Cost(self.0 + rhs.0)
    }
}


pub struct TacticalMode {
    selected_character: Option<CharacterRef>,
    tile_radius: f32,
    mouse_pos: [f64; 2],
    camera: Camera2d,
    viewport: Viewport,
    terrain_renderer: TerrainRenderer,
    widgets: Widgets,
    display_event_clock: GameEventClock,
    realtime_clock: f64,
    event_clock_last_advanced: f64,
    last_time : Instant
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
            widgets: Default::default(),
            display_event_clock: 0,
            realtime_clock: 0.0,
            event_clock_last_advanced: 0.0,
            last_time: Instant::now()
        }
    }
}

impl TacticalMode {
    pub fn normalize_mouse(mouse: [f64; 2], viewport: &Viewport) -> [f64; 2] {
        let in_x = mouse[0];
        let in_y = viewport.window_size[1] as f64 - mouse[1] - 1.0;

        let centered_x = in_x - (viewport.window_size[0] / 2) as f64;
        let centered_y = in_y - (viewport.window_size[1] / 2) as f64;

        let norm_x = centered_x / viewport.window_size[0] as f64;
        let norm_y = centered_y / viewport.window_size[1] as f64;

        let scale_factor = viewport.draw_size[0] as f64 / viewport.window_size[0] as f64;

        let scaled_x = norm_x * scale_factor;
        let scaled_y = norm_y * scale_factor;

        [scaled_x, scaled_y]
    }

    pub fn mouse_game_pos(&self) -> Vec2f {
        let camera_mat = self.camera.matrix(self.viewport);
        let inverse_mat = vecmath::mat2x3_inv(camera_mat);
        let norm_mouse = TacticalMode::normalize_mouse(self.mouse_pos, &self.viewport);
        let transformed_mouse = math::transform_pos(inverse_mat, norm_mouse);
        let mouse_pos = v2(transformed_mouse[0] as f32, transformed_mouse[1] as f32);
        mouse_pos
    }

    pub fn quad(&self, texture_identifier: String, pos: AxialCoord) -> Quad {
        Quad::new(texture_identifier, pos.as_cartesian(self.tile_radius)).centered()
    }

    pub fn path(&self, from: AxialCoord, to: AxialCoord) -> Option<(Vec<AxialCoord>, Cost)> {
        astar(&from, |c| c.neighbors().into_iter().map(|c| (c, Cost(1f32))), |c| Cost(c.distance(&to)), |c| *c == to)
    }

    pub fn advance_event_clock(&mut self, max_event_clock : GameEventClock) {
        let dt = Instant::now().duration_since(self.last_time).subsec_nanos() as f64 / 1e9f64;
        self.last_time = Instant::now();
        self.camera.position = self.camera.position + self.camera.move_delta * (dt as f32) * self.camera.move_speed;
        self.realtime_clock += dt;
        if self.realtime_clock - self.event_clock_last_advanced > 1.0 && max_event_clock > self.display_event_clock {
            self.last_time = Instant::now();
            self.display_event_clock += 1;
            self.event_clock_last_advanced = self.realtime_clock;
        }
    }
}

impl GameMode for TacticalMode {
    fn update(&mut self, _: &mut World, _: f64) {
    }

    fn draw(&mut self, world_in: &mut World, g: &mut GraphicsWrapper) {
        self.advance_event_clock(world_in.event_clock);

        let world_view = world_in.view_at_time(self.display_event_clock);

        if let Some(v) = g.context.viewport {
            self.viewport = v;
        }
        g.context.view = self.camera.matrix(self.viewport);

        self.terrain_renderer.render_tiles(&world_view, g, self.display_event_clock);

        let hovered_hex = AxialCoord::from_cartesian(&self.mouse_game_pos(), self.tile_radius);

        g.draw_quad(self.quad(String::from("ui/hoverHex"), hovered_hex));


        if let Some(selected) = self.selected_character {
            let sel_c = world_view.character(selected);
            g.draw_quad(self.quad(String::from("ui/selectedTop"), sel_c.position));

            if let Some(path_result) = self.path(sel_c.position, hovered_hex) {
                let path = path_result.0;
                for hex in path {
                    g.draw_quad(self.quad(String::from("ui/feet"), hex));
                }
            }
        }

        for c in world_view.characters.values() {
            let cartesian_pos = c.position.as_cartesian(self.tile_radius);
            let quad = Quad::new(format!("entities/{}", c.sprite), cartesian_pos).centered();
            g.draw_quad(quad);
        }

        if let Some(selected) = self.selected_character {
            let sel_c = world_view.character(selected);
            g.draw_quad(self.quad(String::from("ui/selectedBottom"), sel_c.position));
        }
    }

    fn on_event(&mut self, world: &mut World, event: conrod::event::Widget) {
        use conrod::event::Widget;
        match event {
            Widget::Motion(motion) => match motion.motion {
                conrod::input::Motion::MouseCursor {x,y} => {
                    self.mouse_pos = [x + self.viewport.window_size[0] as f64/2.0, self.viewport.window_size[1] as f64/2.0 - y];
                },
                _ => ()
            },
            Widget::Click(click) => match click.button {
                conrod::input::MouseButton::Left => {
                    let mouse_pos = self.mouse_game_pos();
                    let clicked_coord = AxialCoord::from_cartesian(&mouse_pos, self.tile_radius);

                    let mut found = false;
                    for raw_c in world.characters() {
                        let c: CharacterData = raw_c.data(world);

                        if c.position == clicked_coord {
                            match self.selected_character {
                                Some(prev_sel) if prev_sel == raw_c.as_ref() => self.selected_character = None,
                                _ => self.selected_character = Some(raw_c.as_ref())
                            }

                            found = true;
                            break;
                        }
                    }
                    if !found {
                        if let Some(sel_c) = self.selected_character {
                            let start_pos = world.character(sel_c).position;
                            if let Some(path_result) = self.path(start_pos, clicked_coord) {
                                let path = path_result.0;
                                let mut prev_hex = start_pos;
                                for hex in path {
                                    if hex != start_pos {
                                        // advance the event clock
                                        world.add_event(GameEvent::Move { character : sel_c, from : prev_hex, to : hex });
                                        // now do the things that happen on that clock tick
                                        let hex_cost = world.tile(&hex).move_cost as f64;

                                        world.add_character_modifier(sel_c, character_modifier(move |c| c.moves.reduce_by(hex_cost)));
                                        world.add_character_modifier(sel_c, character_modifier(move |c| c.position = hex));

                                        prev_hex = hex;
                                    }
                                }
                            }
                        }
                    }
                },
                _ => ()
            },
            _ => ()
        }
//        if let Some(but) = event.button_args() {}
//        if let Some(mouse) = event.mouse_cursor_args() {
//            self.mouse_pos = mouse;
//        }
//        if let Some(button_pressed) = event.press_args() {
//            match button_pressed {
//                Button::Mouse(mouse_button) => {
//                    let mouse_pos = self.mouse_game_pos();
//                    let clicked_coord = AxialCoord::from_cartesian(&mouse_pos, self.tile_radius);
//
//                    let mut found = false;
//                    for raw_c in world.characters() {
//                        let c: CharacterData = raw_c.data(world);
//
//                        if c.position == clicked_coord {
//                            match self.selected_character {
//                                Some(prev_sel) if prev_sel == raw_c.as_ref() => self.selected_character = None,
//                                _ => self.selected_character = Some(raw_c.as_ref())
//                            }
//
//                            found = true;
//                            break;
//                        }
//                    }
//                    if !found {
//                        if let Some(sel_c) = self.selected_character {
//                            let start_pos = world.character(sel_c).position;
//                            if let Some(path_result) = self.path(start_pos, clicked_coord) {
//                                let path = path_result.0;
//                                for hex in path {
//                                    if hex != start_pos {
//                                        let hex_cost = world.tile(&hex).move_cost as f32;
//                                        let char_d = world.character_mut(sel_c).raw_data();
//                                        char_d.moves.reduce_by(hex_cost);
//                                        char_d.position = hex;
//                                    }
//                                }
//                            }
//                        }
//                    }
//                }
//                Button::Keyboard(key) => {
//                    match key {
//                        Key::Left => self.camera.move_delta.x = -1.0,
//                        Key::Right => self.camera.move_delta.x = 1.0,
//                        Key::Up => self.camera.move_delta.y = 1.0,
//                        Key::Down => self.camera.move_delta.y = -1.0,
//                        _ => ()
//                    }
//                }
//                _ => ()
//            }
//        }
//        if let Some(button_released) = event.release_args() {
//            match button_released {
//                Button::Keyboard(key) => {
//                    match key {
//                        Key::Left => self.camera.move_delta.x = 0.0,
//                        Key::Right => self.camera.move_delta.x = 0.0,
//                        Key::Up => self.camera.move_delta.y = 0.0,
//                        Key::Down => self.camera.move_delta.y = 0.0,
//                        _ => ()
//                    }
//                }
//                _ => ()
//            }
//        }
    }
    fn enter(&mut self, world: &mut World) {
        for x in -10..10 {
            for y in -10..10 {
                world.add_tile(Tile::new(TileData {
                    position: AxialCoord::new(x, y),
                    name: "grass",
                    move_cost: 1
                }));
            }
        }

        world.add_character(Character::new(CharacterData {
            position: AxialCoord::new(0, 0),
            sprite: String::from("elf/archer"),
            name: String::from("Archer"),
            moves: Reduceable::new(10.0),
            ..Default::default()
        }));
    }

    fn update_gui(&mut self, world: &mut World, ui: &mut conrod::UiCell) {
        self.widgets.init(&mut ui.widget_id_generator());


        if let Some(character_ref) = self.selected_character {
            let character = world.character_at_time(character_ref, self.display_event_clock);
            let main_widget = widget::Canvas::new()
                .pad(10.0)
                .scroll_kids_vertically()
                .top_right()
                .w(400.0)
                .h(800.0);

            main_widget.set(self.widgets.main_widget, ui);

            widget::Text::new(character.name.as_str())
                .font_size(20)
                .mid_top_of(self.widgets.main_widget)
                .parent(self.widgets.main_widget)
                .set(self.widgets.name_widget, ui);

            widget::RoundedRectangle::fill([72.0, 72.0], 5.0)
                .color(conrod::color::BLUE.alpha(1.0))
                .align_middle_x()
                .down_from(self.widgets.name_widget, 10.0)
                .parent(self.widgets.main_widget)
                .set(self.widgets.unit_icon, ui);

            widget::Text::new(format!("{} / {}", character.moves.cur_value().max(0.0), character.moves.max_value().max(0.0)).as_str())
                .font_size(16)
                .down_from(self.widgets.unit_icon, 20.0)
                .parent(self.widgets.main_widget)
                .set(self.widgets.moves_widget, ui);
        }
    }
}

#[derive(Default)]
pub struct Widgets {
    initialized: bool,
    main_widget: Wid,
    name_widget: Wid,
    unit_icon: Wid,
    moves_widget: Wid,

}

impl Widgets {
    pub fn init(&mut self, gen: &mut widget::id::Generator) {
        if !self.initialized {
            self.main_widget = gen.next();
            self.name_widget = gen.next();
            self.unit_icon = gen.next();
            self.moves_widget = gen.next();

            self.initialized = true;
        }
    }
}