use common::prelude::*;
use game::World;
use arx_graphics::core::GraphicsWrapper;
use conrod;
use piston_window::*;
use game::entities::TileData;
use common::hex::*;
use gfx_device_gl;
use tactical::TacticalMode;
use gui::GUI;
use gui::Wid;
use gui::Widget;
use gui::WidgetType;
use gui::Sizing;
use gui::UIEvent;
use common::Color;

//use arx_graphics::core::Context as ArxContext;
use arx_graphics::core::GraphicsResources;


pub static mut GLOBAL_MODIFIERS : Modifiers = Modifiers {
    alt : false,
    ctrl : false,
    shift : false
};

//pub static mut MOUSE_POSITION : Vec2f = Vec2f {
//    x : 0.0,
//    y : 0.0
//};

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
    fn update_gui (&mut self, world: &mut World, ui : &mut GUI, frame_id : Option<Wid>);
    fn draw (&mut self, world : &mut World, g : &mut GraphicsWrapper);
    fn on_event<'a, 'b> (&'a mut self, world : &mut World, ui : &'b mut GUI, event : &UIEvent);
    fn handle_event(&mut self, world: &mut World, gui : &mut GUI, event: &UIEvent);
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
    pub viewport: Viewport,
    pub gui : GUI
}

impl Game {
    pub fn new(factory: gfx_device_gl::Factory) -> Game {
        let mut gui = GUI::new();

        Game {
            world: World::new(),
            resources: GraphicsResources::new(factory, "survival"),
            active_mode: Box::new(TacticalMode::new(&mut gui)),
            gui,
            viewport: Viewport {
                window_size: [256, 256],
                draw_size: [256, 256],
                rect: [0, 0, 256, 256]
            },
        }
    }
    pub fn on_load(&mut self, _: &mut PistonWindow) {

    }
    pub fn on_update(&mut self, upd: UpdateArgs) {
        self.active_mode.update(&mut self.world, upd.dt);

        self.active_mode.update_gui(&mut self.world, &mut self.gui, None);

        self.gui.reset_events();
    }

    pub fn on_draw<'a>(&'a mut self, c: Context, g: &'a mut G2d) {
        if let Some(v) = c.viewport {
            self.viewport = v;
        }

        c.reset();

        clear([0.8, 0.8, 0.8, 1.0], g);

        let mut wrapper = GraphicsWrapper::new(c, &mut self.resources, g);

        self.active_mode.draw(&mut self.world, &mut wrapper);

        self.gui.draw(&mut wrapper);
        //        self.player.render(g, center);
    }

    pub fn on_event(&mut self, event: &Event) {
        if let Some(ui_event) = self.gui.convert_event(event.clone()) {
            if !self.gui.handle_ui_event_for_self(&ui_event) {
                self.active_mode.handle_event(&mut self.world, &mut self.gui, &ui_event);
            }
            self.active_mode.on_event(&mut self.world, &mut self.gui, &ui_event);
        }
    }
}


pub fn normalize_mouse(mouse: Vec2f, viewport: &Viewport) -> Vec2f {
    let in_x = mouse.x;
    let in_y = viewport.window_size[1] as f32 - mouse.y - 1.0;

    let centered_x = in_x - (viewport.window_size[0] / 2) as f32;
    let centered_y = in_y - (viewport.window_size[1] / 2) as f32;

    let norm_x = centered_x / viewport.window_size[0] as f32;
    let norm_y = centered_y / viewport.window_size[1] as f32;

    let scale_factor = viewport.draw_size[0] as f32 / viewport.window_size[0] as f32;

    let scaled_x = norm_x * scale_factor;
    let scaled_y = norm_y * scale_factor;

    v2(scaled_x, scaled_y)
}