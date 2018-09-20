use gui::*;
use game::logic::combat::*;
use common::prelude::*;
use game::prelude::*;
use common::color::Color;
use game::logic::combat;
use game::entities::combat::Attack;
use game::entities::combat::AttackRef;
use state::ControlContext;
use control_events::TacticalEvents;
use common::hex::AxialCoord;
use game::logic::breakdown::Breakdown;

pub struct AttackDescriptionsWidget {
    attack_list: ListWidget<AttackDescriptionWidget>
}

impl DelegateToWidget for AttackDescriptionsWidget {
    fn as_widget(&mut self) -> &mut Widget { self.attack_list.as_widget() }

    fn as_widget_immut(&self) -> &Widget { self.attack_list.as_widget_immut() }
//    fn for_each_widget<F: FnMut(&mut Widget)>(&mut self, func: F) {
//        self.attack_list.for_each_widget(func);
//    }

}

impl AttackDescriptionsWidget {
    pub fn new(parent: &Widget) -> AttackDescriptionsWidget {
        let mut attack_list = ListWidget::new()
            .size(Sizing::match_parent(), Sizing::surround_children())
            .parent(parent)
            .and_consume(EventConsumption::mouse_events())
            .named("attack descriptions list");

        attack_list.item_archetype.set_margin(1.ux());
        AttackDescriptionsWidget { attack_list }
    }


    pub fn update(&mut self, gui: &mut GUI, view: &WorldView, character: Entity, control: &mut ControlContext) {
        let attack_refs = possible_attack_refs(view, character);
        let active_attack = primary_attack_ref(view, character);

        let counter_to_use = combat::counter_attack_ref_to_use(view, character);

        self.attack_list.update(gui, attack_refs.as_ref(), |widget, attack_ref| {
            if let Some(attack) = attack_ref.resolve(view, character) {
                widget.update(&attack, attack_ref.identity(view), active_attack.as_ref() == Some(attack_ref), counter_to_use.as_ref() == Some(attack_ref));
            } else {
                widget.clear();
            }
        });

        for event in gui.events_for(self.attack_list.as_widget_immut()) {
            if let UIEvent::WidgetEvent{ event, .. } = event {
                if let WidgetEvent::ListItemClicked(index, button) = event {
                    if let Some(attack_ref) = attack_refs[*index].as_option() {
                        match button {
                            MouseButton::Left => control.event_bus.push_event(TacticalEvents::AttackSelected(attack_ref.clone())),
                            _ => control.event_bus.push_event(TacticalEvents::CounterattackSelected(attack_ref.clone()))
                        }
                    } else {
                        println!("attack didn't resolve when trying to select");
                    }
                }
            }
        }
    }
}

#[derive(WidgetContainer, Clone)]
pub struct AttackDescriptionWidget {
    pub counter_indicator: Widget,
    pub active_indicator: Widget,
    pub name: Widget,
    pub to_hit: Widget,
    pub damage: Widget,
}

impl Default for AttackDescriptionWidget {
    fn default() -> Self {
        let counter_indicator = Widget::image("ui/active_counterattack_indicator", Color::new(0.8, 0.65, 0.1, 1.0), 0)
            .x(Positioning::origin())
            .y(Positioning::constant(4.px()))
            .size(Sizing::constant(12.px()), Sizing::constant(12.px()))
            .with_tooltip("This will be used for counter-attacks.")
            .named("counterattack indicator");

        let active_indicator = Widget::image("ui/active_attack_indicator", Color::new(0.1, 0.5, 0.1, 1.0), 0)
            .x(Positioning::origin())
            .y(Positioning::constant(4.px()))
            .size(Sizing::constant(12.px()), Sizing::constant(12.px()))
            .with_tooltip("This will be used when attacking, click another to select it instead")
            .named("active attack indicator");

        let name = Widget::text("Name here", FontSize::Standard)
            .x(Positioning::constant(13.px()))
            .with_tooltip("The name of the attack")
            .named("attack description name");
        let to_hit = Widget::text("To hit here", FontSize::Standard)
            .y(Positioning::below(&name, 2.px()))
            .x(Positioning::constant(13.px()))
            .with_tooltip("Bonus applied to every roll to determine if the attack hits")
            .named("attack to hit text");
        let damage = Widget::text("Damage here", FontSize::Standard)
            .x(Positioning::constant(13.px()))
            .y(Positioning::below(&to_hit, 2.px()))
            .with_tooltip("The amount and kind of damage done on a successful hit. \
            May be modified by the defender's armor, resistances and special abilities")
            .named("attack description damage text");

        AttackDescriptionWidget { name, to_hit, damage, active_indicator, counter_indicator }
    }
}

impl AttackDescriptionWidget {
    pub fn update(&mut self, attack: &Attack, identity : &IdentityData, is_active_attack : bool, is_active_counter : bool) -> &mut Self {
        self.active_indicator.set_showing(is_active_attack);
        self.counter_indicator.set_showing(is_active_counter);

        self.name.set_text(attack.name.capitalized());
        self.to_hit.set_text(format!("{} to hit", attack.to_hit_bonus.to_string_with_sign()));
        self.damage.set_text(format!("{} {} {} {}",
                                       attack.damage_dice,
                                       attack.damage_bonus.sign_str(),
                                       attack.damage_bonus.abs(),
                                       attack.primary_damage_type.to_string()));
        self
    }
    
    pub fn clear(&mut self)  -> &mut Self {
        self.name.set_text("Unknown, attack referenced was not present");
        self.to_hit.set_text(format!("N/A"));
        self.damage.set_text(format!("N/A"));
        self
    }
}


#[derive(WidgetContainer, DelegateToWidget)]
pub struct AttackDetailsWidget {
    pub to_hit_div: Widget,
    pub to_hit_details_div: Widget,
    pub damage_div: Widget,
    pub damage_details_div: Widget,

    pub name: Widget,

    pub to_hit: Widget,
    pub damage: Widget,
    pub divider: Widget,
    pub to_hit_details: Widget,
    pub to_miss_details: Widget,
    pub damage_dice_details: Widget,
    pub damage_bonus_details: Widget,
    pub damage_absorption_details: Widget,


    pub body: Widget,
}

impl AttackDetailsWidget {
    pub const PositiveColor: Color = Color::new(0.1, 0.5, 0.15, 1.0);
    pub const NegativeColor: Color = Color::new(0.5, 0.15, 0.1, 1.0);

    pub fn new() -> AttackDetailsWidget {
        let positive_color = AttackDetailsWidget::PositiveColor;
        let negative_color = AttackDetailsWidget::NegativeColor;
        let neutral_color = Color::black();

        let body = Widget::window(Color::new(0.8, 0.8, 0.9, 1.0), 2)
            .size(Sizing::surround_children(), Sizing::surround_children())
            .margin(2.px());

        let name = Widget::text("name", FontSize::HeadingMinor).parent(&body).named("ADT name");

        let to_hit_div = Widget::div().below(&name, 1.px()).named("ADT to hit div").parent(&body);
        let damage_div = Widget::div().below(&name, 1.px()).named("ADT damage div").right_of(&to_hit_div, 9.px()).parent(&body);

        let to_hit = Widget::text("to hit", FontSize::Standard).named("ADT to hit").parent(&to_hit_div);
        let divider = Widget::window(Color::greyscale(0.5), 1).size(Sizing::PcntOfParentAllowingLoop(1.0), Sizing::constant(3.px())).below(&to_hit, 3.px()).named("ADT divider").parent(&body);

        let to_hit_details_div = Widget::div().below(&to_hit, 10.px()).named("ADT to hit DT div").parent(&to_hit_div);

        let to_hit_details = Widget::text("to hit details", FontSize::Small).color(positive_color).named("ADT to hit details").parent(&to_hit_details_div);
        let to_miss_details = Widget::text("to miss details", FontSize::Small).below(&to_hit_details, 2.px()).color(negative_color).named("ADT to miss details").parent(&to_hit_details_div);


        let damage_details_div = Widget::div().match_y_of(&to_hit_details_div).named("ADT damage details div").parent(&damage_div);
        let damage_dice_details = Widget::text("damage dice details", FontSize::Small).color(neutral_color).named("ADT damage dice").parent(&damage_details_div);
        let damage_bonus_details = Widget::text("damage bonus details", FontSize::Small).below(&damage_dice_details, 1.px()).color(positive_color).named("ADT damage bonus").parent(&damage_details_div);
        let damage_absorption_details = Widget::text("damage absorption", FontSize::Small).below(&damage_bonus_details, 1.px()).color(negative_color).named("ADT damage absorb").parent(&damage_details_div);

        let damage = Widget::text("damage", FontSize::Standard).match_y_of(&to_hit).match_x_of(&damage_div).named("ADT damage").parent(&body);

        AttackDetailsWidget {
            body,
            name,
            to_hit,
            damage,
            divider,
            to_hit_details,
            to_miss_details,
            damage_dice_details,
            damage_bonus_details,
            damage_absorption_details,
            to_hit_div,
            damage_div,
            to_hit_details_div,
            damage_details_div,
        }
    }

    pub fn update(&mut self, gui: &mut GUI, world: &World, view: &WorldView, attacker: Entity, defender: Entity, attack_ref: &AttackRef, attacking_from : Option<AxialCoord>, ap_remaining: i32) {
//        self.to_hit_div.reapply(gui);
//        self.damage_div.reapply(gui);
//        self.to_hit_details_div.reapply(gui);
//        self.damage_details_div.reapply(gui);

        let attack_breakdown = combat::compute_attack_breakdown(world, view, attacker, defender, attack_ref, attacking_from, Some(ap_remaining));

        if let Some(strike) = attack_breakdown.strikes.first() {
            if let Some(strike_target) = strike.per_target_breakdowns.first() {
                self.for_each_widget(|w| { w.set_showing(true); });
                let components_to_str = |v: &Breakdown<i32>| {
                    v.components.iter().filter(|t| t.0 != "+0").map(|(bonus, description)| format!("{}  {}", bonus, description)).join("\n")
                };

                self.name.set_text(format!("{} x {}", strike.attack.name.capitalized(), attack_breakdown.strikes.len()));
                self.to_hit.set_text(format!("{} to hit", (strike_target.to_hit_total() - strike_target.to_miss_total()).to_string_with_sign()));
                let combined_dice_str = strike_target.damage_dice_total().map(|dd| dd.to_string()).join(" + ");
                let net_damage_mod = strike_target.damage_bonus_total() - strike_target.damage_absorption_total();
                let damage_type_str = strike.damage_types.iter().map(|dt| dt.to_string().to_lowercase()).join("/");
                self.damage.set_text(format!("{} {} {} {}", combined_dice_str, net_damage_mod.sign_str(), net_damage_mod.abs(), damage_type_str));
                self.to_hit_details.set_text(components_to_str(&strike_target.to_hit_components));
                self.to_miss_details.set_text(components_to_str(&strike_target.to_miss_components));
                self.damage_dice_details.set_text(strike_target.damage_dice_components.iter().map(|(dice, reason)| format!("{}  {}", dice, reason)).join("\n"));
                self.damage_bonus_details.set_text(components_to_str(&strike_target.damage_bonus_components));
                self.damage_absorption_details.set_text(components_to_str(&strike_target.damage_absorption_components));
            }
        } else {
            self.for_each_widget(|w| { w.set_showing(false); });
            self.body.set_showing(true);
            self.name.set_showing(true).set_text("Insufficient AP to attack");
        }
        self.reapply_all(gui);
    }

    pub fn hide(&mut self, gui: &mut GUI) {
        self.body.set_showing(false).reapply(gui);
    }
    pub fn show(&mut self, gui: &mut GUI) {
        self.body.set_showing(true).reapply(gui);
    }
}