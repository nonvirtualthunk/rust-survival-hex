use gui::*;
use game::logic::combat::*;
use common::prelude::*;
use game::prelude::*;
use common::color::Color;
use game::logic::combat;
use game::entities::combat::Attack;


pub struct AttackDescriptionsWidget {
    attack_list: ListWidget<AttackDescriptionWidget>
}

impl WidgetContainer for AttackDescriptionsWidget {
    fn for_all_widgets<F: FnMut(&mut Widget)>(&mut self, func: F) {
        self.attack_list.for_all_widgets(func);
    }
}

impl AttackDescriptionsWidget {
    pub fn new(gui: &mut GUI, parent: &Widget) -> AttackDescriptionsWidget {
        let mut attack_list = ListWidget::new()
            .size(Sizing::match_parent(), Sizing::surround_children())
            .parent(parent)
            .apply(gui);

        attack_list.item_archetype.set_margin(1.ux());
        AttackDescriptionsWidget { attack_list }
    }


    pub fn update(&mut self, gui: &mut GUI, view: &WorldView, character: Entity) {
        let attacks = possible_attacks(view, character);
        let active_attack = primary_attack(view, character);

        self.attack_list.update(gui, attacks.as_ref(), |widget, attack| {
            widget.name.set_text(attack.name.capitalized());
            widget.to_hit.set_text(format!("{} to hit", attack.to_hit_bonus.to_string_with_sign()));
            widget.damage.set_text(format!("{} {} {} {}",
                                           attack.damage_dice,
                                           attack.damage_bonus.sign_str(),
                                           attack.damage_bonus.abs(),
                                           attack.primary_damage_type.to_string()));
        });
    }
}

#[derive(WidgetContainer, Clone)]
struct AttackDescriptionWidget {
    pub name: Widget,
    pub to_hit: Widget,
    pub damage: Widget,
}

impl Default for AttackDescriptionWidget {
    fn default() -> Self {
        let name = Widget::text("Name here", 14)
            .with_tooltip("The name of the attack");
        let to_hit = Widget::text("To hit here", 14)
            .y(Positioning::below(&name, 2.px()))
            .with_tooltip("Bonus applied to every roll to determine if the attack hits");
        let damage = Widget::text("Damage here", 14)
            .y(Positioning::below(&to_hit, 2.px()))
            .with_tooltip("The amount and kind of damage done on a successful hit. \
            May be modified by the defender's armor, resistances and special abilities");

        AttackDescriptionWidget { name, to_hit, damage }
    }
}


#[derive(WidgetContainer, DelegateToWidget)]
pub struct AttackDetailsWidget {
    pub body: Widget,
    pub name: Widget,
    pub to_hit: Widget,
    pub damage: Widget,
    pub divider: Widget,
    pub to_hit_details: Widget,
    pub to_miss_details: Widget,
    pub damage_dice_details: Widget,
    pub damage_bonus_details: Widget,
    pub damage_absorption_details: Widget
}

impl AttackDetailsWidget {
    pub const PositiveColor : Color = Color::new(0.1,0.5,0.15, 1.0);
    pub const NegativeColor : Color = Color::new(0.5,0.15,0.1, 1.0);

    pub fn new() -> AttackDetailsWidget {
        let positive_color = AttackDetailsWidget::PositiveColor;
        let negative_color = AttackDetailsWidget::NegativeColor;

        let body = Widget::window(Color::new(0.8, 0.8, 0.9, 1.0), 2)
            .size(Sizing::surround_children(), Sizing::surround_children())
            .margin(2.px());

        let name = Widget::text("name", 14).parent(&body);
        let to_hit = Widget::text("to hit", 14).below(&name, 1.px()).parent(&body);
        let damage = Widget::text("damage", 14).right_of(&to_hit, 10.px()).y(Positioning::match_to(&to_hit)).parent(&body);
        let divider = Widget::window(Color::greyscale(0.5), 1).size(Sizing::match_parent(), Sizing::constant(3.px())).below(&to_hit, 3.px()).parent(&body);
        let to_hit_details = Widget::text("to hit details", 12).below(&divider, 3.px()).color(positive_color).parent(&body);
        let to_miss_details = Widget::text("to miss details", 12).below(&to_hit_details, 2.px()).color(negative_color).parent(&body);
        let damage_dice_details = Widget::text("damage dice details", 12).below(&to_miss_details, 2.px()).color(positive_color).parent(&body);
        let damage_bonus_details = Widget::text("damage bonus details", 12).below(&damage_dice_details, 1.px()).color(positive_color).parent(&body);
        let damage_absorption_details = Widget::text("damage absorption", 12).below(&damage_bonus_details, 1.px()).color(negative_color).parent(&body);

        AttackDetailsWidget { body, name, to_hit, damage, divider, to_hit_details, to_miss_details, damage_dice_details, damage_bonus_details, damage_absorption_details }
    }

    pub fn update(&mut self, gui : &mut GUI, world : &WorldView, attacker : Entity, defender : Entity, attack : &Attack) {

        let attack_breakdown = combat::compute_attack_breakdown(world, attacker, defender, attack);

        if let Some(strike) = attack_breakdown.strikes.first() {
            self.for_all_widgets(|w| { w.set_showing(true); });
            let components_to_str = |v : &Vec<(i32,Str)>| v.iter().filter(|t| t.0 != 0).map(|(bonus, reason)| format!("{}  {}", bonus.to_string_with_sign(), reason)).join("\n");
            self.name.set_text(format!("{} x {}", attack.name, attack_breakdown.strikes.len()));
            self.to_hit.set_text(format!("{}", (strike.to_hit_total() - strike.to_miss_total()).to_string_with_sign()));
            let combined_dice_str = strike.damage_dice_total().map(|dd| dd.to_string()).join(" + ");
            let net_damage_mod = strike.damage_bonus_total() - strike.damage_absorption_total();
            self.damage.set_text(format!("{} {} {}", combined_dice_str, net_damage_mod.sign_str(), net_damage_mod.abs()));
            self.to_hit_details.set_text(components_to_str(&strike.to_hit_components));
            self.to_miss_details.set_text(components_to_str(&strike.to_miss_components));
            self.damage_dice_details.set_text(strike.damage_dice_components.iter().map(|(dice, reason)| format!("{}  {}", dice, reason)).join("\n"));
            self.damage_bonus_details.set_text(components_to_str(&strike.damage_bonus_components));
            self.damage_absorption_details.set_text(components_to_str(&strike.damage_absorption_components));
        } else {
            self.for_all_widgets(|w| { w.set_showing(false); });
            self.body.set_showing(true);
            self.name.set_showing(true).set_text("Insufficient AP to attack");
        }
        self.reapply(gui);
    }

    pub fn hide(&mut self, gui : &mut GUI) {
        self.body.set_showing(false).reapply(gui);
    }
    pub fn show(&mut self, gui : &mut GUI) {
        self.body.set_showing(true).reapply(gui);
    }
}