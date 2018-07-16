use tactical::TacticalMode;
use conrod::Positionable;
use conrod::Sizeable;
use conrod::Colorable;
use conrod::Borderable;
use conrod::widget;
use game::World;
use game::world::Entity;
use game::WorldView;
use conrod;
use std::sync::Mutex;
use game::entities::*;
use game::core::Reduceable;
use game::core::GameEventClock;
use std::ops;
use std::fmt;
use std;
use game::actions::skills;
use gui::*;
use gui::ToGUIUnit;
use common::Color;

use game::actions;
use common::prelude::*;
use arx_graphics::core::GraphicsWrapper;
use conrod::UiCell;


lazy_static! {
    static ref WIDGETS : Mutex<Widgets> = {
        Mutex::new(Widgets::default())
    };
}


#[derive(Default)]
pub struct LazyWid {
    raw_id: Option<conrod::widget::Id>
}

impl LazyWid {
    pub fn id(&mut self, ui: &mut UiCell) -> conrod::widget::Id {
        if self.raw_id.is_some() {
            self.raw_id.unwrap()
        } else {
            let id = ui.widget_id_generator().next();
            self.raw_id = Some(id);
            id
        }
    }
}

trait ConvenienceSettable<R> {
    fn setl(self, wid: &mut LazyWid, ui: &mut UiCell) -> R;
}
//impl <Style, State, Event> ConvenienceSettable for Widget<Style = Style, State = State, Event = Event> {
impl<T> ConvenienceSettable<T::Event> for T where T: conrod::Widget {
    fn setl(self, wid: &mut LazyWid, ui: &mut UiCell) -> T::Event {
        if wid.raw_id.is_some() {
            self.set(wid.raw_id.unwrap(), ui)
        } else {
            let id = ui.widget_id_generator().next();
            wid.raw_id = Some(id);
            self.set(id, ui)
        }
    }
}

//pub struct CharacterStat <CF : Fn(&CharacterData) -> String, MF : Fn(&CharacterData) -> Option<String>> {
//    cur_value_func : CF,
//    max_value_func : MF
//}

//trait Stringable = std::string::ToString;

pub struct CharacterStat {
    name: &'static str,
    cur_value_func: Box<Fn(&CharacterData) -> String>,
    max_value_func: Option<Box<Fn(&CharacterData) -> String>>
}

impl CharacterStat {
    //    fn new_cur <F : Fn(&CharacterData) -> Box<std::string::ToString> + 'static>(f : F) -> CharacterStat {
    //        CharacterStat {
    //            cur_value_func : box f,
    //            max_value_func : None
    //        }
    //    }

    fn new<T: std::string::ToString, F: Fn(&CharacterData) -> T + 'static, MF: Fn(&CharacterData) -> T + 'static>(name: &'static str, f: F, mf: MF) -> CharacterStat {
        CharacterStat {
            name,
            cur_value_func: box move |raw| f(raw).to_string(),
            max_value_func: Some(box move |raw| mf(raw).to_string())
        }
    }

    //    fn new_reduceable <T: ops::Sub<Output=T> + ops::Add<Output=T> + Copy + Default + Into<f64> + fmt::Display, F : Fn(&CharacterData) -> Reduceable<T> + 'static> (f : F) -> CharacterStat {
    //        CharacterStat {
    //            cur_value_func : box |cdata| format!("{}", f(cdata).cur_value()),
    //            max_value_func : None
    //        }
    //    }
}

pub struct GameState {
    pub display_event_clock: GameEventClock,
    pub selected_character: Option<Entity>,
    pub victory: Option<bool>
}

#[derive(Default)]
pub struct TacticalGui {
    character_stats: Vec<CharacterStat>,
    gui : GUI,
    test_window : Widget,
    sub_window : Widget,
    sub_text : Widget,
    right_text : Widget,
    left_sub_window : Widget,
    button : Button
}

impl TacticalGui {
    pub fn new() -> TacticalGui {
        let mut gui = GUI::new();

        let test_window = Widget::new(WidgetType::window(Color::new(0.8,0.5,0.2,1.0), 1))
            .size(Sizing::DeltaOfParent(-50.0.ux()), Sizing::PcntOfParent(0.5))
            .position(Positioning::CenteredInParent, Positioning::CenteredInParent)
            .apply(&mut gui);

        let sub_window = Widget::new(WidgetType::image("entities/default/defaultium", Color::white(), 0 ))
            .size(Sizing::Constant(40.0.ux()), Sizing::PcntOfParent(0.75))
            .position(Positioning::Constant(2.0.ux()), Positioning::CenteredInParent)
            .alignment(Alignment::Right, Alignment::Top)
            .parent(&test_window)
            .apply(&mut gui);

        let sub_text = Widget::new(WidgetType::Text {text : String::from("Hello, world\nNewline"), font_size : 16, font : None, color : Color::black(), wrap : false })
            .size(Sizing::Derived, Sizing::Derived)
            .position(Positioning::Constant(1.0.ux()), Positioning::Constant(1.0.ux()))
            .parent(&sub_window)
            .apply(&mut gui);

        let right_text = Widget::new(WidgetType::Text { text : String::from("| Right |"), font_size : 16, font : None, color : Color::black(), wrap : false})
            .size(Sizing::Derived, Sizing::Derived)
            .position(Positioning::DeltaOfWidget(sub_text.id, 1.px(), Alignment::Right), Positioning::DeltaOfWidget(sub_text.id, 0.0.px(), Alignment::Top))
            .parent(&sub_window)
            .apply(&mut gui);

        let left_sub_window = Widget::new(WidgetType::window(Color::white(), 1))
            .size(Sizing::Constant(20.0.ux()), Sizing::Constant(20.0.ux()))
            .position(Positioning::DeltaOfWidget(sub_window.id, 1.px(), Alignment::Left), Positioning::DeltaOfWidget(sub_window.id, 0.px(), Alignment::Top))
            .alignment(Alignment::Right, Alignment::Top)
            .parent(&test_window)
            .apply(&mut gui);

        let button = Button::new("Test Button")
            .position(Positioning::DeltaOfWidget(left_sub_window.id, 0.px(), Alignment::Left),
                      Positioning::DeltaOfWidget(left_sub_window.id, 2.px(), Alignment::Bottom))
            .parent(&test_window)
            .apply(&mut gui);

        TacticalGui {
            character_stats: vec![
                CharacterStat::new("HP", |cdata| cdata.health.cur_value(), |cdata| cdata.health.max_value()),
                CharacterStat::new("AP", |cdata| cdata.action_points.cur_value(), |cdata| cdata.action_points.max_value()),
                CharacterStat::new("Moves", |cdata| cdata.max_moves_remaining().as_i32(), |cdata| cdata.max_moves_per_turn().as_i32()),
                CharacterStat::new("Stamina", |cdata| cdata.stamina.cur_value(), |cdata| cdata.stamina.max_value()),
                //                CharacterStat::new_reduceable(|cdata| cdata.health)
            ],
            test_window,
            sub_window,
            sub_text,
            right_text,
            left_sub_window,
            button,
            gui,
        }
    }

    pub fn draw(&mut self, world: &mut World, g : &mut GraphicsWrapper) {
        self.gui.draw(g);
    }

    pub fn draw_gui(&mut self, world: &mut World, ui: &mut UiCell, frame_id: conrod::widget::Id, game_state: GameState) {
        //        let tmp = CharacterStat {
        //            cur_value_func : box |cdata : &CharacterData|  box cdata.health.cur_value(),
        //            max_value_func : Some(box |cdata : &CharacterData|  box cdata.health.max_value())
        //        };

        let world_view = world.view_at_time(game_state.display_event_clock);

        let mut widgets = WIDGETS.lock().unwrap();
        widgets.init(&mut ui.widget_id_generator());


        if let Some(character_ref) = game_state.selected_character {
            let character = world_view.character(character_ref);
            use conrod::Widget;

            let main_widget = conrod::widget::Canvas::new()
                .pad(10.0)
                .scroll_kids_vertically()
                .top_right()
                .w(400.0)
                .h(800.0);

            main_widget.set(widgets.main_widget, ui);

            widget::Text::new(character.name.as_str())
                .font_size(20)
                .mid_top_of(widgets.main_widget)
                .parent(widgets.main_widget)
                .setl(&mut widgets.name_widget, ui);

            let mut stats = widget::List::flow_down(self.character_stats.len())
//                .x(20.0)
//                .align_left_of(widgets.main_widget)
//                .x_place_on(widgets.main_widget, position::Place::Start(Some(20.0)))
//                .down_from(widgets.unit_icon, 0.0)
                .parent(widgets.main_widget)
                .set(widgets.stat_list_widget, ui).0;

            while let Some(item) = stats.next(ui) {
                let stat = &self.character_stats[item.i];
                let numeral_display = match stat.max_value_func {
                    Some(ref mvf) => format!("{} / {}", (stat.cur_value_func)(&character), mvf(&character)),
                    _ => (stat.cur_value_func)(&character)
                };
                let text = format!("{}: {}", stat.name, numeral_display);
                widget::Text::new(text.as_str())
                    .font_size(16)
                    .set(item.widget_id, ui)
            }

            widget::Tabs::new(&[(widgets.skills.tab, "Skills"), (widgets.attacks.tab, "Attacks")])
                .h(500.0)
                .padded_w_of(widgets.main_widget, 20.0)
                .middle_of(widgets.main_widget)
                .parent(widgets.main_widget)
                .set(widgets.tabs, ui);

            TacticalGui::skills_widget(world_view.skills(character_ref), &mut widgets, ui);
            TacticalGui::attacks_widget(&world_view, character_ref, &mut widgets, ui);

            widget::RoundedRectangle::fill([72.0, 72.0], 5.0)
                .color(conrod::color::BLUE.alpha(1.0))
                .x_place_on(widgets.main_widget, conrod::position::Place::Start(Some(20.0)))
//                .y(20.0)
                .down_from(widgets.name_widget.id(ui), 10.0)
                .parent(widgets.main_widget)
                .set(widgets.unit_icon, ui);
        }

        if let Some(victorious) = game_state.victory {
            use conrod::Widget;
            widget::Canvas::new()
                .pad(10.0)
                .parent(frame_id)
                .middle_of(frame_id)
                .w(600.0)
                .h(200.0)
                .set(widgets.victory_widget, ui);

            let text = if victorious {
                "Victory!"
            } else {
                "Defeat!"
            };
            widget::Text::new(text)
                .font_size(32)
                .middle_of(widgets.victory_widget)
                .set(widgets.victory_text, ui);
        }
    }

    pub fn skills_widget(character : &SkillData, widgets : &mut Widgets, ui : &mut UiCell) {
        use conrod::Widget;

        let mut character_skills : Vec<(Skill,&u32)> = character.skills.iter().collect();
        let num_skills = character_skills.len();
        widgets.resize_skills(&mut ui.widget_id_generator(), num_skills);

        character_skills.sort_by_key(|&(_,lvl)| lvl);

//        widget::Canvas::new()
//            .font_size(14)
//            .middle_of(widgets.skills.tab)
//            .set(widgets.skills.list, ui);

        let mut skills_list : widget::list::Items<widget::list::Down, widget::list::Dynamic> = widget::List::flow_down(num_skills)
//            .item_size(20.0)
//            .h(200.0)
//            .x_place_on(widgets.skills.tab, position::Place::Start(Some(20.0)))
//            .y_place_on(widgets.skills.tab, position::Place::Start(Some(20.0)))
            .top_left_with_margins_on(widgets.skills.tab, 10.0, 10.0)
            .set(widgets.skills.list, ui).0;

        while let Some(item) = skills_list.next(ui) {
            let (skill,lvl) : (Skill, &u32) = character_skills[item.i];
            let skill_info = skill_info(skill);
            let text = format!("{} : {}", skill_info.name, lvl);

            let canvas = widget::Canvas::new()
                .border(0.0)
                .color(conrod::Color::Rgba(0.0f32,0.0f32,0.0f32,0.0f32))
//                .padded_w_of(widgets.skills.list, 20.0)
                .h(40.0);

            item.set(canvas, ui);

            widget::Text::new(text.as_str())
                .font_size(14)
                .top_left_with_margins_on(item.widget_id, 5.0, 5.0)
                .set(widgets.skills.text[item.i], ui);

            let xp_bar_width = 100.0;

            widget::RoundedRectangle::fill([xp_bar_width,10.0], 4.0)
                .color(conrod::Color::Rgba(1.0f32,1.0f32,1.0f32,1.0f32))
                .down_from(widgets.skills.text[item.i], 5.0)
                .set(widgets.skills.xp_bar_base[item.i], ui);

            let xp_required_for_current_level = skills::xp_required_for_level(*lvl);
            let xp_required_for_next_level = skills::xp_required_for_level(lvl + 1);
            let current_xp = character.skill_xp(skill);

            let required_delta = xp_required_for_next_level - xp_required_for_current_level;
            let actual_delta = current_xp - xp_required_for_current_level;
            let xp_pcnt = actual_delta as f64 / required_delta as f64;
            widget::RoundedRectangle::fill([xp_bar_width * xp_pcnt,10.0], 4.0)
                .color(conrod::Color::Rgba(0.15f32,0.7f32,0.05f32,1.0f32))
                .down_from(widgets.skills.text[item.i], 5.0)
                .set(widgets.skills.xp_bar[item.i], ui);

//                .parent(widgets.skills.list)
//                .set(item.widget_id, ui);
        }
    }

    pub fn attacks_widget(world : &WorldView, character : Entity, widgets : &mut Widgets, ui : &mut UiCell) {
        use conrod::Widget;
        let attacks = actions::combat::possible_attacks(world, character);
        let num_attacks = attacks.len();
        widgets.resize_attacks(&mut ui.widget_id_generator(), num_attacks);

        let mut attacks_list : widget::list::Items<widget::list::Down, widget::list::Dynamic> = widget::List::flow_down(num_attacks)
//            .item_size(20.0)
//            .h(200.0)
//            .x_place_on(widgets.skills.tab, position::Place::Start(Some(20.0)))
//            .y_place_on(widgets.skills.tab, position::Place::Start(Some(20.0)))
            .top_left_with_margins_on(widgets.attacks.tab, 10.0, 10.0)
            .set(widgets.attacks.list, ui).0;

        while let Some(item) = attacks_list.next(ui) {
            let attack = &attacks[item.i];
            let text = format!("{} AP : {} + {}, {}% Acc", attack.ap_cost, attack.damage_dice.to_d20_string(), attack.damage_bonus, attack.relative_accuracy.to_string_with_sign());

            let canvas = widget::Canvas::new()
                .border(0.0)
                .color(conrod::Color::Rgba(0.0f32,0.0f32,0.0f32,0.0f32))
//                .padded_w_of(widgets.skills.list, 20.0)
                .h(40.0);

            item.set(canvas, ui);

            widget::Text::new(text.as_str())
                .font_size(14)
                .top_left_with_margins_on(item.widget_id, 5.0, 5.0)
                .set(widgets.attacks.text[item.i], ui);
        }
    }
}

#[derive(Default)]
pub struct Widgets {
    initialized: bool,
    main_widget: conrod::widget::Id,
    name_widget: LazyWid,
    unit_icon: conrod::widget::Id,
    moves_widget: conrod::widget::Id,
    health_widget: conrod::widget::Id,
    victory_widget: conrod::widget::Id,
    victory_text: conrod::widget::Id,
    stat_list_widget: conrod::widget::Id,
    skills : Skills,
    attacks : Attacks,
    tabs: conrod::widget::Id
}

pub struct Skills {
    tab: conrod::widget::Id,
    list : conrod::widget::Id,
    text : conrod::widget::id::List,
    xp_bar_base : conrod::widget::id::List,
    xp_bar : conrod::widget::id::List
}

pub struct Attacks {
    tab : conrod::widget::Id,
    list : conrod::widget::Id,
    text : conrod::widget::id::List,
}

impl Default for Skills {
    fn default() -> Self {
        Skills {
            tab: conrod::widget::Id::default(),
            list : conrod::widget::Id::default(),
            text : conrod::widget::id::List::new(),
            xp_bar_base : conrod::widget::id::List::new(),
            xp_bar : conrod::widget::id::List::new()
        }
    }
}

impl Default for Attacks {
    fn default() -> Self {
        Attacks {
            tab : conrod::widget::Id::default(),
            list : conrod::widget::Id::default(),
            text : conrod::widget::id::List::new()
        }
    }
}

impl Widgets {
    pub fn init(&mut self, gen: &mut widget::id::Generator) {
        if !self.initialized {
            self.main_widget = gen.next();
            //            self.name_widget = gen.next();
            self.unit_icon = gen.next();
            self.moves_widget = gen.next();
            self.health_widget = gen.next();
            self.victory_widget = gen.next();
            self.victory_text = gen.next();
            self.stat_list_widget = gen.next();
            self.tabs = gen.next();

            self.skills.tab = gen.next();
            self.skills.list = gen.next();

            self.attacks.tab = gen.next();
            self.attacks.list = gen.next();

            self.initialized = true;
        }
    }

    pub fn resize_skills(&mut self, gen: &mut widget::id::Generator, n : usize) {
        let skills = &mut self.skills;
        for skills_wids in vec![&mut skills.text, &mut skills.xp_bar_base, &mut skills.xp_bar] {
            skills_wids.resize(n, gen);
        }
    }

    pub fn resize_attacks(&mut self, gen : &mut widget::id::Generator, n : usize) {
        let attacks = &mut self.attacks;
        for attack_wids in vec![&mut attacks.text] {
            attack_wids.resize(n, gen);
        }
    }
}


//pub struct ListWidget <F : Fn(int) -> Widget> {
//    ids : conrod::widget::id::List,
//
//}