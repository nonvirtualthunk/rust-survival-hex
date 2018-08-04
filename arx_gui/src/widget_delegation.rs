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
use compound_widgets::TextDisplayWidget;

pub trait DelegateToWidget where Self: Sized {
    fn id(&self) -> Wid {
        self.as_widget_immut().id
    }
    fn signifier(&self) -> String {
        if let Some(name) = self.as_widget_immut().name {
            String::from(name)
        } else {
            format!("{:?}", self.id())
        }
    }
    fn named(mut self, name: Str) -> Self {
        self.as_widget().name = Some(name);
        self
    }
    fn draw_layer(mut self, layer: GUILayer) -> Self {
        self.as_widget().draw_layer = layer;
        self
    }
    fn position(mut self, x: Positioning, y: Positioning) -> Self {
        self.set_position(x, y);
        self
    }
    fn centered(self) -> Self {
        self.position(Positioning::centered(), Positioning::centered())
    }
    fn set_position(&mut self, x: Positioning, y: Positioning) -> &mut Self {
        self.as_widget().position = [x, y];
        self
    }
    fn size(mut self, w: Sizing, h: Sizing) -> Self {
        self.set_size(w, h);
        self
    }
    fn fixed_size(mut self, w: UIUnits, h: UIUnits) -> Self {
        self.set_size(Sizing::constant(w), Sizing::constant(h));
        self
    }
    fn parent(mut self, p: &Widget) -> Self {
        self.set_parent(p);
        self
    }
    fn set_parent(&mut self, p: &Widget) -> &mut Self {
        self.set_parent_id(p.id);
        self
    }
    fn ignore_parent_bounds(mut self) -> Self {
        self.as_widget().ignores_parent_bounds = true;
        self
    }
    fn color(mut self, c: Color) -> Self {
        self.set_color(c);
        self
    }
    fn set_color(&mut self, c: Color) -> &mut Self {
        self.as_widget().color = c;
        self
    }

    fn widget_type(mut self, widget_type: WidgetType) -> Self {
        self.set_widget_type(widget_type);
        self
    }
    fn set_widget_type(&mut self, widget_type: WidgetType) -> &mut Self {
        self.as_widget().widget_type = widget_type;
        self
    }
    fn set_text<S: Into<String>>(&mut self, text: S) -> &mut Self {
        self.as_widget().widget_type.set_text(text);
        self
    }

    fn modify_widget_type<F: Fn(&mut WidgetType)>(&mut self, func: F) -> &mut Self {
        (func)(&mut self.as_widget().widget_type);
        self
    }

    fn showing(mut self, showing: bool) -> Self {
        self.set_showing(showing);
        self
    }
    fn set_showing(&mut self, showing: bool) -> &mut Self {
        self.as_widget().showing = showing;
        self
    }

    fn set_border(&mut self, border: Border) -> &mut Self {
        self.as_widget().border = border;
        self
    }
    fn border(mut self, border: Border) -> Self {
        self.set_border(border);
        self
    }

    fn border_width(mut self, border_width: u8) -> Self {
        self.as_widget().border.width = border_width;
        self
    }
    fn border_color(mut self, border_color: Color) -> Self {
        self.as_widget().border.color = border_color;
        self
    }
    fn border_sides(mut self, border_sides: BorderSides) -> Self {
        self.as_widget().border.sides = border_sides;
        self
    }

    fn parent_id(mut self, parent: Wid) -> Self {
        self.set_parent_id(parent);
        self
    }
    fn set_parent_id(&mut self, parent_id: Wid) -> &mut Self {
        if parent_id == NO_WID {
            error!("Attempting to add a widget to a parent that has no ID, this is not acceptable:\n{:?}", Backtrace::new());
        }
        self.as_widget().parent_id = Some(parent_id);
        self
    }

    fn set_x(&mut self, x: Positioning) -> &mut Self {
        self.as_widget().position[0] = x;
        self
    }
    fn set_size(&mut self, w: Sizing, h: Sizing) -> &mut Self {
        self.as_widget().size = [w, h];
        self
    }
    fn margin(mut self, margin: UIUnits) -> Self {
        self.set_margin(margin);
        self
    }
    fn set_margin(&mut self, margin: UIUnits) -> &mut Self {
        self.as_widget().margin = margin;
        self
    }
    fn x(mut self, x: Positioning) -> Self {
        self.set_x(x);
        self
    }
    fn y(mut self, y: Positioning) -> Self {
        self.set_y(y);
        self
    }
    fn set_y(&mut self, y: Positioning) -> &mut Self {
        self.as_widget().position[1] = y;
        self
    }
    fn alignment(mut self, x: Alignment, y: Alignment) -> Self {
        if (x == Alignment::Top || x == Alignment::Bottom) && (y == Alignment::Left || y == Alignment::Right) {
            self.as_widget().alignment = [y, x];
        } else {
            self.as_widget().alignment = [x, y];
        }
        self
    }
    fn width(mut self, w: Sizing) -> Self {
        self.set_width(w);
        self
    }
    fn set_width(&mut self, w: Sizing) -> &mut Self {
        self.as_widget().size[0] = w;
        self
    }
    fn height<S: Into<Sizing>>(mut self, h: S) -> Self {
        self.set_height(h.into());
        self
    }
    fn set_height(&mut self, h: Sizing) -> &mut Self {
        self.as_widget().size[1] = h;
        self
    }
    fn only_consume(mut self, consumption: EventConsumption) -> Self {
        self.as_widget().event_consumption = consumption;
        self
    }
    fn and_consume(&mut self, consumption: EventConsumption) -> &mut Self {
        let new_consumption = {
            let old_consumption = &self.as_widget().event_consumption;
            old_consumption.and(&consumption)
        };
        self.as_widget().event_consumption = new_consumption;
        self
    }
    fn accepts_focus(mut self, accept: bool) -> Self {
        self.as_widget().accepts_focus = accept;
        self
    }

    fn with_child(self, mut child_widget: Widget) -> Self {
        child_widget.set_parent_id(self.id());
        self
    }

    fn with_cleared_callbacks(mut self) -> Self {
        self.clear_callbacks();
        self
    }
    fn clear_callbacks(&mut self) {
        self.as_widget().clear_callbacks();
    }
    fn with_callback<State: 'static, U: Fn(&mut State, &UIEvent) + 'static>(mut self, function: U) -> Self {
        self.as_widget().add_callback(function);
        self
    }
    fn add_callback<State: 'static, U: Fn(&mut State, &UIEvent) + 'static>(&mut self, function: U) -> &mut Self {
        self.as_widget().add_callback(function);
        self
    }
    fn with_callback_2<State: 'static, OtherState: 'static, U: Fn(&mut State, &mut OtherState, &UIEvent) + 'static>(mut self, function: U) -> Self {
        self.as_widget().add_callback_2(function);
        self
    }

    fn set_tooltip<S: Into<String>>(&mut self, string: S) -> &mut Self {
        let string: String = string.into();
        self.as_widget().tooltip = Some(string);
//
//        if let Some(tooltip) = self.as_widget().tooltip {
//            let tooltip_children = tooltip.for_all_widgets()
//            let text_portion = tooltip_children.find(|c| if let WidgetType::Text { .. } = c.widget_type { true } else { false });
//            if let Some(tooltip_text) = text_portion {
//                if let WidgetType::Text { ref text, .. } = tooltip_text.widget_type {
//                    if text != &string {
//                        info!("Tooltip text set to something new, from {} to {}, updating.", text, string);
//                        gui.alter_widget(tooltip_text.id(), |w| { w.set_text(string.clone()); });
//                    }
//                } else {
//                    error!("Somehow the text widget no longer has a text widget type? That should be logically impossible");
//                }
//                return self;
//            } else {
//                info!("Tooltip existed, but it didn't follow expected form, had to destroy and recreate");
//                gui.remove_widget_by_id(tooltip.id());
//            }
//        }
//
//        let mut tdw = TextDisplayWidget::new(string.clone(), 14, None, ImageSegmentation::None)
//            .parent_id(self.id())
//            .draw_layer_for_all(GUILayer::Overlay)
//            .ignore_parent_bounds()
//            .position(Positioning::Constant(1.ux()), Positioning::Constant(1.ux()))
//            .named("tooltip")
//            .showing(false);
//
//        if string.len() > 40 {
//            tdw = tdw.wrapped(Sizing::Constant(20.ux()));
//            tdw.text.set_x(Positioning::centered());
//        }
//
//        let tdw_id = tdw.id();
//        tdw.apply(gui);
//
//        self.add_callback(move |ctxt: &mut WidgetContext, evt: &UIEvent| {
//            match evt {
//                UIEvent::HoverStart { pos, .. } => {
//                    let (x, y) = ((pos.absolute_pos.x + 1.0).ux(), (pos.absolute_pos.y + 1.0).ux());
//                    ctxt.alter_widget(tdw_id, move |w| {
//                        w.set_showing(true);
//                        w.set_position(Positioning::absolute(x), Positioning::absolute(y));
//                    })
//                }
//                UIEvent::HoverEnd { .. } => ctxt.alter_widget(tdw_id, |w| { w.set_showing(false); }),
//                _ => ()
//            }
//        }).and_consume(EventConsumption::EventTypes(HOVER_START.bit_flag | HOVER_END.bit_flag));

        self
    }
    fn with_tooltip<S: Into<String>>(mut self, string: S) -> Self {
        self.set_tooltip(string);
        self
    }


    fn as_widget(&mut self) -> &mut Widget;
    fn as_widget_immut(&self) -> &Widget;
}