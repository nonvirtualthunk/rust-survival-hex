#![allow(unused_imports)]
#![allow(where_clauses_object_safety)]

extern crate piston;
extern crate piston_window;
extern crate gfx_device_gl;
extern crate find_folder;
extern crate gfx_graphics;
extern crate gfx;
extern crate cgmath;
#[macro_use]
extern crate lazy_static;
extern crate image;
extern crate vecmath;
extern crate pathfinding;
extern crate conrod;
extern crate arx_graphics;
extern crate arx_common as common;
extern crate samvival_control as control;
extern crate samvival_game as game;
extern crate arx_gui as gui;
extern crate opengl_graphics;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate graphics;

//#![allow(dead_code)]

use piston_window::*;
use piston::input::keyboard::ModifierKey;
use opengl_graphics::{GlGraphics, OpenGL};

mod tmp;

use game::World;
use game::entities::TileData;
use common::hex::*;
use arx_graphics::core::GraphicsWrapper;
use control::core::Game;
use gui::Modifiers;
use piston::input::GenericEvent;

//use arx_graphics::core::Context as ArxContext;
use arx_graphics::core::GraphicsResources;

use control::GameMode;
use control::tactical::TacticalMode;

mod ui_playground;


pub fn theme() -> conrod::Theme {
    use conrod::position::{Align, Direction, Padding, Position, Relative};
    conrod::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 5.0), None),
        background_color: conrod::color::DARK_CHARCOAL,
        shape_color: conrod::color::LIGHT_CHARCOAL,
        border_color: conrod::color::BLACK,
        border_width: 1.0,
        label_color: conrod::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}

use std::env;

fn main() {
    pretty_env_logger::init();
    info!("Init!");

    for arg in env::args() {
        info!("Argument: {}", arg);
    }

    if env::args().find(|a| a == "gui_playground").is_some() {
        ui_playground::run();
        return;
    }

    let mut window: PistonWindow = WindowSettings::new(
        "survival-hex",
        [1440, 900],
    )
        .vsync(true)
        .build()
        .unwrap();


    let mut game = Game::new(window.factory.clone());
    game.on_load(&mut window);

    game.active_mode.enter(&mut game.world);

    while let Some(e) = window.next() {
        if let Some(render_args) = e.render_args() {
            let adjusted_viewport = render_args.viewport();
            game.resources.assets.dpi_scale = adjusted_viewport.draw_size[0] as f32 / adjusted_viewport.window_size[0] as f32;

            window.window.make_current();
            window.g2d.draw(
                &mut window.encoder,
                &window.output_color,
                &window.output_stencil,
                adjusted_viewport,
                |c, g| {
                    game.on_draw(c, g);
                },
            );
            window.encoder.flush(&mut window.device);
        }

        if let Some(btn) = e.button_args() {
            if btn.state == ButtonState::Press && btn.button == Button::Keyboard(Key::Q) && game.gui.active_modifiers().ctrl {
                break;
            }
        }

        if let Some(upd) = e.update_args() {
            game.on_update(upd);
        }

        game.on_event(&e);
    }
}