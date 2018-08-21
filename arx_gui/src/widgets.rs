use anymap;
use anymap::any::CloneAny;
use anymap::AnyMap;
use backtrace::Backtrace;
use common::Color;
use common::prelude::*;
pub use compound_widgets::*;
use events::EventPosition;
use events::ui_event_types;
use events::UIEvent;
use events::UIEventType;
use events::WidgetEvent;
use graphics::DrawList;
use graphics::FontIdentifier;
use graphics::ImageIdentifier;
use gui::*;
use piston_window::keyboard;
use piston_window::MouseButton;
use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};
use std::sync::Mutex;
use ui_event_types::*;
use compound_widgets::TextDisplayWidget;
use widget_delegation::DelegateToWidget;

pub static WIDGET_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;



pub type WidIntern = usize;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug, Display)]
pub struct Wid(WidIntern);

pub fn create_wid() -> Wid {
    let id = WIDGET_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    Wid(id)
}

pub static NO_WID: Wid = Wid(0);


thread_local! {
//    static ref WIDGET_CALLBACKS : Mutex<WidgetCallbackRegistry> = Mutex::new(WidgetCallbackRegistry::new());
    pub static WIDGET_CALLBACKS : RefCell<WidgetCallbackRegistry> = RefCell::new(WidgetCallbackRegistry::new());
}

pub fn execute_global_widget_registry_callback<State: 'static>(id: u32, state: &mut State, evt: &UIEvent) {
    WIDGET_CALLBACKS.with(|registry| registry.borrow().execute_callback(id, state, evt))
}

pub fn execute_global_widget_registry_callback_2<State: 'static, OtherState: 'static>(id: u32, state: &mut State, other: &mut OtherState, evt: &UIEvent) {
    WIDGET_CALLBACKS.with(|registry| registry.borrow().execute_callback_2(id, state, other, evt))
}


#[derive(Clone, PartialEq)]
pub struct Border {
    pub width: u8,
    pub color: Color,
    pub sides: BorderSides,
}

impl Default for Border {
    fn default() -> Self {
        Border { width: 0, color: Color::black(), sides: BorderSides::all() }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct BorderSides(u8);

impl BorderSides {
    pub fn all() -> BorderSides { BorderSides(0xff) }

    pub fn three_sides(a: Alignment, b: Alignment, c: Alignment) -> BorderSides {
        BorderSides(BorderSides::alignment_to_flag(a) | BorderSides::alignment_to_flag(b) | BorderSides::alignment_to_flag(c))
    }
    pub fn two_sides(a: Alignment, b: Alignment) -> BorderSides {
        BorderSides(BorderSides::alignment_to_flag(a) | BorderSides::alignment_to_flag(b))
    }
    pub fn one_side(a: Alignment) -> BorderSides {
        BorderSides(BorderSides::alignment_to_flag(a))
    }
    pub fn none() -> BorderSides {
        BorderSides(0)
    }

    pub fn without_side(&self, side : Alignment) -> BorderSides {
        BorderSides(self.0 & (!(BorderSides::alignment_to_flag(side))))
    }
    pub fn with_side(&self, side : Alignment) -> BorderSides {
        BorderSides(self.0 | (BorderSides::alignment_to_flag(side)))
    }

    pub fn has_side(&self, a: Alignment) -> bool {
        (self.0 & BorderSides::alignment_to_flag(a)) != 0
    }
    pub fn has_near_side_for_axis(&self, axis: usize) -> bool {
        match axis {
            0 => self.has_side(Alignment::Left),
            1 => self.has_side(Alignment::Top),
            _ => {
                error!("Tried to get the near side for a non two dimensional axis");
                false
            }
        }
    }
    pub fn has_far_side_for_axis(&self, axis: usize) -> bool {
        match axis {
            0 => self.has_side(Alignment::Right),
            1 => self.has_side(Alignment::Bottom),
            _ => {
                error!("Tried to get the near side for a non two dimensional axis");
                false
            }
        }
    }

    fn alignment_to_flag(a: Alignment) -> u8 {
        match a {
            Alignment::Left => 0b00000001,
            Alignment::Right => 0b00000010,
            Alignment::Top => 0b00000100,
            Alignment::Bottom => 0b00001000
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ImageSegmentation {
    None,
    Horizontal,
    Vertical,
    Sides,
}

impl Default for ImageSegmentation {
    fn default() -> Self {
        ImageSegmentation::None
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TextWrap {
    ToMaximumOf(UIUnits),
    WithinParent
}

#[derive(Clone, PartialEq, Debug)]
pub enum WidgetType {
    Text { text: String, font: Option<FontIdentifier>, font_size: u32, wrap: Option<TextWrap> },
    Window { image: Option<String>, segment: ImageSegmentation },
}

impl WidgetType {
    pub fn text<S>(text: S, font_size: u32) -> WidgetType where S: Into<String> {
        WidgetType::Text { text: text.into(), font_size, font: None, wrap: None }
    }
    pub fn wrapped_text<S>(text: S, font_size: u32, wrap : TextWrap) -> WidgetType where S: Into<String> {
        WidgetType::Text { text: text.into(), font_size, font: None, wrap : Some(wrap) }
    }
    pub fn window() -> WidgetType {
        WidgetType::Window { image: None, segment: ImageSegmentation::None }
    }
    pub fn image<S>(image: S) -> WidgetType where S: Into<String> {
        WidgetType::Window { image: Some(image.into()), segment: ImageSegmentation::None }
    }

    pub fn set_text<S>(&mut self, text_in: S) where S: Into<String> {
        match self {
            WidgetType::Text { ref mut text, .. } => *text = text_in.into(),
            _ => ()
        };
    }
    pub fn set_text_wrap(&mut self, wrap_in : Option<TextWrap>) {
        match self {
            WidgetType::Text { ref mut wrap, .. } => *wrap = wrap_in,
            _ => ()
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum EventConsumption {
    EventTypes(u32),
    Key(keyboard::Key),
    Compound(Box<EventConsumption>, Box<EventConsumption>)
}

impl EventConsumption {
    pub fn all() -> EventConsumption {
        EventConsumption::EventTypes(0xffffffff)
    }
    pub fn none() -> EventConsumption {
        EventConsumption::EventTypes(0x0)
    }
    pub fn of_types(event_types: &Vec<UIEventType>) -> EventConsumption {
        let mut flag = 0;
        for evt_type in event_types {
            flag = flag | evt_type.bit_flag
        }
        EventConsumption::EventTypes(flag)
    }
    pub fn mouse_press_and_release() -> EventConsumption {
        EventConsumption::EventTypes(MOUSE_PRESS.bit_flag | MOUSE_RELEASE.bit_flag)
    }
    pub fn mouse_events() -> EventConsumption {
        EventConsumption::EventTypes(MOUSE_PRESS.bit_flag | MOUSE_RELEASE.bit_flag | MOUSE_MOVE.bit_flag | MOUSE_DRAG.bit_flag | MOUSE_POSITION.bit_flag)
    }
    pub fn enter_and_leave() -> EventConsumption {
        EventConsumption::EventTypes(MOUSE_ENTERED.bit_flag | MOUSE_EXITED.bit_flag)
    }
    pub fn widget_events() -> EventConsumption {
        EventConsumption::EventTypes(WIDGET_EVENT.bit_flag)
    }
    pub fn and(&self, other : &EventConsumption) -> EventConsumption {
        if let EventConsumption::EventTypes(type_flags_a) = self {
            if let EventConsumption::EventTypes(type_flags_b) = other {
                return EventConsumption::EventTypes(*type_flags_a | *type_flags_b);
            }
        }
        EventConsumption::Compound(box self.clone(), box other.clone())
    }

    pub fn consumes_event(&self, e: &UIEvent) -> bool {
        match self {
            EventConsumption::EventTypes(flags) => (flags & e.bit_flag()) != 0,
            EventConsumption::Key(consumed_key) => match e {
                UIEvent::KeyPress { key } => key == consumed_key,
                UIEvent::KeyRelease { key } => key == consumed_key,
                _ => false
            },
            EventConsumption::Compound(a,b) => a.consumes_event(e) || b.consumes_event(e)
        }
    }
}


pub trait CustomWidgetRenderer {
    fn render(&self, widget: &Widget, state: &WidgetState, pixel_pos: Vec2f, pixel_dim: Vec2f) -> DrawList;
    fn type_id(&self) -> TypeId;
}

impl PartialEq<CustomWidgetRenderer> for CustomWidgetRenderer {
    fn eq(&self, other: &CustomWidgetRenderer) -> bool {
        self.type_id() == other.type_id()
    }
}


#[derive(Clone, PartialEq, Debug)]
pub enum WidgetState {
    NoState,
    Slider { current_value: f64 },
    Button { pressed: bool },
    Text { text: String },
    Tab { active_tab: Option<u32>, tabs: Vec<Wid>, tab_buttons: Vec<Wid> },
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sizing {
    PcntOfParent(f32),
    PcntOfParentAllowingLoop(f32), // special sizing to be parent dependent, but allowed even when parent is dependent on children. Should basically omit the element from consideration
    DeltaOfParent(UIUnits),
    ExtendToParentEdge,
    Constant(UIUnits),
    Derived,
    SurroundChildren,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Positioning {
    PcntOfParent(f32),
    CenteredInParent,
    Constant(UIUnits),
    DeltaOfWidget(Wid, UIUnits, Alignment),
    MatchWidget(Wid),
    Absolute(UIUnits)
}

impl Positioning {
    pub fn origin() -> Positioning { Positioning::Constant(0.0.ux()) }
    pub fn pcnt_of_parent(pcnt : f32) -> Positioning { Positioning::PcntOfParent(pcnt) }
    pub fn centered() -> Positioning { Positioning::CenteredInParent }
    pub fn px(n : i32) -> Positioning { Positioning::Constant(n.px()) }
    pub fn ux(f : f32) -> Positioning { Positioning::Constant(f.ux()) }
    pub fn constant(uiu: UIUnits) -> Positioning { Positioning::Constant(uiu) }
    pub fn below(other_widget : &Widget, delta_units : UIUnits) -> Positioning { Positioning::delta_of_widget(other_widget, delta_units, Alignment::Bottom) }
    pub fn above(other_widget : &Widget, delta_units : UIUnits) -> Positioning { Positioning::delta_of_widget(other_widget, delta_units, Alignment::Top) }
    pub fn left_of(other_widget : &Widget, delta_units : UIUnits) -> Positioning { Positioning::delta_of_widget(other_widget, delta_units, Alignment::Left) }
    pub fn right_of(other_widget : &Widget, delta_units : UIUnits) -> Positioning { Positioning::delta_of_widget(other_widget, delta_units, Alignment::Right) }
    pub fn delta_of_widget(other_widget: &Widget, delta_units :UIUnits, alignment : Alignment) -> Positioning {
        Positioning::DeltaOfWidget(other_widget.id(), delta_units, alignment)
    }
    pub fn match_to(other_widget : &Widget) -> Positioning { Positioning::MatchWidget(other_widget.id()) }
    pub fn absolute(uiu: UIUnits) -> Positioning { Positioning::Absolute(uiu) }

    pub fn depends_on(&self) -> Option<Wid> {
        match self {
            Positioning::DeltaOfWidget(wid,_,_) => Some(*wid),
            Positioning::MatchWidget(wid) => Some(*wid),
            _ => None
        }
    }
}
impl Default for Positioning {
    fn default() -> Self {
        Positioning::origin()
    }
}
impl Sizing {
    pub fn match_parent() -> Sizing { Sizing::PcntOfParent(1.0f32) }
    pub fn surround_children() -> Sizing { Sizing::SurroundChildren }
    pub fn pcnt_of_parent(f: f32) -> Sizing { Sizing::PcntOfParent(f) }
    pub fn ux(ux : f32) -> Sizing { Sizing::Constant(ux.ux()) }
    pub fn px(px : i32) -> Sizing { Sizing::Constant(px.px()) }
    pub fn derived() -> Sizing { Sizing::Derived }
    pub fn relative(uiu : UIUnits) -> Sizing { Sizing::DeltaOfParent(uiu) }
    pub fn constant(uiu : UIUnits) -> Sizing { Sizing::Constant(uiu) }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Alignment {
    Top,
    Left,
    Right,
    Bottom,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct WidgetCallbackRef(u32);

impl WidgetCallbackRef {
    pub fn new(id : u32, inherent : bool) -> WidgetCallbackRef {
        if inherent { WidgetCallbackRef::inherent_callback(id) }
        else { WidgetCallbackRef::listener_callback(id) }
    }
    pub fn inherent_callback(id : u32) -> WidgetCallbackRef { WidgetCallbackRef(id | 0b10000000000000000000000000000000) }
    pub fn listener_callback(id : u32) -> WidgetCallbackRef { WidgetCallbackRef(id & 0b01111111111111111111111111111111) }

    pub fn is_inherent(&self) -> bool { (self.0 & 0b10000000000000000000000000000000) != 0 }
    pub fn id(&self) -> u32 { self.0 & 0b01111111111111111111111111111111 }
}


#[derive(Clone, PartialEq)]
pub struct Widget {
    pub name : Option<Str>,
    pub callbacks: Vec<WidgetCallbackRef>,
    pub(crate) id: Wid,
    pub parent_id: Option<Wid>,
    pub widget_type: WidgetType,
    pub size: [Sizing; 2],
    pub position: [Positioning; 2],
    pub alignment: [Alignment; 2],
    pub border: Border,
    pub color: Color,
    pub margin: UIUnits,
    pub showing: bool,
    pub accepts_focus: bool,
    pub ignores_parent_bounds : bool,
    pub draw_layer : GUILayer,
    pub state_override: Option<WidgetState>,
    pub event_consumption: EventConsumption,
    pub tooltip : Option<String>
}


impl Widget {
    pub fn new(widget_type: WidgetType) -> Widget {
        Widget {
            id: create_wid(),
            parent_id: None,
            widget_type,
            size: [Sizing::Constant(10.0.ux()), Sizing::Constant(10.0.ux())],
            position: [Positioning::Constant(0.0.ux()), Positioning::Constant(0.0.ux())],
            alignment: [Alignment::Left, Alignment::Top],
            border: Border::default(),
            color: Color::white(),
            margin: 0.ux(),
            showing: true,
            state_override: None,
            callbacks: Vec::new(),
            event_consumption: EventConsumption::none(),
            accepts_focus: false,
            ignores_parent_bounds: false,
            name : None,
            draw_layer : GUILayer::Main,
            tooltip: None
        }
    }

    pub fn id(&self) -> Wid {
        if self.id == NO_WID {
            error!("Attempting to access the ID of a widget that has not been initialized, this is basically never a good idea {:?}", Backtrace::new());
        }
        self.id
    }

    pub fn has_id(&self) -> bool {
        self.id != NO_WID
    }

    pub fn set_id(&mut self, wid : Wid) {
        self.id = wid;
    }

    pub fn clear_id(&mut self) {
        self.set_id(NO_WID);
    }

    pub fn div() -> Widget {
        Widget::new(WidgetType::Window { image : None, segment : ImageSegmentation::None })
            .color(Color::clear())
            .size(Sizing::SurroundChildren, Sizing::SurroundChildren)
    }

    pub fn image<S>(image: S, color: Color, border_width: u8) -> Widget where S: Into<String> {
        Widget::new(WidgetType::image(image))
            .color(color)
            .border(Border { color: Color::black(), width: border_width, sides: BorderSides::all() })
    }

    pub fn segmented_image<S>(image: S, color: Color, segment: ImageSegmentation) -> Widget where S: Into<String> {
        Widget::new(WidgetType::Window { image: Some(image.into()), segment })
            .color(color)
    }

    pub fn text<S>(text: S, font_size: u32) -> Widget where S: Into<String> {
        Widget::new(WidgetType::text(text, font_size))
            .color(Color::black())
            .size(Sizing::Derived, Sizing::Derived)
    }

    pub fn wrapped_text<S>(text: S, font_size: u32, wrap : TextWrap) -> Widget where S: Into<String> {
        Widget::new(WidgetType::wrapped_text(text, font_size, wrap))
            .color(Color::black())
            .size(Sizing::Derived, Sizing::Derived)
    }

    pub fn window(color: Color, border_width: u8) -> Widget {
        Widget::new(WidgetType::window())
            .color(color)
            .border(Border { color: Color::black(), width: border_width, sides: BorderSides::all() })
    }

    pub fn none() -> Widget {
        Widget::new(WidgetType::Window { image: None, segment: ImageSegmentation::None })
    }

    pub fn dependent_on_children(&self) -> bool {
        Widget::sizing_dependent_on_children(self.size[0]) || Widget::sizing_dependent_on_children(self.size[1])
    }
    pub fn sizing_dependent_on_children(sizing: Sizing) -> bool {
        match sizing {
            Sizing::SurroundChildren => true,
            _ => false
        }
    }

    pub fn add_callback<State: 'static, U: Fn(&mut State, &UIEvent) + 'static>(&mut self, function: U, inherent : bool) -> &mut Self {
        let new_id = WIDGET_CALLBACKS.with(|widget_callbacks| widget_callbacks.borrow_mut().add_callback(function));
        self.callbacks.push(WidgetCallbackRef::new(new_id, inherent));
        self
    }
    pub fn add_inherent_callback<State: 'static, U: Fn(&mut State, &UIEvent) + 'static>(&mut self, function: U) -> &mut Self {
        self.add_callback(function, true);
        self
    }
    pub fn with_callback<State: 'static, U: Fn(&mut State, &UIEvent) + 'static>(mut self, function: U) -> Self {
        self.add_callback(function, false);
        self
    }
    pub fn with_inherent_callback<State: 'static, U: Fn(&mut State, &UIEvent) + 'static>(mut self, function: U) -> Self {
        self.add_callback(function, true);
        self
    }
    pub fn add_callback_2<State: 'static, OtherState: 'static, U: Fn(&mut State, &mut OtherState, &UIEvent) + 'static>(&mut self, function: U) -> &mut Self {
        let new_id = WIDGET_CALLBACKS.with(|widget_callbacks| widget_callbacks.borrow_mut().add_callback_2(function));
        self.callbacks.push(WidgetCallbackRef::new(new_id, false));
        self
    }
    pub fn with_callback_2<State: 'static, OtherState: 'static, U: Fn(&mut State, &mut OtherState, &UIEvent) + 'static>(mut self, function: U) -> Self {
        self.add_callback_2(function);
        self
    }
    pub fn clear_callbacks(&mut self) {
        self.callbacks.retain(|c| c.is_inherent());
    }

    // -------------------------------------------------- private functions -----------------------------------------------------------
    fn parent_based_size(sizing: Sizing) -> bool {
        match sizing {
            Sizing::Constant(_) => false,
            Sizing::Derived => false,
            Sizing::DeltaOfParent(_) => true,
            Sizing::PcntOfParent(_) => true,
            Sizing::PcntOfParentAllowingLoop(_) => true, // be careful with this one, it's intended to be allowed in cases parent normally wouldn't be
            Sizing::ExtendToParentEdge => true, // likewise this one
            Sizing::SurroundChildren => false
        }
    }
    fn parent_based_pos(pos: Positioning) -> bool {
        match pos {
            Positioning::PcntOfParent(_) => true,
            Positioning::CenteredInParent => true,
            Positioning::Constant(_) => false,
            Positioning::DeltaOfWidget(_, _, _) => false,
            Positioning::MatchWidget(_) => false,
            Positioning::Absolute(_) => false,
        }
    }
    pub(crate) fn validate(&self) -> bool {
        if self.alignment[0] == Alignment::Top || self.alignment[0] == Alignment::Bottom ||
            self.alignment[1] == Alignment::Left || self.alignment[1] == Alignment::Right {
            error!("Widget created that has nonsensical alignment");
            return false;
        }
        if self.accepts_focus && !self.event_consumption.consumes_event(&UIEvent::MouseRelease { button: MouseButton::Left, pos: EventPosition::absolute(v2(0.0, 0.0), v2(0.0, 0.0)) }) {
            error!("Widget created that theoretically accepts focus, but ignores mouse events that would give it focus");
            return false;
        }
        true
    }


    pub(crate) fn depends_on(&self) -> Vec<Wid> {
        let mut ret = Vec::new();
        for pos in self.position.iter() {
            if let Some(dependent) = pos.depends_on() {
                ret.push(dependent);
            }
        }
        ret
    }
}

impl Default for Widget {
    fn default() -> Self {
        Widget::none()
    }
}


pub trait WidgetContainer {
    fn for_all_widgets<F: FnMut(&mut Widget)>(&mut self, func: F);

    fn reapply_all(&mut self, gui : &mut GUI) {
        self.for_all_widgets(|w| w.reapply(gui));
    }
    fn apply_all(mut self, gui : &mut GUI) -> Self where Self : Sized {
        self.reapply_all(gui);
        self
    }

    fn draw_layer_for_all(mut self, draw_layer : GUILayer) -> Self where Self : Sized {
        self.for_all_widgets(|w| { w.draw_layer = draw_layer; });
        self
    }
}



impl DelegateToWidget for Widget {
    fn as_widget(&mut self) -> &mut Widget {
        self
    }

    fn as_widget_immut(& self) -> &Widget {
        self
    }
}

impl WidgetContainer for Widget {
    fn for_all_widgets<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
        (func)(self)
    }
}


pub struct WidgetCallbackRegistry {
    pub callback_id_counter: u32,
    pub sub_callbacks: AnyMap,
}


type TypedWidgetCallback<T> = Fn(&mut T, &UIEvent);
type TypedWidgetCallback2<T, U> = Fn(&mut T, &mut U, &UIEvent);

struct TypedWidgetCallbacks<T> {
    pub callbacks: HashMap<u32, Box<TypedWidgetCallback<T>>>
}

struct TypedWidgetCallbacks2<T, U> {
    pub callbacks: HashMap<u32, Box<TypedWidgetCallback2<T, U>>>
}

impl WidgetCallbackRegistry {
    pub fn add_callback<State: 'static, U: Fn(&mut State, &UIEvent) + 'static>(&mut self, callback: U) -> u32 {
        let new_id = self.callback_id_counter;
        self.callback_id_counter += 1;
        let mut typed_callbacks = self.sub_callbacks.remove::<TypedWidgetCallbacks<State>>().unwrap_or_else(|| TypedWidgetCallbacks { callbacks: HashMap::new() });

        typed_callbacks.callbacks.insert(new_id, box callback);

        self.sub_callbacks.insert(typed_callbacks);
        new_id
    }

    pub fn add_callback_2<State: 'static, OtherState: 'static, U: Fn(&mut State, &mut OtherState, &UIEvent) + 'static>(&mut self, callback: U) -> u32 {
        let new_id = self.callback_id_counter;
        self.callback_id_counter += 1;
        let mut typed_callbacks = self.sub_callbacks.remove::<TypedWidgetCallbacks2<State, OtherState>>().unwrap_or_else(|| TypedWidgetCallbacks2 { callbacks: HashMap::new() });

        typed_callbacks.callbacks.insert(new_id, box callback);

        self.sub_callbacks.insert(typed_callbacks);
        new_id
    }


    pub fn execute_callback<State: 'static>(&self, id: u32, t: &mut State, evt: &UIEvent) {
        if let Some(typed_callbacks) = self.sub_callbacks.get::<TypedWidgetCallbacks<State>>() {
            if let Some(f) = typed_callbacks.callbacks.get(&id) {
                (f)(t, evt)
            }
        }
    }

    pub fn execute_callback_2<State: 'static, OtherState: 'static>(&self, id: u32, t: &mut State, u: &mut OtherState, evt: &UIEvent) {
        if let Some(typed_callbacks) = self.sub_callbacks.get::<TypedWidgetCallbacks2<State, OtherState>>() {
            if let Some(f) = typed_callbacks.callbacks.get(&id) {
                (f)(t, u, evt)
            }
        }
    }

    pub fn new() -> WidgetCallbackRegistry {
        WidgetCallbackRegistry {
            sub_callbacks: AnyMap::new(),
            callback_id_counter: 1,
        }
    }
}



#[derive(Debug)]
struct TestStruct {
    pub i: i32,
    pub f: f32,
}

#[test]
pub fn test() {
    let mut callbacks = WidgetCallbackRegistry::new();

    let func_id = callbacks.add_callback(|t: &mut TestStruct, evt: &UIEvent| {
        match evt {
            UIEvent::MousePress { .. } => t.i = 3,
            _ => t.f = 9.0
        };
    });

    let mut test_struct = TestStruct {
        i: 0,
        f: 0.0,
    };

    let evt = UIEvent::MousePress { button: MouseButton::Left, pos: EventPosition::absolute(v2(20.0, 20.0), v2(20.0, 20.0)) };
    callbacks.execute_callback(func_id, &mut test_struct, &evt);

    println!("{:?}", test_struct);
    assert_eq!(test_struct.i, 3);
}