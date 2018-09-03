use backtrace::Backtrace;
use common::Color;
use common::prelude::*;
use events::EventPosition;
use events::ui_event_types;
use events::UIEvent;
use events::UIEventType;
use events::WidgetEvent;
use graphics::ImageIdentifier;
use gui::*;
use itertools::Itertools;
use piston_window::keyboard;
use piston_window::MouseButton;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use ui_event_types::*;
use widgets::*;
use widget_delegation::DelegateToWidget;

#[derive(Default, Clone)]
pub struct Button {
    pub body: Widget,
    pub text: Widget,
}

impl Button {
    fn create_body(image: ImageIdentifier) -> Widget {
        Widget::new(
            WidgetType::Window { image: Some(image), segment: ImageSegmentation::None })
            .color(Color::white())
            .border_width(2)
            .border_color(Color::black())
            .size(Sizing::SurroundChildren, Sizing::SurroundChildren)
            .margin(0.5.ux())
            .with_inherent_callback(|ctxt: &mut WidgetContext, evt: &UIEvent| {
                let widget_id = ctxt.widget_id;
                if let UIEvent::MouseRelease { .. } = evt {
                    ctxt.update_state(WidgetState::Button { pressed: false });
                    ctxt.trigger_event(UIEvent::widget_event(WidgetEvent::ButtonClicked(widget_id), widget_id));
                } else if let UIEvent::MousePress { .. } = evt {
                    ctxt.update_state(WidgetState::Button { pressed: true });
                } else if let UIEvent::MouseExited { .. } = evt {
                    ctxt.update_state(WidgetState::Button { pressed: false });
                } else if let UIEvent::MouseDrag { .. } = evt {
                    ctxt.update_state(WidgetState::Button { pressed: true });
                }
            })
            .only_consume(EventConsumption::mouse_press_and_release())
    }

    pub fn image_button(image: ImageIdentifier) -> Widget {
        // TODO: make this use derived sizing, have derived sizing work with image widgets and default to the literal size of the image
        Button::create_body(image)
            .size(Sizing::constant(10.ux()), Sizing::constant(10.ux()))
    }

    pub fn new<S>(text: S) -> Button where S: Into<String> {
        let body = Button::create_body(String::from("ui/blank"));

        Button {
            text: Widget::text(text, 14)
                .size(Sizing::Derived, Sizing::Derived)
                .position(Positioning::Constant(0.px()), Positioning::Constant(0.px()))
                .parent(&body),
            body,
        }
    }

    pub fn set_text<S : Into<String>>(&mut self, text : S) -> &mut Self {
        self.text.set_text(text);
        self
    }

    pub fn apply(mut self, gui: &mut GUI) -> Self {
        self.reapply(gui);
        self
    }
    pub fn reapply(&mut self, gui: &mut GUI) {
        if let Sizing::SurroundChildren = self.body.size[0] {
            self.text.set_x(Positioning::origin());
        }

        self.body.reapply(gui);
        self.text.set_parent(&self.body).reapply(gui);
    }

    pub fn text_position(mut self, x: Positioning, y: Positioning) -> Self {
        self.text.set_position(x, y);
        self
    }

    pub fn with_on_click<F: Fn(&mut WidgetContext, &UIEvent) -> () + 'static>(mut self, function: F) -> Self {
        self.body = self.body.with_callback(move |ctxt: &mut WidgetContext, evt: &UIEvent| {
            if let UIEvent::WidgetEvent{ event, .. } = evt {
                if let WidgetEvent::ButtonClicked(btn) = event {
                    (function)(ctxt, evt)
                }
            };
        });
        self
    }
    pub fn with_on_click_2<State: 'static, OtherState: 'static, F: Fn(&mut State, &mut OtherState) -> () + 'static>(mut self, function: F) -> Self {
        self.body = self.body.with_callback_2(move |state: &mut State, other: &mut OtherState, event: &UIEvent| {
            if let UIEvent::WidgetEvent{ event, .. } = event {
                if let WidgetEvent::ButtonClicked(btn) = event {
                    (function)(state, other)
                }
            };
        });
        self
    }

    pub fn font_size(mut self, s: u32) -> Self {
        self.text.modify_widget_type(|m| if let WidgetType::Text { ref mut font_size, .. } = m { *font_size = s; });
        self
    }

    pub fn clicked(&self, gui: &GUI) -> bool {
        for event in gui.events_for(&self.body) {
            if let UIEvent::WidgetEvent{ event, .. } = event {
                if let WidgetEvent::ButtonClicked(btn) = event {
                    return true;
                }
            }
        }
        false
    }
}

impl DelegateToWidget for Button {
    fn as_widget(&mut self) -> &mut Widget {
        &mut self.body
    }

    fn as_widget_immut(&self) -> &Widget {
        &self.body
    }
}

impl WidgetContainer for Button {
    fn for_all_widgets<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
        (func)(&mut self.body);
        (func)(&mut self.text);
    }
}