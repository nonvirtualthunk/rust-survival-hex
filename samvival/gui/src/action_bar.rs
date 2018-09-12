use common::prelude::*;
use game::prelude::*;
use gui::*;
use game::entities::actions::*;
use common::color::Color;
use game::Entity;
use std::collections::HashMap;
use state::GameState;
use state::ControlContext;
use control_events::TacticalEvents;
use game::entities::Attack;
use game::entities::AttackRef;
use game::entities::MovementType;
use game::entities::MovementTypeRef;
use graphics::FontSize;


#[derive(PartialEq, Clone, Debug, Hash)]
pub enum PlayerActionType {
    MoveAndAttack(MovementTypeRef, AttackRef),
    Harvest,
    InteractWithInventory,
    Move(MovementTypeRef),
    Wait,
    None,
}

impl PlayerActionType {
    pub fn name(&self, world : &WorldView, character : Entity) -> String {
        match self {
            PlayerActionType::MoveAndAttack(move_ref, attack_ref) => format!("Move and {}", attack_ref.resolve(world, character).map(|a| a.name.capitalized()).unwrap_or_else(|| String::from("Attack"))),
            PlayerActionType::InteractWithInventory => String::from("Open Inventory"),
            PlayerActionType::Move(move_ref) => move_ref.resolve(world).map(|mt| mt.name.capitalized()).unwrap_or_else(|| String::from("Unknown move type")),
            PlayerActionType::Wait => String::from("Wait"),
            PlayerActionType::Harvest => String::from("Harvest"),
            PlayerActionType::None => String::from("None"),
        }
    }
    pub fn description(&self, world : &WorldView, character : Entity) -> String {
        match self {
            PlayerActionType::MoveAndAttack(move_ref, attack_ref) => format!("Move (if necessary) and {} an enemy", attack_ref.resolve(world, character).map(|a| a.verb.unwrap_or(a.name)).unwrap_or(String::from("Attack"))),
            PlayerActionType::InteractWithInventory => format!("Transfer items from your inventory to or from another. Can be used to drop items on the ground or pick them up."),
            PlayerActionType::Move(move_ref) => format!("{} across terrain to another location", move_ref.resolve(world).map(|m| m.name.as_str()).unwrap_or("move")),
            PlayerActionType::Wait => format!("Do nothing for the moment"),
            PlayerActionType::Harvest => format!("Harvest resources from a nearby hex."),
            PlayerActionType::None => String::from("None"),
        }
    }
    pub fn icon(&self, world : &WorldView, character : Entity) -> String {
        match self {
            PlayerActionType::MoveAndAttack(move_ref, attack_ref) => format!("ui/attack_icon"),
            PlayerActionType::InteractWithInventory => format!("ui/interact_with_inventory_icon"),
            PlayerActionType::Move(move_ref) => format!("ui/move_icon"),
            PlayerActionType::Wait => format!("ui/clock_icon"),
            PlayerActionType::Harvest => format!("ui/harvest_icon"),
            PlayerActionType::None => String::from("None"),
        }
    }
}


#[derive(Default)]
pub struct ActionBar {
    pub action_list : ListWidget<ActionButton>,
    pub actions : Vec<PlayerActionType>,
    pub selected_actions : HashMap<Entity, PlayerActionType>
}
impl DelegateToWidget for ActionBar {
    fn as_widget(&mut self) -> &mut Widget { self.action_list.as_widget() }

    fn as_widget_immut(&self) -> &Widget { self.action_list.as_widget_immut() }
}

#[derive(WidgetContainer)]
pub struct ActionButton {
    pub icon : Widget,
    pub info_body : Widget,
    pub info_name : Widget,
    pub info_description : Widget
}

impl Default for ActionButton {
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
        let info_name = Widget::text("Name: ", FontSize::HeadingMinor).parent(&info_body).named("info name");
        let info_description = Widget::wrapped_text("Description: ", FontSize::Standard, TextWrap::WithinParent).parent(&info_body).named("info description")
            .y(Positioning::below(&info_name, 1.ux()));

        let info_body_id = info_body.id();
        let icon_id = icon.id();
        let icon = icon.with_callback(move |ctxt : &mut WidgetContext, evt : &UIEvent| {
            if let UIEvent::MouseEntered { to_widget, .. } = evt {
                ctxt.alter_widget(info_body_id, |w| { w.set_showing(true); });
            } else if let UIEvent::MouseExited { from_widget, .. } = evt {
                ctxt.alter_widget(info_body_id, |w| { w.set_showing(false); });
            }
        }).only_consume(EventConsumption::none());

        ActionButton { icon, info_body, info_name, info_description }
    }
}

impl ActionBar {
    pub fn new(gui : &mut GUI, parent : &Widget) -> ActionBar {
        let mut action_list = ListWidget::featherweight()
            .alignment(Alignment::Bottom, Alignment::Left)
            .position(Positioning::constant(2.ux()), Positioning::constant(2.ux()))
            .size(Sizing::surround_children(), Sizing::surround_children())
            .widget_type(WidgetType::window())
            .horizontal()
            .named("Action bar list")
            .only_consume(EventConsumption::all())
            .parent(parent)
            .apply(gui);

        action_list.item_archetype.set_size(Sizing::surround_children(), Sizing::surround_children()).set_color(Color::greyscale(0.8));

        ActionBar {
            actions : Vec::new(),
            action_list,
            selected_actions : HashMap::new()
        }
    }

    pub fn update(&mut self, gui : &mut GUI, view : &WorldView, actions : Vec<PlayerActionType>, game_state : &GameState, control_context : &mut ControlContext) {
        if let Some(selected_char) = game_state.selected_character {
            self.action_list.set_showing(true).reapply(gui);

            let mut selection_changed = false;
            for event in gui.events_for(self.action_list.as_widget_immut()) {
                if let UIEvent::WidgetEvent{ event, .. } = event {
                    if let WidgetEvent::ListItemClicked(index, button) = event {
                        let action_type = self.actions[*index].clone();
                        self.selected_actions.insert(selected_char, action_type.clone());
                        control_context.event_bus.push_event(TacticalEvents::ActionSelected(action_type));
                        selection_changed = true;
                    }
                }
            }

            if selection_changed || self.actions != actions {
                self.actions = actions;
                let selected_action = self.selected_action_for(view, selected_char);
                self.action_list.update(gui, self.actions.as_ref(), |action_button, action| {
                    action_button.icon.set_widget_type(WidgetType::image(action.icon(view, selected_char)));
                    if action == &selected_action {
                        action_button.icon.set_border(Border { color : Color::new(0.1,0.7,0.1,1.0), sides : BorderSides::all(), width : 2});
                    } else {
                        action_button.icon.set_border(Border { color : Color::black(), sides : BorderSides::all(), width : 2});
                    }
                    action_button.info_name.set_text(format!("{}", action.name(view, selected_char)));
                    action_button.info_description.set_text(format!("{}", action.description(view, selected_char)));
                });
            }

        } else {
            self.action_list.clear(gui);
        }
    }

    pub fn selected_action_for(&self, view : &WorldView, character : Entity) -> PlayerActionType {
        use game::logic;

        if let Some(selected) = self.selected_actions.get(&character).cloned() {
            return selected;
        } else if let Some(default_move) = logic::movement::default_movement_type(view, character) {
            if let Some(default_attack) = logic::combat::primary_attack_ref(view, character) {
                return PlayerActionType::MoveAndAttack(default_move, default_attack);
            }
        }

        PlayerActionType::Wait
    }

    pub fn set_selected_action_for(&mut self, world_ : &World, character : Entity, action : PlayerActionType) {
        self.selected_actions.insert(character, action);
    }
}