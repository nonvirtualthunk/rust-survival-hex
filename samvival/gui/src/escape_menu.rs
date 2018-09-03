use common::prelude::*;

use gui::*;
use common::event_bus::EventBus;
use common::Color;
use control_events::TacticalEvents;

#[derive(WidgetContainer,Clone,Default)]
pub struct EscapeMenu {
    body : Widget,
    save_button : Button,
    main_menu_button : Button,
}

impl DelegateToWidget for EscapeMenu {
    fn as_widget(&mut self) -> &mut Widget { &mut self.body }
    fn as_widget_immut(&self) -> &Widget { &self.body }
}

impl EscapeMenu {
    pub fn new(gui : &mut GUI, parent : &Widget) -> EscapeMenu {
        let body = Widget::window(Color::greyscale(0.8), 2).surround_children().margin(10.px()).centered().parent(parent).apply(gui);

        let save_button = Button::new("Save").parent(&body)//.x(Positioning::CenteredInParent)
            .with_on_click(|ctxt : &mut WidgetContext, evt : &UIEvent| {
               ctxt.trigger_event(UIEvent::custom_event(TacticalEvents::Save, ctxt.widget_id));
            }).apply(gui);

        let main_menu_button = Button::new("Main Menu").parent(&body)//.x(Positioning::CenteredInParent)
            .below(&save_button, 5.px())
            .with_on_click(|ctxt : &mut WidgetContext, evt : &UIEvent| {
                ctxt.trigger_event(UIEvent::custom_event(TacticalEvents::MainMenu, ctxt.widget_id));
            }).apply(gui);

        let mut ret = EscapeMenu {
            body,
            save_button,
            main_menu_button,
        };
        ret.toggle(gui);
        ret
    }

    pub fn toggle(&mut self, gui : &mut GUI) {
        self.body.toggle_showing().reapply(gui)
    }
//    pub fn update(&mut self, event_bus : &mut EventBus<ControlEvent>) {
//
//    }
}