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
use itertools::Itertools;


impl DelegateToWidget for TabWidget {
    fn as_widget(&mut self) -> &mut Widget {
        &mut self.body
    }

    fn as_widget_immut(&self) -> &Widget {
        &self.body
    }
}


#[derive(Default)]
pub struct Button {
    pub body: Widget,
    pub text: Widget,
}

impl Button {
    fn create_body(image : ImageIdentifier) -> Widget {
        Widget::new(
            WidgetType::Window { image: Some(image), segment : ImageSegmentation::None })
            .color(Color::white())
            .border_width(2)
            .border_color(Color::black())
            .size(Sizing::SurroundChildren, Sizing::SurroundChildren)
            .margin(0.5.ux())
            .with_callback(|ctxt: &mut WidgetContext, evt: &UIEvent| {
                let widget_id = ctxt.widget_id;
                if let UIEvent::MouseRelease { .. } = evt {
                    ctxt.update_state(WidgetState::Button { pressed: false });
                    ctxt.trigger_event(UIEvent::WidgetEvent(WidgetEvent::ButtonClicked(widget_id)));
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

    pub fn image_button(image : ImageIdentifier) -> Widget {
        // TODO: make this use derived sizing, have derived sizing work with image widgets and default to the literal size of the image
        Button::create_body(image)
            .size(Sizing::constant(10.ux()), Sizing::constant(10.ux()))
    }

    pub fn new<S>(text: S) -> Button where S: Into<String> {
        let body = Button::create_body(String::from("ui/blank"));

        Button {
            text: Widget::text(text, 14)
                .size(Sizing::Derived, Sizing::Derived)
                .position(Positioning::Constant(0.px()), Positioning::Constant(0.px())),
            body,
        }
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

    pub fn text_position(mut self, x : Positioning, y : Positioning) -> Self {
        self.text.set_position(x, y);
        self
    }

    pub fn with_on_click<State : 'static, F : Fn(&mut State) -> () + 'static>(mut self, function : F) -> Self {
        self.body = self.body.with_callback(move |state : &mut State, evt : &UIEvent| {
            if let UIEvent::WidgetEvent(evt) = evt {
                if let WidgetEvent::ButtonClicked(btn) = evt {
                    (function)(state)
                }
            };
        });
        self
    }
    pub fn with_on_click_2<State : 'static, OtherState : 'static, F : Fn(&mut State, &mut OtherState) -> () + 'static>(mut self, function : F) -> Self {
        self.body = self.body.with_callback_2(move |state : &mut State, other : &mut OtherState, evt : &UIEvent| {
            if let UIEvent::WidgetEvent(evt) = evt {
                if let WidgetEvent::ButtonClicked(btn) = evt {
                    (function)(state, other)
                }
            };
        });
        self
    }

    pub fn font_size(mut self, s : u32) -> Self {
        self.text.modify_widget_type(|m| if let WidgetType::Text{ ref mut font_size, .. } = m { *font_size = s; });
        self
    }

    pub fn clicked(&self, gui: &GUI) -> bool {
        for evt in gui.events_for(&self.body) {
            if let UIEvent::WidgetEvent(evt) = evt {
                if let WidgetEvent::ButtonClicked(btn) = evt {
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

pub struct ListWidget<T : Default> {
    pub body : Widget,
    pub item_archetype : Widget,
    pub item_gap : UIUnits,
    pub child_structs : Vec<T>,
    pub children : Vec<Widget>,
    pub children_to_remove : Vec<Widget>,
    pub orientation : Orientation
}

impl <T : Default + WidgetContainer> ListWidget<T> {
    pub fn new() -> ListWidget<T> {
        let item = Widget::window(Color::greyscale(0.7), 1)
            .size(Sizing::match_parent(), Sizing::SurroundChildren)
            .named("List widget item archetype");
        ListWidget::custom(item, 2.px())
    }

    /// creates a list widget without explicit sectioning
    pub fn lightweight () -> ListWidget<T> {
        let item = Widget::div()
            .size(Sizing::PcntOfParent(1.0), Sizing::SurroundChildren);
        ListWidget::custom(item, 2.px())
            .margin(2.px())
    }

    /// creates a list widget with no background or sectioning, it exists only to arrange its children
    pub fn featherweight () -> ListWidget<T> {
        ListWidget::lightweight()
            .color(Color::clear())
            .margin(0.px())
            .border_width(0)
            .height(Sizing::surround_children())
    }

    pub fn custom (item_archetype : Widget, item_gap : UIUnits) -> ListWidget<T> {
        ListWidget {
            body : Widget::window(Color::greyscale(0.8),2).margin(2.px()),
            item_archetype,
            child_structs : Vec::new(),
            children : Vec::new(),
            children_to_remove : Vec::new(),
            item_gap,
            orientation : Orientation::Vertical
        }.only_consume(EventConsumption::widget_events())
    }

    pub fn parent(mut self, parent : &Widget) -> Self {
        self.body = self.body.parent(parent);
        self
    }

    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.orientation = Orientation::Horizontal;
        if self.body.size[1] == Sizing::SurroundChildren {
            let w = self.body.size[0];
            let h = self.body.size[1];
            self.body.size = [h, w];
        }
        if self.item_archetype.size[1] == Sizing::SurroundChildren {
            let w = self.item_archetype.size[0];
            let h = self.item_archetype.size[1];
            self.item_archetype.size = [h, w];
        }
        self
    }

    pub fn with_items<U, F : Fn(&mut T, &U)>(mut self, gui : &mut GUI, data : &[U], func : F) -> Self {
        self.update(gui, data, func);
        self
    }

    pub fn with_body<F : Fn(Widget) -> Widget>(mut self, func : F) -> Self {
        self.body = (func)(self.body);
        self
    }

    pub fn clear(&mut self, gui : &mut GUI) -> &mut Self {
        self.update(gui, &[], |t,u : &i32| {})
    }

    pub fn update<U, F : Fn(&mut T, &U)>(&mut self, gui : &mut GUI, data : &[U], func : F) -> &mut Self {
        self.reapply(gui);
        while data.len() > self.children.len() {
            let index = self.children.len();
            let mut new_item = self.item_archetype.clone().parent(&self.body)
                .with_callback(move |ctxt : &mut WidgetContext, evt : &UIEvent| {
                    if let UIEvent::MouseRelease { .. } = evt {
                        ctxt.trigger_event(UIEvent::WidgetEvent(WidgetEvent::ListItemClicked(index)))
                    }
                });
            new_item.clear_id();

            match self.orientation {
                Orientation::Vertical => {
                    new_item.position[1] = match self.children.last()  {
                        Some(prev) => Positioning::below(prev, self.item_gap),
                        None => Positioning::origin()
                    };
                },
                Orientation::Horizontal => {
                    new_item.position[0] = match self.children.last() {
                        Some(prev) => Positioning::right_of(prev, self.item_gap),
                        None => Positioning::origin()
                    }
                },
                _ => error!("Somehow conspired to get a list widget with a non-horizontal, non-vertical orientation. That doesn't work.")
            }

            new_item.reapply(gui);

            self.children.push(new_item);
            self.child_structs.push(T::default());
        }

        for (i, value) in data.iter().enumerate() {
            let child_id = self.children[i].id();
            func(&mut self.child_structs[i], &value);
            ListWidget::auto_apply(child_id, gui, &mut self.child_structs[i]);
        }

        while data.len() < self.children.len() {
            let child = self.children.pop().expect("children can't be empty, that would indicate that data.len() < 0");
            self.children_to_remove.push(child);
        }

        self
    }

    fn auto_apply(id : Wid, gui : &mut GUI, child_struct : &mut T) {
        child_struct.for_all_widgets(|w| {
            if w.parent_id.is_none() {
                w.set_parent_id(id);
            }
            w.reapply(gui);
        });
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

impl <T : Default + WidgetContainer> Default for ListWidget<T> {
    fn default() -> Self {
        ListWidget::new()
    }
}

impl <T: Default> DelegateToWidget for ListWidget<T> {
    fn as_widget(&mut self) -> &mut Widget {
        &mut self.body
    }

    fn as_widget_immut(&self) -> &Widget {
        &self.body
    }
}

impl <T: Default> WidgetContainer for ListWidget<T> {
    fn for_all_widgets<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
        (func)(&mut self.body);
        for child in &mut self.children {
            (func)(child);
        }
    }
}

impl WidgetContainer for TabWidget {
    fn for_all_widgets<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
        (func)(&mut self.body);
        for tab in &mut self.tabs {
            (func)(tab);
        }
        for button in &mut self.tab_buttons {
            (func)(button.as_widget());
        }
    }
}

pub struct TabWidget {
    pub body : Widget,
    pub tab_titles : Vec<String>,
    pub tabs : Vec<Widget>,
    pub tab_buttons : Vec<Button>,
}

impl TabWidget {
    pub fn new(tab_titles : Vec<&'static str>) -> TabWidget {
        let tab_titles = tab_titles.map(|str| String::from(*str));
        let num_tab_titles = tab_titles.len() as f32;
        let tab_bar_height = 3.ux();
        let body_color = Color::greyscale(0.8);
        let tab_buttons = tab_titles.iter().enumerate().map(|(i,t)| {
            let start_pcnt = i as f32 / num_tab_titles;
            let dim_pcnt = 1.0 / num_tab_titles;

            let border_sides = if i != 0 {
                BorderSides::two_sides(Alignment::Left, Alignment::Bottom)
            } else {
                BorderSides::none()
            };

            Button::new(t.clone())
                .position(Positioning::PcntOfParent(start_pcnt), Positioning::origin())
                .font_size(16)
                .text_position(Positioning::CenteredInParent, Positioning::CenteredInParent)
                .border(Border { width : 1, color : Color::black(), sides : border_sides })
                .color(body_color)
                .size(Sizing::PcntOfParent(dim_pcnt), Sizing::Constant(tab_bar_height))
        }).collect_vec();
        let tabs = tab_titles.iter().enumerate().map(|(i,t_)| {
            Widget::window(Color::clear(), 0)
                .position(Positioning::default(), Positioning::Constant(tab_bar_height))
                .size(Sizing::match_parent(), Sizing::DeltaOfParent(-tab_bar_height))
                .showing(i == 0)
        }).collect_vec();

        let body = Widget::window(body_color,2)
            .with_callback_2(|gui : &mut GUI, ctxt: &mut WidgetContext, evt : &UIEvent| {
                if let UIEvent::WidgetEvent(WidgetEvent::ButtonClicked(btn)) = evt {
                    let mut to_show : Vec<Wid> = Vec::new();
                    let mut to_hide : Vec<Wid> = Vec::new();
                    let mut to_show_bar : Vec<Wid> = Vec::new();
                    let mut to_hide_bar : Vec<Wid> = Vec::new();
                    let mut new_active_tab = 0;

                    if let WidgetState::Tab { ref tabs, ref tab_buttons, active_tab } = ctxt.widget_state {
                        new_active_tab = active_tab;
                        if let Some(clicked_tab_index) = tab_buttons.iter().position(|&w| w == *btn) {
                            let clicked_tab = tabs[clicked_tab_index];
                            to_show.push(clicked_tab);
                            to_hide_bar.push(tab_buttons[clicked_tab_index]);

                            to_hide = tabs.iter().filter(|tab| **tab != clicked_tab).cloned().collect_vec();
                            to_show_bar = tab_buttons.iter().enumerate().filter(|(i, button)| *i != clicked_tab_index).map(|(_,button)| button).cloned().collect_vec();
                        }
                    }

                    for wid in to_show {
                        gui.alter_widget(wid, |widget| {
                            widget.showing = true;
                        })
                    }
                    for wid in to_show_bar {
                        gui.alter_widget(wid, |widget| {
                            widget.border.sides = widget.border.sides.with_side(Alignment::Bottom);
                        })
                    }
                    for wid in to_hide {
                        gui.alter_widget(wid, |widget| {
                            widget.showing = false;
                            widget.border.sides = widget.border.sides.with_side(Alignment::Bottom);
                        })
                    }
                    for wid in to_hide_bar {
                        gui.alter_widget(wid, |widget| {
                            widget.border.sides = widget.border.sides.without_side(Alignment::Bottom);
                        })
                    }

                    let mut new_state = ctxt.widget_state.clone();
                    if let WidgetState::Tab { ref mut active_tab, .. } = new_state {
                        *active_tab = new_active_tab;
                    }
                    ctxt.update_state(new_state);
                }
            });

        TabWidget {
            body,
            tab_titles,
            tabs,
            tab_buttons
        }
    }

    pub fn tab_at_index(&self, i : usize) -> &Widget {
        & self.tabs[i]
    }
    pub fn tab_named<S>(&self, name : S) -> &Widget where S : Into<String> {
        let string : String = name.into();
        self.tab_at_index(self.tab_titles.iter().position(|t| *t == string).unwrap())
    }

    pub fn parent(mut self, parent : &Widget) -> Self {
        self.body = self.body.parent(parent);
        self
    }

    pub fn with_body<F : Fn(Widget) -> Widget>(mut self, func : F) -> Self {
        self.body = (func)(self.body);
        self
    }

    pub fn apply(mut self, gui : &mut GUI) -> Self {
        self.reapply(gui);
        self
    }
    pub fn reapply(&mut self, gui : &mut GUI) {
        self.body.reapply(gui);

        for tab_button in &mut self.tab_buttons {
            tab_button.set_parent(&self.body).reapply(gui);
        }
        for tab in &mut self.tabs {
            tab.set_parent(&self.body).reapply(gui);
        }

        if *gui.widget_state(self.body.id()) == WidgetState::NoState {
            self.body.state_override = Some(WidgetState::Tab { tab_buttons : self.tab_buttons.map(|b| b.id()), tabs : self.tabs.map(|t| t.id()), active_tab : 0 });
            self.body.reapply(gui);
        }
    }
}

impl Default for TabWidget {
    fn default() -> Self {
        TabWidget::new(Vec::new())
    }
}