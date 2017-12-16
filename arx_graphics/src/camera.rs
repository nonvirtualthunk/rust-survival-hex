use common::prelude::*;

use piston_window::math::*;

use piston_window::Viewport;

pub struct Camera2d {
    pub zoom: f32,
    pub move_speed: f32,
    pub position: Vec2f,
    pub move_delta: Vec2f
}

impl Camera2d {
    pub fn new() -> Camera2d {
        Camera2d {
            zoom: 1.0,
            move_speed: 1.0,
            position: v2(0.0, 0.0),
            move_delta: v2(0.0, 0.0)
        }
    }

    pub fn matrix(&self, viewport: Viewport) -> Matrix2d {
        //        match viewport_opt {
        //            Some(viewport) => {
        let max_draw_size = viewport.draw_size[0].max(viewport.draw_size[1]);
        let max_window_size = viewport.window_size[0].max(viewport.window_size[1]);

        let draw_window_ratio = max_draw_size as f64 / max_window_size as f64;
        let base_zoom = draw_window_ratio / max_window_size as f64;
        let zoom = self.zoom as f64 * base_zoom;
        let proportion = viewport.draw_size[0] as f64 / viewport.draw_size[1] as f64;
        multiply(scale(zoom, zoom * proportion), translate(as_f64(self.position * -1.0)))
        //            }
        //            None => {
        //                let zoom = 1.0 / 512.0;
        //                multiply(translate(as_f64(self.position)), scale(zoom, zoom))
        //            }
        //        }
    }
}