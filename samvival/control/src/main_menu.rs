use GameMode;
use game::universe::Universe;
use gui::*;
use graphics::GraphicsWrapper;
use gui::control_events::GameModeEvent;
use common::event_bus::EventBus;
use common::Color;
use std::sync::Mutex;
use std::rc::Rc;
use std::cell::RefCell;
use common;
use std::fs::File;
use std::io::BufReader;
use common::prelude::strf;
use game::World;
use game::universe::WorldRef;
use game::scenario::test_scenarios::FirstEverScenario;

#[derive(Default)]
pub struct MainMenu {
    pub menu: Widget,
    pub load_button: Button,
    pub new_button: Button,
    pub exit_button: Button
}

impl MainMenu {
    pub fn new(gui: &mut GUI) -> MainMenu {
        let menu = Widget::window(Color::greyscale(0.8), 3)
            .centered()
            .width(Sizing::surround_children())
            .height(Sizing::surround_children())
            .margin(20.px())
            .apply(gui);

        let new_button = Button::new("New Game")
            .parent(&menu)
            .font_size(FontSize::ExtraLarge)
            .x(Positioning::centered())
            .apply(gui);

        let mut load_button = Button::new("Continue")
            .parent(&menu)
            .font_size(FontSize::ExtraLarge)
            .below(&new_button, 5.px())
            .x(Positioning::centered())
            .apply(gui);
        if open_save_file(false).is_none() {
            load_button.set_color(Color::greyscale(0.2)).reapply(gui);
        }

        let exit_button = Button::new("Exit")
            .parent(&menu)
            .font_size(FontSize::ExtraLarge)
            .below(&load_button, 5.px())
            .x(Positioning::centered())

            .apply(gui);
        MainMenu {
            menu,
            load_button,
            new_button,
            exit_button
        }
    }
}

impl GameMode for MainMenu {
    fn enter(&mut self, gui: &mut GUI, universe: &mut Universe, event_bus: &mut EventBus<GameModeEvent>) {

    }

    fn update(&mut self, universe: &mut Universe, dt: f64, event_bus: &mut EventBus<GameModeEvent>) {}

    fn update_gui(&mut self, universe: &mut Universe, ui: &mut GUI, frame_id: Option<Wid>, event_bus: &mut EventBus<GameModeEvent>) {
        for event in ui.events_for(&self.menu) {
            if let UIEvent::WidgetEvent { event: WidgetEvent::ButtonClicked(button), .. } = event {
                if button == &self.load_button.id() {
                    event_bus.push_event(GameModeEvent::Load(strf("savegame")));
                } else if button == &self.new_button.id() {
                    event_bus.push_event(GameModeEvent::InitScenario(box FirstEverScenario{}));
                } else if button == &self.exit_button.id() {
                    event_bus.push_event(GameModeEvent::Exit);
                }
            }
        }
    }

    fn draw<'a, 'b>(&mut self, universe: &mut Universe, g: &mut GraphicsWrapper<'a, 'b>, event_bus: &mut EventBus<GameModeEvent>) {}

    fn on_event<'a, 'b>(&'a mut self, universe: &mut Universe, ui: &'b mut GUI, event: &UIEvent, event_bus: &mut EventBus<GameModeEvent>) {}

    fn handle_event(&mut self, universe: &mut Universe, gui: &mut GUI, event: &UIEvent, event_bus: &mut EventBus<GameModeEvent>) {}
}