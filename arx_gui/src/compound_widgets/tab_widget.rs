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

impl DelegateToWidget for TabWidget {
    fn as_widget(&mut self) -> &mut Widget {
        &mut self.body
    }

    fn as_widget_immut(&self) -> &Widget {
        &self.body
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
    pub body: Widget,
    pub tab_titles: Vec<String>,
    pub tabs: Vec<Widget>,
    pub tab_buttons: Vec<Button>,
}

impl TabWidget {
    pub fn new(tab_titles: Vec<&'static str>) -> TabWidget {
        let tab_titles = tab_titles.map(|str| String::from(*str));
        let num_tab_titles = tab_titles.len() as f32;
        let tab_bar_height = 3.ux();
        let body_color = Color::greyscale(0.8);
        let tab_buttons = tab_titles.iter().enumerate().map(|(i, t)| {
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
                .border(Border { width: 1, color: Color::black(), sides: border_sides })
                .color(body_color)
                .size(Sizing::PcntOfParent(dim_pcnt), Sizing::Constant(tab_bar_height))
        }).collect_vec();
        let tabs = tab_titles.iter().enumerate().map(|(i, t_)| {
            Widget::window(Color::clear(), 0)
                .position(Positioning::default(), Positioning::Constant(tab_bar_height))
                .size(Sizing::match_parent(), Sizing::DeltaOfParent(-tab_bar_height))
                .showing(i == 0)
        }).collect_vec();

        let body = Widget::window(body_color, 2)
            .with_callback_2(|gui: &mut GUI, ctxt: &mut WidgetContext, evt: &UIEvent| {
                if let UIEvent::WidgetEvent(WidgetEvent::ButtonClicked(btn)) = evt {
                    let mut to_show: Vec<Wid> = Vec::new();
                    let mut to_hide: Vec<Wid> = Vec::new();
                    let mut to_show_bar: Vec<Wid> = Vec::new();
                    let mut to_hide_bar: Vec<Wid> = Vec::new();
                    let mut new_active_tab = 0;

                    if let WidgetState::Tab { ref tabs, ref tab_buttons, active_tab } = ctxt.widget_state {
                        new_active_tab = active_tab;
                        if let Some(clicked_tab_index) = tab_buttons.iter().position(|&w| w == *btn) {
                            let clicked_tab = tabs[clicked_tab_index];
                            to_show.push(clicked_tab);
                            to_hide_bar.push(tab_buttons[clicked_tab_index]);

                            to_hide = tabs.iter().filter(|tab| **tab != clicked_tab).cloned().collect_vec();
                            to_show_bar = tab_buttons.iter().enumerate().filter(|(i, button)| *i != clicked_tab_index).map(|(_, button)| button).cloned().collect_vec();
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
            tab_buttons,
        }
    }

    pub fn tab_at_index(&self, i: usize) -> &Widget {
        &self.tabs[i]
    }
    pub fn tab_named<S>(&self, name: S) -> &Widget where S: Into<String> {
        let string: String = name.into();
        self.tab_at_index(self.tab_titles.iter().position(|t| *t == string).unwrap())
    }

    pub fn parent(mut self, parent: &Widget) -> Self {
        self.body = self.body.parent(parent);
        self
    }

    pub fn with_body<F: Fn(Widget) -> Widget>(mut self, func: F) -> Self {
        self.body = (func)(self.body);
        self
    }

    pub fn apply(mut self, gui: &mut GUI) -> Self {
        self.reapply(gui);
        self
    }
    pub fn reapply(&mut self, gui: &mut GUI) {
        self.body.reapply(gui);

        for tab_button in &mut self.tab_buttons {
            tab_button.set_parent(&self.body).reapply(gui);
        }
        for tab in &mut self.tabs {
            tab.set_parent(&self.body).reapply(gui);
        }

        if *gui.widget_state(self.body.id()) == WidgetState::NoState {
            self.body.state_override = Some(WidgetState::Tab { tab_buttons: self.tab_buttons.map(|b| b.id()), tabs: self.tabs.map(|t| t.id()), active_tab: 0 });
            self.body.reapply(gui);
        }
    }
}

impl Default for TabWidget {
    fn default() -> Self {
        TabWidget::new(Vec::new())
    }
}