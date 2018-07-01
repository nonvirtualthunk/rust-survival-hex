use tactical::TacticalMode;
use conrod::*;
use game::World;
use game::world::Entity;
use conrod::widget::Id as Wid;
use conrod::widget::id::List as Wids;
use std::sync::Mutex;
use game::entities::*;
use game::core::Reduceable;
use game::core::GameEventClock;
use std::ops;
use std::fmt;
use std;
use game::actions::skills;


lazy_static! {
    static ref WIDGETS : Mutex<Widgets> = {
        Mutex::new(Widgets::default())
    };
}


#[derive(Default)]
pub struct LazyWid {
    raw_id: Option<Wid>
}

impl LazyWid {
    pub fn id(&mut self, ui: &mut UiCell) -> Wid {
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
impl<T> ConvenienceSettable<T::Event> for T where T: Widget {
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

pub struct TacticalGui {
    character_stats: Vec<CharacterStat>
}

impl TacticalGui {
    pub fn new() -> TacticalGui {
        TacticalGui {
            character_stats: vec![
                CharacterStat::new("HP", |cdata| cdata.health.cur_value(), |cdata| cdata.health.max_value()),
                CharacterStat::new("Moves", |cdata| cdata.moves.cur_value(), |cdata| cdata.moves.max_value()),
                CharacterStat::new("Stamina", |cdata| cdata.stamina.cur_value(), |cdata| cdata.stamina.max_value()),
                //                CharacterStat::new_reduceable(|cdata| cdata.health)
            ]
        }
    }

    pub fn draw_gui(&mut self, world: &mut World, ui: &mut UiCell, frame_id: Wid, game_state: GameState) {
        //        let tmp = CharacterStat {
        //            cur_value_func : box |cdata : &CharacterData|  box cdata.health.cur_value(),
        //            max_value_func : Some(box |cdata : &CharacterData|  box cdata.health.max_value())
        //        };

        let world_view = world.view_at_time(game_state.display_event_clock);

        let mut widgets = WIDGETS.lock().unwrap();
        widgets.init(&mut ui.widget_id_generator());


        if let Some(character_ref) = game_state.selected_character {
            let character = world_view.character(character_ref);
            let main_widget = widget::Canvas::new()
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

            TacticalGui::skills_widget(&character, &mut widgets, ui);

            widget::RoundedRectangle::fill([72.0, 72.0], 5.0)
                .color(color::BLUE.alpha(1.0))
                .x_place_on(widgets.main_widget, position::Place::Start(Some(20.0)))
//                .y(20.0)
                .down_from(widgets.name_widget.id(ui), 10.0)
                .parent(widgets.main_widget)
                .set(widgets.unit_icon, ui);
        }

        if let Some(victorious) = game_state.victory {
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

    pub fn skills_widget(character : &CharacterData, widgets : &mut Widgets, ui : &mut UiCell) {
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
                .color(Color::Rgba(0.0f32,0.0f32,0.0f32,0.0f32))
//                .padded_w_of(widgets.skills.list, 20.0)
                .h(40.0);

            item.set(canvas, ui);

            widget::Text::new(text.as_str())
                .font_size(14)
                .top_left_with_margins_on(item.widget_id, 5.0, 5.0)
                .set(widgets.skills.text[item.i], ui);

            let xp_bar_width = 100.0;

            widget::RoundedRectangle::fill([xp_bar_width,10.0], 4.0)
                .color(Color::Rgba(1.0f32,1.0f32,1.0f32,1.0f32))
                .down_from(widgets.skills.text[item.i], 5.0)
                .set(widgets.skills.xp_bar_base[item.i], ui);

            let xp_required_for_current_level = skills::xp_required_for_level(*lvl);
            let xp_required_for_next_level = skills::xp_required_for_level(lvl + 1);
            let current_xp = character.skill_xp(skill);

            let required_delta = xp_required_for_next_level - xp_required_for_current_level;
            let actual_delta = current_xp - xp_required_for_current_level;
            let xp_pcnt = actual_delta as f64 / required_delta as f64;
            widget::RoundedRectangle::fill([xp_bar_width * xp_pcnt,10.0], 4.0)
                .color(Color::Rgba(0.15f32,0.7f32,0.05f32,1.0f32))
                .down_from(widgets.skills.text[item.i], 5.0)
                .set(widgets.skills.xp_bar[item.i], ui);

//                .parent(widgets.skills.list)
//                .set(item.widget_id, ui);
        }
    }
}

#[derive(Default)]
pub struct Widgets {
    initialized: bool,
    main_widget: Wid,
    name_widget: LazyWid,
    unit_icon: Wid,
    moves_widget: Wid,
    health_widget: Wid,
    victory_widget: Wid,
    victory_text: Wid,
    stat_list_widget: Wid,
    skills : Skills,
    attacks : Attacks,
    tabs: Wid
}

pub struct Skills {
    tab: Wid,
    list : Wid,
    text : Wids,
    xp_bar_base : Wids,
    xp_bar : Wids
}

pub struct Attacks {
    tab : Wid,
    list : Wid
}

impl Default for Skills {
    fn default() -> Self {
        Skills {
            tab: Wid::default(),
            list : Wid::default(),
            text : Wids::new(),
            xp_bar_base : Wids::new(),
            xp_bar : Wids::new()
        }
    }
}

impl Default for Attacks {
    fn default() -> Self {
        Attacks {
            tab : Wid::default(),
            list : Wid::default()
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
}


//pub struct ListWidget <F : Fn(int) -> Widget> {
//    ids : Wids,
//
//}