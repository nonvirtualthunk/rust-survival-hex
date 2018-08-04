#[allow(unused_variables)]

use piston_window;
use piston_window::Window;
use piston_window::PistonWindow;
use piston_window::WindowSettings;
use piston_window::ButtonState;
use piston_window::RenderEvent;
use piston_window::OpenGLWindow;
use piston_window::ButtonEvent;
use piston_window::UpdateEvent;

use std::f32::consts::PI;

use common::prelude::*;

use control::core::Game;

use arx_graphics::GraphicsWrapper;
use arx_graphics::GraphicsResources;
use gui::*;
use common::color::Color;
use graphics::clear;
use arx_graphics::camera::Camera2d;

#[allow(unused_variables)]
pub fn run() {
    let mut window: PistonWindow = WindowSettings::new(
        "ui-playground",
        [1440, 900],
    )
        .build()
        .unwrap();



    let mut gui = GUI::new();

    make_windows(&mut gui);

    let mut resources = GraphicsResources::new(window.factory.clone(), "ui_playground");

    while let Some(e) = window.next() {
        if let Some(render_args) = e.render_args() {
            let adjusted_viewport = render_args.viewport();
            resources.assets.dpi_scale = adjusted_viewport.draw_size[0] as f32 / adjusted_viewport.window_size[0] as f32;

            window.window.make_current();
            window.g2d.draw(
                &mut window.encoder,
                &window.output_color,
                &window.output_stencil,
                adjusted_viewport,
                |c,g| {
                    c.reset();

                    clear([0.8, 0.8, 0.8, 1.0], g);

                    let mut g = GraphicsWrapper::new(c, &mut resources, g);

                    let gui_camera = Camera2d::new();
                    g.context.view = gui_camera.matrix(adjusted_viewport);

                    gui.draw(&mut g);
                }
            );
            window.encoder.flush(&mut window.device);
        }

        gui.handle_event_for_self(e.clone());

        if let Some(btn) = e.button_args() {
            if btn.state == ButtonState::Press && btn.button == piston_window::Button::Keyboard(Key::Q) && gui.active_modifiers().ctrl {
                break;
            }
        }

        if let Some(upd) = e.update_args() {

        }
    }

}

#[allow(unused_variables)]
fn make_windows(gui: &mut GUI) {
    let test_window = Widget::window(Color::new(0.8, 0.5, 0.2, 1.0), 1)
        .size(Sizing::DeltaOfParent(-50.0.ux()), Sizing::PcntOfParent(0.75))
        .position(Positioning::CenteredInParent, Positioning::CenteredInParent)
        .apply(gui);

    let sub_window = Widget::new(WidgetType::image("default/defaultium"))
        .size(Sizing::Constant(40.0.ux()), Sizing::PcntOfParent(0.75))
        .position(Positioning::Constant(2.0.ux()), Positioning::CenteredInParent)
        .alignment(Alignment::Right, Alignment::Top)
        .parent(&test_window)
        .apply(gui);

    let sub_text = Widget::text(String::from("Hello, world\nNewline"), 16)
        .size(Sizing::Derived, Sizing::Derived)
        .position(Positioning::Constant(1.0.ux()), Positioning::Constant(1.0.ux()))
        .parent(&sub_window)
        .apply(gui);

    let right_text = Widget::text(String::from("| Right |"), 16)
        .size(Sizing::Derived, Sizing::Derived)
        .position(Positioning::right_of(&sub_text, 1.px()), Positioning::match_to(&sub_text))
        .parent(&sub_window)
        .apply(gui);

    let left_sub_window = Widget::window(Color::white(), 1)
        .size(Sizing::Constant(20.0.ux()), Sizing::Constant(20.0.ux()))
        .position(Positioning::left_of(&sub_window, 1.px()), Positioning::match_to(&sub_window))
        .alignment(Alignment::Left, Alignment::Top)
        .parent(&test_window)
        .apply(gui);

    let button = Button::new("Test Button")
        .position(Positioning::DeltaOfWidget(left_sub_window.id(), 0.px(), Alignment::Left),
                  Positioning::DeltaOfWidget(left_sub_window.id(), 2.px(), Alignment::Bottom))
        .parent(&test_window)
        .apply(gui);


    let pill_bar = Widget::new(WidgetType::Window { image : Some(String::from("ui/pill")), segment : ImageSegmentation::Horizontal })
        .position(Positioning::Constant(4.ux()), Positioning::Constant(4.ux()))
        .size(Sizing::Constant(10.ux()), Sizing::Constant(4.ux()))
        .parent(&test_window)
        .with_tooltip("This is a tooltip")
        .apply(gui);

    let tab_widget = TabWidget::new(vec!["Foo","Bar","Bazilicus"])
        .position(Positioning::Constant(4.ux()), Positioning::Constant(10.ux()))
        .size(Sizing::Constant(30.ux()), Sizing::Constant(50.ux()))
        .parent(&test_window)
        .apply(gui);

    let foo_widget = Widget::text("This is the Foo Tab", 14)
        .position(Positioning::CenteredInParent, Positioning::CenteredInParent)
        .parent(tab_widget.tab_named("Foo"))
        .apply(gui);

    let bar_widget = Widget::image("images/archer", Color::white(), 1)
        .position(Positioning::Constant(2.ux()), Positioning::Constant(10.ux()))
        .parent(tab_widget.tab_named("Bar"))
        .with_tooltip("This is a longer tooltip, this should trigger the whole thing to become wrapped by parent")
        .apply(gui);


}


use std::path::Path;
use common::prelude::v2;
use common::*;
use image;
#[test]
pub fn generate_hex_images() {
    let w = 71;
    let h = 62;
    let mut image : image::RgbaImage = image::RgbaImage::new(w, h);

    for x in 0 .. w {
        for y in 0 .. h {
            let xf = ((x as i32 -0) as i32 - w as i32/2) as f32 / (w+1) as f32;
            let yf = ((y as i32 +0) as i32 - h as i32/2) as f32 / (w+1) as f32;

            let mut pixel_ref = image.get_pixel_mut(x,y);
            if AxialCoord::from_cart_coord(CartVec(v2(xf*2.0,yf*2.0))) == AxialCoord::new(0,0) {
                pixel_ref[0] = 255;
                pixel_ref[1] = 255;
                pixel_ref[2] = 255;
                pixel_ref[3] = 255;
            } else {
                pixel_ref[0] = 0;
                pixel_ref[1] = 0;
                pixel_ref[2] = 0;
                pixel_ref[3] = 0;
            }
        }
    }

    image.save(Path::new("/tmp/full_hex.png")).expect("could not save");

    let s32 = 3.0f32.sqrt();
    let slice_func = |xf,yf| {
        if yf < 0.0 {
            if xf < 0.0 {
                if yf > xf * s32 {
                    4
                } else {
                    5
                }
            } else {
                if yf > xf * -s32 {
                    0
                } else {
                    5
                }
            }
        } else {
            if xf < 0.0 {
                if yf < xf * -s32 {
                    3
                } else {
                    2
                }
            } else {
                if yf < xf * s32 {
                    1
                } else {
                    2
                }
            }
        }
    };

    for rounded in vec![true, false] {
        for q in 0 .. 6 {
            let mut image : image::RgbaImage = image::RgbaImage::new(w, h);

            for x in 0 .. w {
                for y in 0 .. h {
                    let xf = (((x as i32 -0) as i32 - w as i32/2) as f32 / (w+1) as f32) * 2.0;
                    let yf = (((y as i32 +0) as i32 - h as i32/2) as f32 / (w+1) as f32) * 2.0;

                    let mut pixel_ref = image.get_pixel_mut(x,y);
                    if AxialCoord::from_cart_coord(CartVec(v2(xf,yf))) == AxialCoord::new(0,0) {
                        if (slice_func)(xf,yf) == q {
                            let mut dist = xf*xf + yf*yf;
                            if dist != 0.0 {
                                dist = dist.sqrt();
                            }

                            let mut angle = f32::atan2(yf, xf) + PI * 2.0;
                            while angle > PI / 3.0 {
                                angle -= PI / 3.0;
                            }

                            let max_d = 3.0f32.sqrt() / (3f32.sqrt() * angle.cos() + angle.sin());
                            let pcnt = dist / max_d;

                            let effective_dist = if rounded { dist } else { pcnt };
                            let (b,a) = if effective_dist > 0.8 {
                                (200,255)
                            } else if effective_dist > 0.5 {
                                (255,(((effective_dist - 0.5) / 0.3) * 255.0) as u8)
                            } else {
                                (255,0)
                            };

                            pixel_ref[0] = b;
                            pixel_ref[1] = b;
                            pixel_ref[2] = b;
                            pixel_ref[3] = a;
                        } else {
                            pixel_ref[0] = 0;
                            pixel_ref[1] = 0;
                            pixel_ref[2] = 0;
                            pixel_ref[3] = 0;
                        }
                    } else {
                        pixel_ref[0] = 0;
                        pixel_ref[1] = 0;
                        pixel_ref[2] = 0;
                        pixel_ref[3] = 0;
                    }
                }
            }

            let rounded_prefix = if rounded { "_rounded" } else { "" };
            image.save(Path::new(&format!("/tmp/hex_edge{}_{}.png", rounded_prefix, q))).expect("could not save");
        }
    }
}