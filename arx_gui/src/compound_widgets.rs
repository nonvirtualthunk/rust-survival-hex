use common::Color;

use gui::*;
use graphics::ImageIdentifier;

use backtrace::Backtrace;
use events::UIEvent;
use events::WidgetEvent;

use anymap;
use anymap::any::CloneAny;
use std::any::Any;
use std::any::TypeId;
use std::mem;
use std::rc::Rc;
use events::EventPosition;
use std::collections::HashMap;
use anymap::AnyMap;
use std::sync::Mutex;
use piston_window::MouseButton;
use common::prelude::*;
use std::cell::RefCell;
use piston_window::keyboard;
use ui_event_types::*;
use events::UIEventType;
use events::ui_event_types;
use widgets::*;

#[derive(Default)]
pub struct Button {
    pub body: Widget,
    pub text: Widget,
}

impl Button {
    pub fn new<S>(text: S) -> Button where S: Into<String> {
        let body = Widget::new(
            WidgetType::Window { image: Some(String::from("ui/blank")), })
            .color(Color::white())
            .border_width(2)
            .border_color(Color::black())
            .size(Sizing::SurroundChildren, Sizing::SurroundChildren)
            .with_callback(|ctxt: &mut WidgetContext, evt: &UIEvent| {
                if let UIEvent::MouseRelease { .. } = evt {
                    ctxt.update_state(WidgetState::Button { pressed: false });
                    ctxt.trigger_event(UIEvent::WidgetEvent(WidgetEvent::ButtonClicked));
                } else if let UIEvent::MousePress { .. } = evt {
                    ctxt.update_state(WidgetState::Button { pressed: true });
                } else if let UIEvent::MouseExited { .. } = evt {
                    ctxt.update_state(WidgetState::Button { pressed: false });
                }
            })
            .event_consumption(EventConsumption::mouse_press_and_release());

        Button {
            text: Widget::text(text, 14)
                .size(Sizing::Derived, Sizing::Derived)
                .position(Positioning::Constant(0.px()), Positioning::Constant(0.px())),
            body,
        }
    }

    pub fn apply(mut self, gui: &mut GUI) -> Self {
        self.body = self.body.apply(gui);
        self.text = self.text.parent(&self.body).apply(gui);
        self
    }

    pub fn position(mut self, x: Positioning, y: Positioning) -> Self {
        self.body = self.body.position(x, y);
        self
    }

    pub fn parent(mut self, parent: &Widget) -> Self {
        self.body = self.body.parent(parent);
        self
    }

    pub fn with_on_click<State : 'static, F : Fn(&mut State) -> () + 'static>(mut self, function : F) -> Self {
        self.body = self.body.with_callback(move |state : &mut State, evt : &UIEvent| {
            if let UIEvent::WidgetEvent(evt) = evt {
                if let WidgetEvent::ButtonClicked = evt {
                    (function)(state)
                }
            };
        });
        self
    }
    pub fn with_on_click_2<State : 'static, OtherState : 'static, F : Fn(&mut State, &mut OtherState) -> () + 'static>(mut self, function : F) -> Self {
        self.body = self.body.with_callback_2(move |state : &mut State, other : &mut OtherState, evt : &UIEvent| {
            if let UIEvent::WidgetEvent(evt) = evt {
                if let WidgetEvent::ButtonClicked = evt {
                    (function)(state, other)
                }
            };
        });
        self
    }

    pub fn width(mut self, w: Sizing) -> Self {
        self.body = self.body.width(w);
        self.text = match w {
            Sizing::SurroundChildren => self.text.x(Positioning::Constant(0.px())),
            _ => self.text.x(Positioning::CenteredInParent)
        };
        self
    }

    pub fn as_widget(&self) -> &Widget {
        & self.text
    }

    pub fn clicked(&self, gui: &GUI) -> bool {
        for evt in gui.events_for(&self.body) {
            if let UIEvent::WidgetEvent(evt) = evt {
                if let WidgetEvent::ButtonClicked = evt {
                    return true;
                }
            }
        }
        false
    }
}


//pub struct CompoundWidget {
//    pub main_widget : Widget,
//    pub children : Vec<Widget>
//}
//
//impl CompoundWidget {
//    pub fn reapply(&mut self, gui : &mut GUI) {
//        self.main_widget.reapply();
//
//    }
//}

//trait CompoundWidget {
//    pub fn
//}

pub struct ListWidget<T : Default> {
    pub body : Widget,
    pub item_archetype : Widget,
    pub item_gap : UIUnits,
    pub child_structs : Vec<T>,
    pub children : Vec<Widget>,
    pub children_to_remove : Vec<Widget>
}

impl <T : Default> ListWidget<T> {
    pub fn new() -> ListWidget<T> {
        let item = Widget::window(Color::greyscale(0.7), 1)
            .x(Positioning::Constant(2.px()))
            .size(Sizing::DeltaOfParent(-4.px()), Sizing::SurroundChildren);
        ListWidget::custom(item, 2.px())
    }

    pub fn custom (item_archetype : Widget, item_gap : UIUnits) -> ListWidget<T> {
        ListWidget {
            body : Widget::window(Color::greyscale(0.8),2),
            item_archetype,
            child_structs : Vec::new(),
            children : Vec::new(),
            children_to_remove : Vec::new(),
            item_gap
        }
    }

    pub fn parent(mut self, parent : &Widget) -> Self {
        self.body = self.body.parent(parent);
        self
    }

    pub fn with_items<U, F : Fn(&mut GUI, &Widget, &mut T, &U)>(mut self, gui : &mut GUI, data : &[U], func : F) -> Self {
        self.update(gui, data, func);
        self
    }

    pub fn with_body<F : Fn(Widget) -> Widget>(mut self, func : F) -> Self {
        self.body = (func)(self.body);
        self
    }

    pub fn update<U, F : Fn(&mut GUI, &Widget, &mut T, &U)>(&mut self, gui : &mut GUI, data : &[U], func : F) -> &mut Self {
        while data.len() > self.children.len() {
            let mut new_item = self.item_archetype.clone().parent(&self.body);
            new_item.id = NO_WID;
            new_item.reapply(gui);

            new_item.position[1] = match self.children.last()  {
                Some(prev) => Positioning::DeltaOfWidget(prev.id, self.item_gap, Alignment::Bottom),
                None => Positioning::Constant(1.ux())
            };
            self.children.push(new_item);
            self.child_structs.push(T::default());
        }

        for (i, value) in data.iter().enumerate() {
            func(gui, &self.children[i], &mut self.child_structs[i], &value);
        }

        while data.len() < self.children.len() {
            let child = self.children.pop().expect("children can't be empty, that would indicate that data.len() < 0");
            self.children_to_remove.push(child);
        }

        self
    }


    pub fn apply(mut self, gui : &mut GUI) -> Self {
        self.reapply(gui);
        self
    }
    pub fn reapply(&mut self, gui : &mut GUI) {
        self.body.reapply(gui);

        loop {
            if let Some(mut child) = self.children_to_remove.pop() {
                gui.remove_widget(&mut child);
            } else {
                break;
            }
        }

        for child in &mut self.children {
            child.reapply(gui);
        }
    }
}

impl <T : Default> Default for ListWidget<T> {
    fn default() -> Self {
        ListWidget::new()
    }
}