#![allow(unused_imports)]

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
extern crate arx_control as control;
extern crate arx_game as game;

//#![allow(dead_code)]

use piston_window::*;
use piston::input::keyboard::ModifierKey;

mod tmp;

use game::World;
use game::entities::TileData;
use common::hex::*;
use arx_graphics::core::GraphicsWrapper;
use control::core::Game;
use control::core::Modifiers;

//use arx_graphics::core::Context as ArxContext;
use arx_graphics::core::GraphicsResources;

use control::GameMode;
use control::tactical::TacticalMode;


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

fn main() {
    let mut window: PistonWindow = WindowSettings::new(
        "piston-tutorial",
        [1440, 900]
    )
        .build()
        .unwrap();


    let width = window.size().width;
    let height = window.size().height;
    let mut ui = conrod::UiBuilder::new([width as f64, height as f64])
        .theme(theme())
        .build();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSerif-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();



    let mut text_vertex_data = Vec::new();
    let (mut glyph_cache, mut text_texture_cache) = {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = conrod::text::GlyphCache::new(width, height, SCALE_TOLERANCE, POSITION_TOLERANCE);
        let buffer_len = width as usize * height as usize; // I see no reason to use the window width/height as the basis for the glyph cache in the long term...
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new().mag(piston_window::Filter::Nearest).min(piston_window::Filter::Nearest);
        let factory = &mut window.factory;
        let texture = G2dTexture::from_memory_alpha(factory, &init, width, height, &settings).unwrap();
        (cache, texture)
    };

    let image_map = conrod::image::Map::new();


    let mut game = Game::new(window.factory.clone());
    game.on_load(&mut window);

    game.active_mode.enter(&mut game.world);

    let mut command_key_down = false;

    while let Some(e) = window.next() {

        let win_w = window.size().width;
        let win_h = window.size().height;

        if let Some(render_args) = e.render_args() {
//            let base_viewport = render_args.viewport();
//            let scale = base_viewport.draw_size[0] as f64 / base_viewport.window_size[0] as f64;
//            let adjusted_viewport = Viewport {
//                rect : [0,0,base_viewport.rect[2] - 400, base_viewport.rect[3]],
//                window_size : base_viewport.window_size,
//                draw_size : [base_viewport.draw_size[0] - 400, base_viewport.draw_size[1]]
//            };
            let adjusted_viewport = render_args.viewport();

            window.window.make_current();
            window.g2d.draw(
                &mut window.encoder,
                &window.output_color,
                &window.output_stencil,
                adjusted_viewport,
                |c,g| {
                    game.on_draw(c, g);

                    let primitives = ui.draw();
                    // A function used for caching glyphs to the texture cache.
                    let cache_queued_glyphs = |graphics: &mut G2d,
                                               cache: &mut G2dTexture,
                                               rect: conrod::text::rt::Rect<u32>,
                                               data: &[u8]|
                        {
                            let offset = [rect.min.x, rect.min.y];
                            let size = [rect.width(), rect.height()];
                            let format = piston_window::texture::Format::Rgba8;
                            let encoder = &mut graphics.encoder;
                            text_vertex_data.clear();
                            text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                            piston_window::texture::UpdateTexture::update(cache, encoder, format, &text_vertex_data[..], offset, size)
                                .expect("failed to update texture")
                        };

                    // Specify how to get the drawable texture from the image. In this case, the image
                    // *is* the texture.
                    fn texture_from_image<T>(img: &T) -> &T { img }

                    // Draw the conrod `render::Primitives`.
                    conrod::backend::piston::draw::primitives(primitives,
                                                              c,
                                                              g,
                                                              &mut text_texture_cache,
                                                              &mut glyph_cache,
                                                              &image_map,
                                                              cache_queued_glyphs,
                                                              texture_from_image);
                }
            );
            window.encoder.flush(&mut window.device);
        }

//        window.draw_2d(&e, |c, g| {
//
//
//            game.on_draw(c, g);
//
//            let primitives = ui.draw();
//            // A function used for caching glyphs to the texture cache.
//            let cache_queued_glyphs = |graphics: &mut G2d,
//                                       cache: &mut G2dTexture,
//                                       rect: conrod::text::rt::Rect<u32>,
//                                       data: &[u8]|
//                {
//                    let offset = [rect.min.x, rect.min.y];
//                    let size = [rect.width(), rect.height()];
//                    let format = piston_window::texture::Format::Rgba8;
//                    let encoder = &mut graphics.encoder;
//                    text_vertex_data.clear();
//                    text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
//                    piston_window::texture::UpdateTexture::update(cache, encoder, format, &text_vertex_data[..], offset, size)
//                        .expect("failed to update texture")
//                };
//
//            // Specify how to get the drawable texture from the image. In this case, the image
//            // *is* the texture.
//            fn texture_from_image<T>(img: &T) -> &T { img }
//
//            // Draw the conrod `render::Primitives`.
//            conrod::backend::piston::draw::primitives(primitives,
//                                                      c,
//                                                      g,
//                                                      &mut text_texture_cache,
//                                                      &mut glyph_cache,
//                                                      &image_map,
//                                                      cache_queued_glyphs,
//                                                      texture_from_image);
//        });


        if let Some(btn) = e.button_args() {
            if btn.scancode == Some(54) || btn.scancode == Some(55) {
                command_key_down = btn.state == ButtonState::Press;
            }
        }

        if let Some(e) = conrod::backend::piston::event::convert(e.clone(), win_w as f64, win_h as f64) {
            ui.handle_event(e);
            let conrod_modifiers = ui.global_input().current.modifiers;
            control::core::set_key_modifiers(Modifiers {
                ctrl : conrod_modifiers.contains(ModifierKey::CTRL) || conrod_modifiers.contains(ModifierKey::GUI) || command_key_down,
                alt : conrod_modifiers.contains(ModifierKey::ALT),
                shift : conrod_modifiers.contains(ModifierKey::SHIFT)
            });
        }

        if let Some(btn) = e.button_args() {
            if btn.state == ButtonState::Press && btn.button == Button::Keyboard(Key::Q) && control::core::get_key_modifiers().ctrl {
                break;
            }
        }

        if let Some(upd) = e.update_args() {
            game.on_update(upd);

            let mut ui = ui.set_widgets();
            game.on_gui_update(&mut ui, upd);
        }

        game.on_event(&e);
    }
}
