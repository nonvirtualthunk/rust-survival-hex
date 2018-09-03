use cgmath::InnerSpace;
use common::Color;
use common::EventBus;
use common::prelude::*;
use game::DebugData;
use game::entities::reactions::reaction_types;
use game::entities::TileData;
use game::GameEvent;
use game::prelude::*;
use game::reflect::*;
use game::scenario::Scenario;
use game::scenario::test_scenarios::FirstEverScenario;
use game::universe::*;
use game::World;
use gfx_device_gl;
//use graphics::core::Context as ArxContext;
use graphics::core::GraphicsResources;
use graphics::core::GraphicsWrapper;
use gui::GUI;
use gui::Sizing;
use gui::UIEvent;
use gui::Wid;
use gui::Widget;
use gui::WidgetType;
use piston_window::*;
use std::collections::HashMap;
use std::path::Path;
use tactical::TacticalMode;
use game::entities::FactionData;
use std::sync::Mutex;
use std::rc::Rc;
use common::event_bus::ConsumerHandle;
use main_menu::*;
use graphics::Camera2d;
use gui::control_events::GameModeEvent;
use std::io::BufReader;
use std::fs::File;


pub trait GameMode {
    fn enter(&mut self, gui: &mut GUI, universe: &mut Universe, event_bus: &mut EventBus<GameModeEvent>);
    fn update(&mut self, universe: &mut Universe, dt: f64, event_bus: &mut EventBus<GameModeEvent>);
    fn update_gui(&mut self, universe: &mut Universe, ui: &mut GUI, frame_id: Option<Wid>, event_bus: &mut EventBus<GameModeEvent>);
    fn draw(&mut self, universe: &mut Universe, g: &mut GraphicsWrapper, event_bus: &mut EventBus<GameModeEvent>);
    fn on_event<'a, 'b>(&'a mut self, universe: &mut Universe, ui: &'b mut GUI, event: &UIEvent, event_bus: &mut EventBus<GameModeEvent>);
    fn handle_event(&mut self, universe: &mut Universe, gui: &mut GUI, event: &UIEvent, event_bus: &mut EventBus<GameModeEvent>);
}

#[derive(Serialize, Deserialize)]
struct GameState {
    universe: Universe,
    active_world: Option<WorldRef>,
}

pub struct Game {
    state: GameState,
    pub event_bus: EventBus<GameModeEvent>,
    pub consumer_handle: ConsumerHandle,
    pub resources: GraphicsResources,
    pub active_mode: Box<GameMode>,
    pub viewport: Viewport,
    pub gui: GUI,
    pub exit_requested: bool,
}

impl Game {
    pub fn new(factory: gfx_device_gl::Factory) -> Game {
        let mut gui = GUI::new();

        let universe = Universe::new();
        let main_mode = MainMenu::new(&mut gui);

        let event_bus = EventBus::new();
        let consumer_handle = event_bus.register_consumer(false);
        Game {
            state: GameState { universe, active_world: None },
            event_bus,
            consumer_handle,
            resources: GraphicsResources::new(factory, "survival"),
            active_mode: Box::new(main_mode),
            gui,
            viewport: Viewport {
                window_size: [256, 256],
                draw_size: [256, 256],
                rect: [0, 0, 256, 256],
            },
            exit_requested : false,
        }
    }


    pub fn init_world() -> (World, Entity) {
        let world = FirstEverScenario {}.initialize_scenario_world();
        let player_faction = world.view().entities_with_data::<FactionData>().iter().find(|(ent, faction_data)| faction_data.player_faction).unwrap();

        (world, *player_faction.0)
    }

    pub fn on_load(&mut self, _: &mut PistonWindow) {
        self.active_mode.enter(&mut self.gui, &mut self.state.universe, &mut self.event_bus);
    }
    pub fn on_update(&mut self, upd: UpdateArgs) {
        self.active_mode.update(&mut self.state.universe, upd.dt, &mut self.event_bus);

        self.active_mode.update_gui(&mut self.state.universe, &mut self.gui, None, &mut self.event_bus);

        self.gui.reset_events();

        let handle = &mut self.consumer_handle;
        let mut mode_changed = false;
        for event in self.event_bus.events_for(handle) {
            match event {
                GameModeEvent::EnterTacticalMode(world_ref, start_at_beginning) => {
                    self.gui = GUI::new();
                    let tactical_mode = TacticalMode::new(&mut self.gui, *world_ref, *start_at_beginning);
                    self.active_mode = box tactical_mode;
                    self.state.active_world = Some(*world_ref);
                    mode_changed = true;
                }
                GameModeEvent::InitScenario(scenario) => {
                    let world = scenario.initialize_scenario_world();
                    let world_ref = self.state.universe.register_world(world);
                    self.gui = GUI::new();
                    let tactical_mode = TacticalMode::new(&mut self.gui, world_ref, true);
                    self.active_mode = box tactical_mode;
                    self.state.active_world = Some(world_ref);
                    mode_changed = true;
                }
                GameModeEvent::Load(save_name) => {
                    use gui::open_save_file;
                    if let Some(save_file) = open_save_file(false) {
                        let buf_reader = BufReader::new(save_file);
                        use ron;
                        if let Ok(game_state) = ron::de::from_reader(buf_reader) {
                            self.state = game_state;
                            // TODO: Once we have non-tactical worlds, this will get...different
                            for world in &mut self.state.universe.worlds {
                                world.initialize_loaded_world();
                                register_world_data(world);
                            }
                            self.gui = GUI::new();
                            let tactical_mode = TacticalMode::new(&mut self.gui, self.state.active_world.expect("Loaded with no active world, which is weird"), false);
                            self.active_mode = box tactical_mode;
                            mode_changed = true;
                        } else {
                            error!("Error while attempting to read save file");
                        }
                    } else {
                        error!("Attempted to load non-existent save file");
                    }
                }
                GameModeEvent::Save(save_name) => {
                    use gui::open_save_file;
                    use std::io::Write;

                    if let Some(mut save_file) = open_save_file(true) {
                        use ron;
                        if let Ok(serialized) = ron::ser::to_string(&self.state) {
                            save_file.write_all(serialized.as_bytes()).expect("failed to write to save file");
                        } else {
                            error!("Failed to serialize game state");
                        }
                    } else {
                        error!("Attempted to load non-existent save file");
                    }
                },
                GameModeEvent::MainMenu => {
                    self.gui = GUI::new();
                    let main_menu = MainMenu::new(&mut self.gui);
                    self.active_mode = box main_menu;
                    mode_changed = true;
                },
                GameModeEvent::Exit => {
                  self.exit_requested = true;
                }
            }
        }
        if mode_changed {
            self.active_mode.enter(&mut self.gui, &mut self.state.universe, &mut self.event_bus);
        }
    }

    pub fn on_draw<'a>(&'a mut self, c: Context, g: &'a mut G2d) {
        if let Some(v) = c.viewport {
            self.viewport = v;
        }

        c.reset();

        clear([0.8, 0.8, 0.8, 1.0], g);

        let mut wrapper = GraphicsWrapper::new(c, &mut self.resources, g);

        self.active_mode.draw(&mut self.state.universe, &mut wrapper, &mut self.event_bus);


        let gui_camera = Camera2d::new();
        wrapper.context.view = gui_camera.matrix(self.viewport);

        self.gui.draw(&mut wrapper);
        //        self.player.render(g, center);
    }

    pub fn on_event(&mut self, event: &Event) {
        if let Some(ui_event) = self.gui.convert_event(event.clone()) {
            if !self.gui.handle_ui_event_for_self(&ui_event) {
                self.active_mode.handle_event(&mut self.state.universe, &mut self.gui, &ui_event, &mut self.event_bus);
            }
            self.active_mode.on_event(&mut self.state.universe, &mut self.gui, &ui_event, &mut self.event_bus);
        }
    }
}

pub fn normalize_screen_pos(screen_pos: Vec2f, viewport: &Viewport) -> Vec2f {
    let in_x = screen_pos.x;
    let in_y = viewport.window_size[1] as f32 - screen_pos.y - 1.0;

    let centered_x = in_x - (viewport.window_size[0] / 2) as f32;
    let centered_y = in_y - (viewport.window_size[1] / 2) as f32;

    let norm_x = centered_x / viewport.window_size[0] as f32;
    let norm_y = centered_y / viewport.window_size[1] as f32;

    let scale_factor = viewport.draw_size[0] as f32 / viewport.window_size[0] as f32;

    let scaled_x = norm_x * scale_factor;
    let scaled_y = norm_y * scale_factor;

    v2(scaled_x, scaled_y)
}