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
use graphics::FontSize;

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

#[derive(Clone)]
pub struct TabWidget {
    pub body: Widget,
    pub tab_titles: Vec<String>,
    pub tabs: Vec<Widget>,
    pub tab_buttons: Vec<Button>,
    pub to_remove : Vec<Wid>
}

impl TabWidget {
    pub fn new<S : Into<String>>(tab_titles: Vec<S>) -> TabWidget {
        let body_color = Color::greyscale(0.8);

        let body = Widget::window(body_color, 2)
            .with_callback_2(|gui: &mut GUI, ctxt: &mut WidgetContext, evt: &UIEvent| {
                if let UIEvent::WidgetEvent{ event : WidgetEvent::ButtonClicked(btn), .. } = evt {
                    let mut to_show: Vec<Wid> = Vec::new();
                    let mut to_hide: Vec<Wid> = Vec::new();
                    let mut to_show_bar: Vec<Wid> = Vec::new();
                    let mut to_hide_bar: Vec<Wid> = Vec::new();
                    let mut new_active_tab = None;

                    if let WidgetState::Tab { ref tabs, ref tab_buttons, active_tab } = ctxt.widget_state {
                        new_active_tab = active_tab;
                        if let Some(tab) = active_tab {
                            if let Some(clicked_tab_index) = tab_buttons.iter().position(|&w| w == *btn) {
                                new_active_tab = Some(clicked_tab_index as u32);
                                let clicked_tab = tabs[clicked_tab_index];
                                to_show.push(clicked_tab);
                                to_hide_bar.push(tab_buttons[clicked_tab_index]);

                                to_hide = tabs.iter().filter(|tab| **tab != clicked_tab).cloned().collect_vec();
                                to_show_bar = tab_buttons.iter().enumerate().filter(|(i, button)| *i != clicked_tab_index).map(|(_, button)| button).cloned().collect_vec();
                            }
                        }
                    }

                    for wid in to_show {
                        gui.alter_widget(wid, |widget| { widget.set_showing(true); })
                    }
                    for wid in to_show_bar {
                        gui.alter_widget(wid, |widget| { widget.border.sides = widget.border.sides.with_side(Alignment::Bottom); })
                    }
                    for wid in to_hide {
                        gui.alter_widget(wid, |widget| {
                            let new_sides = widget.border.sides.with_side(Alignment::Bottom);
                            widget.set_showing(false).set_border_sides(new_sides);
                        })
                    }
                    for wid in to_hide_bar {
                        gui.alter_widget(wid, |widget| { widget.set_border_sides(widget.border.sides.without_side(Alignment::Bottom)); })
                    }

                    let mut new_state = ctxt.widget_state.clone();
                    if let WidgetState::Tab { ref mut active_tab, .. } = new_state {
                        *active_tab = new_active_tab;
                    }
                    ctxt.update_state(new_state);
                }
            });

        let mut tab_widget = TabWidget {
            body,
            tab_titles : Vec::new(),
            tabs : Vec::new(),
            tab_buttons : Vec::new(),
            to_remove : Vec::new(),
        };

        tab_widget.set_tabs(tab_titles);

        tab_widget
    }

    pub fn set_tabs<S : Into<String>>(&mut self, tab_titles: Vec<S>) -> &mut Self {
        let tab_titles : Vec<String> = tab_titles.into_iter().map(|s| s.into()).collect_vec();
        let num_tab_titles = tab_titles.len() as f32;
        let tab_bar_height = 3.ux();

        if tab_titles.len() != self.tab_titles.len() {
            while self.tab_buttons.len() > tab_titles.len() { if let Some(button) = self.tab_buttons.pop() { self.to_remove.push(button.id()); } }
            while self.tabs.len() > tab_titles.len() { if let Some(tab) = self.tabs.pop() { self.to_remove.push(tab.id()); } }

            while self.tab_buttons.len() < tab_titles.len() {
                let title_index = self.tab_buttons.len();
                self.tab_buttons.push(Button::new(tab_titles[title_index].clone())
                    .font_size(FontSize::HeadingMajor)
                    .text_position(Positioning::CenteredInParent, Positioning::CenteredInParent)
                    .color(self.body.color)
                );
            }
            while self.tabs.len() < tab_titles.len() {
                let title_index = self.tabs.len();
                self.tabs.push(Widget::window(Color::clear(), 0)
                    .position(Positioning::default(), Positioning::Constant(tab_bar_height))
                    .size(Sizing::match_parent(), Sizing::DeltaOfParent(-tab_bar_height))
                    .showing(title_index == 0))
            }

            self.tab_buttons.iter_mut().enumerate().foreach(|(i, b)| {
                let start_pcnt = i as f32 / num_tab_titles;
                let dim_pcnt = 1.0 / num_tab_titles;

                let border_sides = if i != 0 {
                    BorderSides::two_sides(Alignment::Left, Alignment::Bottom)
                } else {
                    BorderSides::none()
                };
                b.set_position(Positioning::PcntOfParent(start_pcnt), Positioning::origin())
                    .set_size(Sizing::PcntOfParent(dim_pcnt), Sizing::Constant(tab_bar_height))
                    .set_border(Border { width: 1, color: Color::black(), sides: border_sides });
            });
        }

        if tab_titles != self.tab_titles {
            self.tab_buttons.iter_mut().zip(tab_titles.iter()).foreach(|(button, title)| {
                button.set_text(title.clone());
            })
        }
        self.tab_titles = tab_titles;

        self
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

    pub fn apply_all(mut self, gui: &mut GUI) -> Self {
        self.reapply_all(gui);
        self
    }
    pub fn reapply_all(&mut self, gui: &mut GUI) {
        self.body.reapply(gui);

        for wid in &self.to_remove {
            gui.remove_widget_by_id(*wid);
        }
        self.to_remove.clear();

        for tab_button in &mut self.tab_buttons {
            tab_button.set_parent(&self.body).reapply(gui);
        }
        for tab in &mut self.tabs {
            tab.set_parent(&self.body).reapply(gui);
        }

        if *gui.widget_state(self.body.id()) == WidgetState::NoState {
            let active_tab = if self.tabs.is_empty() {
                None
            } else {
                Some(0)
            };
            self.body.state_override = Some(WidgetState::Tab { tab_buttons: self.tab_buttons.map(|b| b.id()), tabs: self.tabs.map(|t| t.id()), active_tab });
            self.body.reapply(gui);
        }
    }

    pub fn active_tab(&self, gui : &GUI) -> Option<u32> {
        match gui.widget_state(self.body.id()) {
            WidgetState::Tab { active_tab , .. } => *active_tab,
            _ => None
        }
    }
}

impl Default for TabWidget {
    fn default() -> Self {
        TabWidget::new(Vec::<Str>::new())
    }
}