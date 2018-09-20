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
use events::MouseButton;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use ui_event_types::*;
use widgets::*;
use widget_delegation::DelegateToWidget;

#[derive(Clone)]
pub struct ListWidget<T: Default> {
    pub body: Widget,
    pub item_archetype: Widget,
    pub item_gap: UIUnits,
    pub child_structs: Vec<T>,
    pub children: Vec<Widget>,
    pub children_to_remove: Vec<Widget>,
    pub orientation: Orientation,
}

impl<T: Default + WidgetContainer> ListWidget<T> {
    pub fn new() -> ListWidget<T> {
        let item = Widget::window(Color::greyscale(0.7), 1)
            .size(Sizing::match_parent(), Sizing::SurroundChildren)
            .named("List widget item archetype");
        ListWidget::custom(item, 2.px())
    }

    /// creates a list widget without explicit sectioning
    pub fn lightweight() -> ListWidget<T> {
        let item = Widget::div()
            .size(Sizing::PcntOfParent(1.0), Sizing::SurroundChildren);
        ListWidget::custom(item, 2.px())
            .margin(2.px())
    }

    /// creates a list widget with no background or sectioning, it exists only to arrange its children
    pub fn featherweight() -> ListWidget<T> {
        ListWidget::lightweight()
            .color(Color::clear())
            .margin(0.px())
            .border_width(0)
            .height(Sizing::surround_children())
    }

    pub fn custom(item_archetype: Widget, item_gap: UIUnits) -> ListWidget<T> {
        ListWidget {
            body: Widget::window(Color::greyscale(0.8), 2).margin(2.px()),
            item_archetype,
            child_structs: Vec::new(),
            children: Vec::new(),
            children_to_remove: Vec::new(),
            item_gap,
            orientation: Orientation::Vertical,
        }
    }

    pub fn parent(mut self, parent: &Widget) -> Self {
        self.body = self.body.parent(parent);
        self
    }

    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }

    pub fn item_gap(mut self, gap : UIUnits) -> Self {
        self.item_gap = gap;
        self
    }

    pub fn rows_surround_children(mut self) -> Self {
        self.item_archetype.set_size(Sizing::surround_children(), Sizing::surround_children());
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

    pub fn with_items<U, F: Fn(&mut T, &U)>(mut self, gui: &mut GUI, data: &[U], func: F) -> Self {
        self.update(gui, data, func);
        self
    }

    pub fn with_body<F: Fn(Widget) -> Widget>(mut self, func: F) -> Self {
        self.body = (func)(self.body);
        self
    }

    pub fn clear(&mut self, gui: &mut GUI) -> &mut Self {
        self.update(gui, &[], |t, u: &i32| {})
    }

    /// lambda takes |widget, data|
    pub fn update<U, F: FnMut(&mut T, &U)>(&mut self, gui: &mut GUI, data: &[U], mut func: F) -> &mut Self {
        self.update_with_row(gui, data, move |t : &mut T, u : &U, _ : &mut Widget| { func(t,u) })
    }

    pub fn update_with_row<U, F: FnMut(&mut T, &U, &mut Widget)>(&mut self, gui: &mut GUI, data: &[U], mut func: F) -> &mut Self {
        let draw_layer = self.as_widget_immut().draw_layer;

        // make sure the body exists. Todo: only actually need to do this the first time
        self.body.reapply(gui);
        let body_id = self.body.id();

        while data.len() > self.children.len() {
            let index = self.children.len();
            let mut new_item = self.item_archetype.clone().parent(&self.body)
                .draw_layer(draw_layer)
                .with_callback(move |ctxt: &mut WidgetContext, evt: &UIEvent| {
                    if let UIEvent::MouseRelease { button, .. } = evt {
                        trace!("Triggering list item event");
                        ctxt.trigger_event(UIEvent::widget_event(WidgetEvent::ListItemClicked(index, *button), ctxt.widget_id()));
                    }
                });
            new_item.clear_id();

            match self.orientation {
                Orientation::Vertical => {
                    new_item.position[1] = match self.children.last() {
                        Some(prev) => Positioning::below(prev, self.item_gap),
                        None => Positioning::origin()
                    };
                }
                Orientation::Horizontal => {
                    new_item.position[0] = match self.children.last() {
                        Some(prev) => Positioning::right_of(prev, self.item_gap),
                        None => Positioning::origin()
                    }
                }
                _ => error!("Somehow conspired to get a list widget with a non-horizontal, non-vertical orientation. That doesn't work.")
            }

            new_item.reapply(gui);

            self.children.push(new_item);
            self.child_structs.push(T::default());
        }

        for (i, value) in data.iter().enumerate() {
            let child = &mut self.children[i];
            let child_id = child.id();
            func(&mut self.child_structs[i], &value, child);
            ListWidget::auto_apply(child_id, gui, &mut self.child_structs[i], draw_layer);
        }

        while data.len() < self.children.len() {
            let child = self.children.pop().expect("children can't be empty, that would indicate that data.len() < 0");
            self.children_to_remove.push(child);
            self.child_structs.pop();
        }
        self.reapply_children(gui);

        self
    }

    fn auto_apply(id: Wid, gui: &mut GUI, child_struct: &mut T, draw_layer : GUILayer) {
        child_struct.for_each_widget(|w| {
            w.set_draw_layer(draw_layer);
            if w.parent_id.is_none() {
                w.set_parent_id(id);
            }
        });
        child_struct.reapply_all(gui);
    }

    pub fn reapply_children(&mut self, gui: &mut GUI) {
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

impl<T: Default + WidgetContainer> Default for ListWidget<T> {
    fn default() -> Self {
        ListWidget::new()
    }
}

impl<T: Default> DelegateToWidget for ListWidget<T> {
    fn as_widget(&mut self) -> &mut Widget {
        &mut self.body
    }

    fn as_widget_immut(&self) -> &Widget {
        &self.body
    }
}

impl<T: Default> WidgetContainer for ListWidget<T> {
    fn for_each_widget<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
        (func)(&mut self.body);
        for child in &mut self.children {
            (func)(child);
        }
    }
}
