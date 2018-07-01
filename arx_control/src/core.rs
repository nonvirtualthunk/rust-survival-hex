use game::World;
use arx_graphics::core::GraphicsWrapper;
use conrod;
use piston_window::*;
use game::entities::TileData;
use common::hex::*;
use gfx_device_gl;
use tactical::TacticalMode;

//use arx_graphics::core::Context as ArxContext;
use arx_graphics::core::GraphicsResources;


pub static mut GLOBAL_MODIFIERS : Modifiers = Modifiers {
    alt : false,
    ctrl : false,
    shift : false
};

pub fn get_key_modifiers() -> Modifiers {
    unsafe {
        GLOBAL_MODIFIERS.clone()
    }
}
pub fn set_key_modifiers(modifiers : Modifiers) {
    unsafe {
        GLOBAL_MODIFIERS = modifiers;
    }
}

pub trait GameMode {
    fn enter (&mut self, world : &mut World);
    fn update (&mut self, world : &mut World, dt : f64);
    fn update_gui (&mut self, world: &mut World, ui : &mut conrod::UiCell, frame_id : conrod::widget::Id);
    fn draw (&mut self, world : &mut World, g : &mut GraphicsWrapper);
    fn on_event (&mut self, world : &mut World, event : conrod::event::Widget);
}

#[derive(Clone,Copy,Default)]
pub struct Modifiers {
    pub ctrl : bool,
    pub shift : bool,
    pub alt : bool
}

pub struct Game {
    pub world: World,
    pub resources: GraphicsResources,
    pub active_mode: Box<GameMode>,
    pub frame_widget: Option<conrod::widget::Id>,
    pub viewport: Viewport
}

impl Game {
    pub fn new(factory: gfx_device_gl::Factory) -> Game {
        Game {
            world: World::new(),
            resources: GraphicsResources::new(factory, "survival"),
            active_mode: Box::new(TacticalMode::new()),
            frame_widget: None,
            viewport: Viewport {
                window_size: [256, 256],
                draw_size: [256, 256],
                rect: [0, 0, 256, 256]
            }
        }
    }
    pub fn on_load(&mut self, _: &mut PistonWindow) {}
    pub fn on_update(&mut self, upd: UpdateArgs) {
        self.active_mode.update(&mut self.world, upd.dt);
    }
    pub fn on_gui_update(&mut self, ui: &mut conrod::UiCell, _upd: UpdateArgs) {
        let frame_widget = *self.frame_widget.get_or_insert_with(|| ui.widget_id_generator().next());

        use conrod::*;
        conrod::widget::Canvas::new()
            .w(self.viewport.window_size[0] as f64)
            .h(self.viewport.window_size[1] as f64)
            .rgba(0.0f32,0.0f32,0.0f32,0.0f32)
            .set(frame_widget, ui);

        for evt in ui.widget_input(frame_widget).events() {
            self.active_mode.on_event(&mut self.world, evt);
        }

        self.active_mode.update_gui(&mut self.world, ui, frame_widget);
    }
    pub fn on_draw<'a>(&'a mut self, c: Context, g: &'a mut G2d) {
        if let Some(v) = c.viewport {
            self.viewport = v;
        }

        c.reset();

        clear([0.8, 0.8, 0.8, 1.0], g);

        let mut wrapper = GraphicsWrapper::new(c, &mut self.resources, g);

        self.active_mode.draw(&mut self.world, &mut wrapper);
        //        self.player.render(g, center);
    }

    pub fn on_event(&mut self, _event: &Event) {

        //        self.active_mode.on_event(&mut self.world, event);
    }
}


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