use gui::*;
use game::entities::reactions::*;
use common::color::Color;
use game::Entity;
use std::collections::HashMap;
use tactical_gui::GameState;
use tactical_gui::ControlContext;
use control_events::ControlEvents;

#[derive(Default)]
pub struct ReactionBar {
    pub reaction_list: ListWidget<ReactionButton>,
    pub reactions: Vec<ReactionType>,
    pub last_selected_reaction: Option<ReactionType>,
}

impl DelegateToWidget for ReactionBar {
    fn as_widget(&mut self) -> &mut Widget { self.reaction_list.as_widget() }

    fn as_widget_immut(&self) -> &Widget { self.reaction_list.as_widget_immut() }
}

#[derive(WidgetContainer)]
pub struct ReactionButton {
    pub icon: Widget,
    pub info_body: Widget,
    pub info_name: Widget,
    pub info_description: Widget,
}

impl Default for ReactionButton {
    fn default() -> Self {
        let icon = Button::image_button(String::from("ui/blank")).fixed_size(34.px(), 34.px()).named("action bar icon");
        let info_body = Widget::window(Color::greyscale(0.85), 1)
            .named("info body")
            .parent(&icon)
            .ignore_parent_bounds()
            .y(Positioning::above(&icon, 2.ux()))
            .alignment(Alignment::Top, Alignment::Left)
            .size(Sizing::constant(30.ux()), Sizing::constant(30.ux()))
            .margin(1.ux())
            .showing(false);
        let info_name = Widget::text("Name: ", 14).parent(&info_body).named("info name");
        let info_description = Widget::wrapped_text("Description: ", 14, TextWrap::WithinParent).parent(&info_body).named("info description")
            .y(Positioning::below(&info_name, 1.ux()));

        let info_body_id = info_body.id();
        let icon_id = icon.id();
        let icon = icon.with_callback(move |ctxt: &mut WidgetContext, evt: &UIEvent| {
            if let UIEvent::MouseEntered { to_widget, .. } = evt {
                ctxt.alter_widget(info_body_id, |w| { w.set_showing(true); });
            } else if let UIEvent::MouseExited { from_widget, .. } = evt {
                ctxt.alter_widget(info_body_id, |w| { w.set_showing(false); });
            }
        }).only_consume(EventConsumption::none());

        ReactionButton { icon, info_body, info_name, info_description }
    }
}

impl ReactionBar {
    pub fn new(gui: &mut GUI) -> ReactionBar {
        let mut action_list = ListWidget::featherweight()
            .alignment(Alignment::Bottom, Alignment::Left)
            .position(Positioning::constant(2.ux()), Positioning::constant(2.ux()))
            .size(Sizing::surround_children(), Sizing::surround_children())
            .widget_type(WidgetType::window())
            .horizontal()
            .named("Reaction bar list")
            .only_consume(EventConsumption::all())
            .apply(gui);

        action_list.item_archetype.set_size(Sizing::surround_children(), Sizing::surround_children()).set_color(Color::greyscale(0.8));

        ReactionBar {
            reactions: Vec::new(),
            reaction_list: action_list,
            last_selected_reaction: None
        }
    }

    pub fn update(&mut self, gui: &mut GUI, reactions: Vec<ReactionType>, selected_reaction: ReactionType,
                  game_state: &GameState, control_context: ControlContext) {
        if let Some(selected_char) = game_state.selected_character {
            self.reaction_list.set_showing(true).reapply(gui);

            for event in gui.events_for(self.reaction_list.as_widget_immut()) {
                if let UIEvent::WidgetEvent(wevent) = event {
                    if let WidgetEvent::ListItemClicked(index) = wevent {
                        let reaction_type = self.reactions[*index].clone();
                        control_context.event_bus.push_event(ControlEvents::ReactionSelected(reaction_type));
                    }
                }
            }

            if self.reactions != reactions || self.last_selected_reaction.as_ref() != Some(&selected_reaction) {
                self.reactions = reactions;
                self.last_selected_reaction = Some(selected_reaction.clone());

                let selected_name = selected_reaction.name;

                self.reaction_list.update(gui, self.reactions.as_ref(), |action_button, reaction| {
                    action_button.icon.set_widget_type(WidgetType::image(reaction.icon));
                    if reaction.name == selected_name {
                        action_button.icon.set_border(Border { color: Color::new(0.1, 0.7, 0.1, 1.0), sides: BorderSides::all(), width: 2 });
                    } else {
                        action_button.icon.set_border(Border { color: Color::black(), sides: BorderSides::all(), width: 2 });
                    }
                    action_button.info_name.set_text(format!("{}", reaction.name));
                    action_button.info_description.set_text(format!("{}", reaction.description));
                });
            }
        } else {
            self.reaction_list.clear(gui);
        }
    }
}