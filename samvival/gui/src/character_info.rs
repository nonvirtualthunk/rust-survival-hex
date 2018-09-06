use std;
use common::Color;
use common::prelude::*;
use game::core::*;
use game::entities::Skill;
use game::WorldView;
use game::entities::*;
use gui::*;
use std::fmt;
use state::GameState;
use attack_descriptions::*;
use state::ControlContext;
use game::Entity;
use graphics::FontSize;


pub struct CharacterInfoWidget {
    character_stats: Vec<CharacterStat>,
    main_widget: Widget,
    name_widget: Widget,
    unit_icon_background: Widget,
    unit_icon: Widget,
    character_stats_widget: ListWidget<CharacterStatsWidget>,
    tabs: TabWidget,
    skills: ListWidget<SkillWidget>,
    attack_descriptions : AttackDescriptionsWidget
}

impl DelegateToWidget for CharacterInfoWidget {
    fn as_widget(&mut self) -> &mut Widget { &mut self.main_widget }
    fn as_widget_immut(&self) -> &Widget { &self.main_widget }
}


impl CharacterInfoWidget {
    pub fn new(gui : &mut GUI) -> CharacterInfoWidget {
        let main_widget = Widget::window(Color::new(0.6, 0.6, 0.6, 1.0), 2)
            .size(Sizing::Constant(40.0.ux()), Sizing::PcntOfParent(1.0))
            .position(Positioning::Constant(0.ux()), Positioning::Constant(0.ux()))
            .alignment(Alignment::Top, Alignment::Right)
            .margin(1.ux())
            .showing(false)
            .apply(gui);

        let name_widget = Widget::text("Name", FontSize::HeadingMajor)
            .x(Positioning::CenteredInParent)
            .parent(&main_widget)
            .apply(gui);

        let unit_icon_background = Widget::image("ui/blank", Color::white(), 1)
            .x(1.ux())
            .below(&name_widget, 0.2.ux())
            .surround_children()
            .parent(&main_widget)
            .apply(gui);

        let unit_icon = Widget::image("ui/blank", Color::white(), 0)
            .parent(&unit_icon_background)
            .apply(gui);

        let character_stats = vec![
            CharacterStat::new_reduceable("HP", |view,ent| view.character(ent).health.clone(), "Health. Characters fall unconscious if this reaches 0"),
            CharacterStat::new("AP", |view,ent| view.character(ent).action_points.cur_value(), |view,ent| view.character(ent).action_points.max_value(),
                               "Action Points, represents how much more this character can do this turn. Moving and using actions consume AP, AP resets every turn"),
            CharacterStat::cur_only("Move Speed", |view,ent| view.character(ent).movement.move_speed,
                                    "How fast this character moves per action point spent at a normal pace. Max movement at a \
                                    normal pace is therefore AP * Move Speed"),
            CharacterStat::new("Stamina", |view,ent| view.character(ent).stamina.cur_value().as_i32(), |view,ent| view.character(ent).stamina.max_value().as_i32(),
                               "How much endurance this character has left for performing strenuous actions. Attacking, reacting, running and the like all \
                               consume some amount of stamina. Normally recovers by 1 each turn."),
        ];

        let character_stats_widget = ListWidget::featherweight()
            .parent(&main_widget)
            .size(Sizing::match_parent(), Sizing::SurroundChildren)
            .position(Positioning::ux(1.0), Positioning::below(&unit_icon_background, 10.px()));

        let tabs = TabWidget::new(vec!["Attacks", "Skills"])
            .parent(&main_widget)
            .size(Sizing::ux(30.0), Sizing::ux(50.0))
            .position(Positioning::ux(1.0), Positioning::ux(30.0))
            .apply_all(gui);

        let mut skills = ListWidget::new()
            .parent(tabs.tab_named("Skills"))
            .size(Sizing::match_parent(), Sizing::match_parent())
            .position(Positioning::origin(), Positioning::origin())
            .apply(gui);
        skills.item_archetype.set_margin(4.px());

        let attack_descriptions = AttackDescriptionsWidget::new(gui, tabs.tab_named("Attacks"));

        CharacterInfoWidget {
            character_stats,
            character_stats_widget,
            main_widget,
            name_widget,
            unit_icon,
            unit_icon_background,
            skills,
            tabs,
            attack_descriptions
        }
    }

    pub fn update(&mut self, world_view: & WorldView, gui: &mut GUI, game_state: &GameState, control: &mut ControlContext) {
        if let Some(selected) = game_state.selected_character {
            let character = world_view.character(selected);
            let skills = world_view.skills(selected);

            self.main_widget.set_showing(true).reapply(gui);

            self.name_widget.modify_widget_type(|t| t.set_text(character.name.as_str())).reapply(gui);

            self.unit_icon.set_widget_type(WidgetType::image(format!("entities/{}", character.sprite))).reapply(gui);

            self.character_stats_widget.update(gui, self.character_stats.as_ref(), |stat_w, stat| {
                let numeral_display = match stat.max_value_func {
                    Some(ref mvf) => format!("{} / {}", (stat.cur_value_func)(world_view, selected), mvf(world_view, selected)),
                    _ => (stat.cur_value_func)(world_view, selected)
                };
                let text = format!("{}: {}", stat.name, numeral_display);
                stat_w.text.set_widget_type(WidgetType::text(text, FontSize::Standard))
                    .set_color(Color::black())
                    .set_height(Sizing::Derived)
                    .set_tooltip(stat.tooltip);
            });



            let mut character_skills: Vec<(Skill, i32)> = ::game::logic::skill::skill_levels(world_view, selected);
            character_skills.sort_by_key(|&(_, lvl)| -(lvl as i32));

            self.skills.update(gui, character_skills.as_ref(), |skill_w, skill| {
                let (skill, lvl): (Skill, i32) = *skill;
                let skill_info = skill_info(skill);
                let text = format!("{} : {}", skill_info.name, lvl);
                skill_w.text.modify_widget_type(|wt| wt.set_text(text.clone()));


                let xp_required_for_current_level = Skill::xp_required_for_level(lvl);
                let xp_required_for_next_level = Skill::xp_required_for_level(lvl + 1);
                let current_xp = skills.cur_skill_xp(skill);

                let required_delta = xp_required_for_next_level - xp_required_for_current_level;
                let actual_delta = current_xp - xp_required_for_current_level;
                let xp_pcnt = actual_delta as f64 / required_delta as f64;

                skill_w.xp_bar_empty.set_y(Positioning::DeltaOfWidget(skill_w.text.id(), 4.px(), Alignment::Bottom));
                skill_w.xp_bar_full.set_parent(&skill_w.xp_bar_empty).set_width(Sizing::PcntOfParent(xp_pcnt as f32));
            }).reapply_all(gui);

            self.attack_descriptions.update(gui, world_view, selected, control);
        }
    }
}


#[derive(WidgetContainer, Clone)]
pub struct SkillWidget {
    text: Widget,
    xp_bar_empty: Widget,
    xp_bar_full: Widget,
}

impl Default for SkillWidget {
    fn default() -> Self {
        SkillWidget {
            text: Widget::text("", FontSize::Standard),
            xp_bar_empty: Widget::segmented_image("ui/pill", Color::white(), ImageSegmentation::Horizontal)
                .size(Sizing::DeltaOfParent(-10.px()), Sizing::Constant(13.px()))
                .x(Positioning::Constant(5.px())),
            xp_bar_full: Widget::segmented_image("ui/pill", Color::new(0.1, 0.8, 0.2, 1.0), ImageSegmentation::Horizontal)
                .position(Positioning::Constant(0.ux()), Positioning::Constant(0.ux()))
                .height(Sizing::Constant(13.px())),
        }
    }
}

pub struct CharacterStat {
    name: Str,
    cur_value_func: Box<Fn(&WorldView, Entity) -> String>,
    max_value_func: Option<Box<Fn(&WorldView, Entity) -> String>>,
    tooltip: Str
}

impl CharacterStat {
    fn new<T: std::string::ToString, F: Fn(&WorldView, Entity) -> T + 'static, MF: Fn(&WorldView, Entity) -> T + 'static>(name: Str, f: F, mf: MF, tooltip : Str) -> CharacterStat {
        CharacterStat {
            name,
            cur_value_func: box move |view,ent| f(view,ent).to_string(),
            max_value_func: Some(box move |view,ent| mf(view,ent).to_string()),
            tooltip
        }
    }

    fn cur_only<T : std::string::ToString + 'static>(name: Str, f: fn(&WorldView, Entity) -> T, tooltip : Str) -> CharacterStat {
        CharacterStat {
            name,
            cur_value_func: box move |view,ent| f(view,ent).to_string(),
            max_value_func: None,
            tooltip
        }
    }

    fn new_reduceable<T: ReduceableType + fmt::Display + 'static>(name: Str, f: fn(&WorldView, Entity) -> Reduceable<T>, tooltip : Str) -> CharacterStat {
        CharacterStat {
            cur_value_func: box move |view,ent| format!("{}", f(view,ent).cur_value()),
            max_value_func: Some(box move |view,ent| format!("{}", f(view,ent).max_value())),
            name,
            tooltip
        }
    }
}

impl Default for CharacterStat {
    fn default() -> Self {
        CharacterStat {
            name: "Uninitialized",
            cur_value_func: box |view,ent| { String::from("uninitialized") },
            max_value_func: None,
            tooltip: "uninitialized"
        }
    }
}


#[derive(Default, WidgetContainer)]
pub struct CharacterStatsWidget {
    text: Widget
}