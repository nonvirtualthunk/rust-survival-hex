use common::Color;

use gui::*;
use graphics::ImageIdentifier;

use backtrace::Backtrace;
use events::UIEvent;
use events::WidgetEvent;

use anymap;
use anymap::any::CloneAny;
use std::any::Any;
use std::any::TypeId;
use std::mem;
use std::rc::Rc;
use events::EventPosition;
use std::collections::HashMap;
use anymap::AnyMap;
use std::sync::Mutex;
use piston_window::MouseButton;
use common::prelude::*;
use std::cell::RefCell;
use piston_window::keyboard;
use ui_event_types::*;
use events::UIEventType;
use events::ui_event_types;

use graphics::FontIdentifier;

pub use compound_widgets::*;


thread_local! {
//    static ref WIDGET_CALLBACKS : Mutex<WidgetCallbackRegistry> = Mutex::new(WidgetCallbackRegistry::new());
    pub static WIDGET_CALLBACKS : RefCell<WidgetCallbackRegistry> = RefCell::new(WidgetCallbackRegistry::new());
}

pub fn execute_global_widget_registry_callback<State : 'static>(id: u32, state: &mut State, evt: &UIEvent) {
    WIDGET_CALLBACKS.with(|registry| registry.borrow().execute_callback(id, state, evt))
}

pub fn execute_global_widget_registry_callback_2<State : 'static, OtherState : 'static>(id: u32, state: &mut State, other : &mut OtherState, evt: &UIEvent) {
    WIDGET_CALLBACKS.with(|registry| registry.borrow().execute_callback_2(id, state, other, evt))
}


#[derive(Clone,PartialEq)]
pub struct Border {
    pub width : u8,
    pub color : Color,
    pub sides : BorderSides
}
impl Default for Border {
    fn default() -> Self {
        Border { width : 0, color : Color::black(), sides : BorderSides::all() }
    }
}

#[derive(Clone,PartialEq,Eq)]
pub struct BorderSides(u8);

impl BorderSides {
    pub fn all() -> BorderSides { BorderSides(0xff) }

    pub fn three_sides(a : Alignment, b : Alignment, c : Alignment) -> BorderSides {
        BorderSides(BorderSides::alignment_to_flag(a) | BorderSides::alignment_to_flag(b) | BorderSides::alignment_to_flag(c))
    }
    pub fn two_sides(a : Alignment, b : Alignment) -> BorderSides {
        BorderSides(BorderSides::alignment_to_flag(a) | BorderSides::alignment_to_flag(b))
    }
    pub fn one_side(a : Alignment) -> BorderSides {
        BorderSides(BorderSides::alignment_to_flag(a))
    }

    pub fn has_side(&self, a : Alignment) -> bool {
        (self.0 & BorderSides::alignment_to_flag(a)) != 0
    }

    fn alignment_to_flag (a : Alignment) -> u8 {
        match a {
            Alignment::Left =>      0b00000001,
            Alignment::Right =>     0b00000010,
            Alignment::Top =>       0b00000100,
            Alignment::Bottom =>    0b00001000
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum WidgetType {
    Text { text: String, font: Option<FontIdentifier>, font_size: u32, wrap: bool },
    Window { image: Option<String> },
}

impl WidgetType {
    pub fn text<S>(text: S, font_size: u32) -> WidgetType where S: Into<String> {
        WidgetType::Text { text: text.into(), font_size, font: None, wrap: false }
    }
    pub fn window() -> WidgetType {
        WidgetType::Window { image: None }
    }
    pub fn image<S>(image: S) -> WidgetType where S: Into<String> {
        WidgetType::Window { image: Some(image.into()) }
    }

    pub fn set_text<S>(&mut self, text_in : S) where S : Into<String>{
        match self {
            WidgetType::Text { ref mut text, .. } => *text = text_in.into(),
            _ => ()
        };

    }
}

#[derive(Clone, PartialEq)]
pub enum EventConsumption {
    EventTypes(u32),
    Key(keyboard::Key),
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
        EventConsumption::EventTypes(MOUSE_PRESS.bit_flag | MOUSE_RELEASE.bit_flag | MOUSE_MOVE.bit_flag | DRAG.bit_flag | MOUSE_POSITION.bit_flag)
    }

    pub fn consumes_event(&self, e: &UIEvent) -> bool {
        match self {
            EventConsumption::EventTypes(flags) => (flags & e.bit_flag()) != 0,
            EventConsumption::Key(consumed_key) => match e {
                UIEvent::KeyPress { key } => key == consumed_key,
                UIEvent::KeyRelease { key } => key == consumed_key,
                _ => false
            }
        }
    }
}


#[derive(Clone, PartialEq, Debug)]
pub enum WidgetState {
    NoState,
    Slider { current_value: f64 },
    Button { pressed: bool },
    Text { text: String },
}

#[derive(Clone, PartialEq)]
pub struct Widget {
    pub callbacks: Vec<u32>,
    pub id: Wid,
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
    pub state_override: Option<WidgetState>,
    pub event_consumption: EventConsumption,
}


impl Widget {
    pub fn new(widget_type: WidgetType) -> Widget {
        Widget {
            id: NO_WID,
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
        }
    }

    pub fn image<S>(image: S, color: Color, border_width: u8) -> Widget where S: Into<String> {
        Widget::new(WidgetType::image(image))
            .color(color)
            .border(Border { color : Color::black(), width : border_width, sides : BorderSides::all() })
    }

    pub fn text<S>(text: S, font_size: u32) -> Widget where S: Into<String> {
        Widget::new(WidgetType::text(text, font_size))
            .color(Color::black())
            .size(Sizing::Derived, Sizing::Derived)
    }

    pub fn window(color: Color, border_width: u8) -> Widget {
        Widget::new(WidgetType::window())
            .color(color)
            .border(Border { color : Color::black(), width: border_width, sides : BorderSides::all() })
    }

    pub fn none() -> Widget {
        Widget::new(WidgetType::Window { image: None })
    }

    pub fn widget_type(mut self, widget_type: WidgetType) -> Self {
        self.set_widget_type(widget_type);
        self
    }
    pub fn set_widget_type(&mut self, widget_type: WidgetType) -> &mut Self {
        self.widget_type = widget_type;
        self
    }

    pub fn modify_widget_type<F : Fn(&mut WidgetType)>(&mut self, func: F) -> &mut Self {
        (func)(&mut self.widget_type);
        self
    }

    pub fn showing(mut self, showing: bool) -> Self {
        self.set_showing(showing);
        self
    }
    pub fn set_showing(&mut self, showing: bool) -> &mut Self {
        self.showing = showing;
        self
    }
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }
    pub fn set_border(&mut self, border: Border) -> &mut Self {
        self.border = border;
        self
    }
    pub fn border(mut self, border: Border) -> Self {
        self.set_border(border);
        self
    }
    pub fn border_width(mut self, border_width : u8) -> Self {
        self.border.width = border_width;
        self
    }
    pub fn border_color(mut self, border_color : Color) -> Self {
        self.border.color = border_color;
        self
    }
    pub fn border_sides(mut self, border_sides : BorderSides) -> Self {
        self.border.sides = border_sides;
        self
    }

    pub fn set_parent(&mut self, parent: &Widget) -> &mut Self {
        if parent.id == NO_WID {
            error!("Attempting to add a widget to a parent that has no ID, this is not acceptable");
        }
        self.parent_id = Some(parent.id);
        self
    }
    pub fn parent(mut self, parent: &Widget) -> Self {
        self.set_parent(parent);
        self
    }
    pub fn parent_id(mut self, parent: Wid) -> Self {
        if parent == NO_WID {
            error!("Attempting to add a widget to a parent that has no ID, this is not acceptable");
        }
        self.parent_id = Some(parent);
        self
    }

    pub fn position(mut self, x: Positioning, y: Positioning) -> Self {
        self.set_position(x, y);
        self
    }
    pub fn set_position(&mut self, x: Positioning, y: Positioning) {
        self.position = [x, y];
    }
    pub fn x(mut self, x: Positioning) -> Self {
        self.position[0] = x;
        self
    }
    pub fn y(mut self, y: Positioning) -> Self {
        self.position[1] = y;
        self
    }
    pub fn alignment(mut self, x: Alignment, y: Alignment) -> Self {
        if (x == Alignment::Top || x == Alignment::Bottom) && (y == Alignment::Left || y == Alignment::Right) {
            self.alignment = [y, x];
        } else {
            self.alignment = [x, y];
        }
        self
    }
    pub fn size(mut self, w: Sizing, h: Sizing) -> Self {
        self.size = [w, h];
        self
    }
    pub fn width(mut self, w: Sizing) -> Self {
        self.size[0] = w;
        self
    }
    pub fn height(mut self, h: Sizing) -> Self {
        self.size[1] = h;
        self
    }
    pub fn event_consumption(mut self, consumption: EventConsumption) -> Self {
        self.event_consumption = consumption;
        self
    }
    pub fn accepts_focus(mut self, accept: bool) -> Self {
        self.accepts_focus = accept;
        self
    }
    pub fn reapply(&mut self, gui: &mut GUI) {
        if !self.validate() {
            error!("Constructing invalid widget\n{:?}", Backtrace::new());
        }
        if self.id == NO_WID {
            self.id = gui.new_id();
        }

        gui.apply_widget(self);
        // the state override will have been applied to gui, if present, we can reset now
        self.state_override = None;
    }
    pub fn apply(mut self, gui: &mut GUI) -> Self {
        self.reapply(gui);

        self
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

    pub fn with_callback<State : 'static, U: Fn(&mut State, &UIEvent) + 'static>(mut self, function: U) -> Self {
        let new_id = WIDGET_CALLBACKS.with(|widget_callbacks| widget_callbacks.borrow_mut().add_callback(function));
        self.callbacks.push(new_id);
        self
    }
    pub fn with_callback_2<State : 'static, OtherState : 'static, U: Fn(&mut State, &mut OtherState, &UIEvent) + 'static>(mut self, function : U) -> Self {
        let new_id = WIDGET_CALLBACKS.with(|widget_callbacks| widget_callbacks.borrow_mut().add_callback_2(function));
        self.callbacks.push(new_id);
        self
    }

    // -------------------------------------------------- private functions -----------------------------------------------------------
    fn parent_based_size(sizing: Sizing) -> bool {
        match sizing {
            Sizing::Constant(_) => false,
            Sizing::Derived => false,
            Sizing::DeltaOfParent(_) => true,
            Sizing::PcntOfParent(_) => true,
            Sizing::SurroundChildren => false
        }
    }
    fn parent_based_pos(pos: Positioning) -> bool {
        match pos {
            Positioning::PcntOfParent(_) => true,
            Positioning::CenteredInParent => true,
            Positioning::Constant(_) => false,
            Positioning::DeltaOfWidget(_, _, _) => false,
        }
    }
    fn validate(&self) -> bool {
//        if self.parent_id.is_none() {
//            if Widget::parent_based_size(self.size[0]) || Widget::parent_based_size(self.size[1]) ||
//                Widget::parent_based_pos(self.position[0]) || Widget::parent_based_pos(self.position[1]) {
//                return false;
//            }
//        }
        if self.alignment[0] == Alignment::Top || self.alignment[0] == Alignment::Bottom ||
            self.alignment[1] == Alignment::Left || self.alignment[1] == Alignment::Right {
            error!("Widget created that has nonsensical alignment");
            return false;
        }
        if self.accepts_focus && !self.event_consumption.consumes_event(&UIEvent::MouseRelease { button: MouseButton::Left, pos: EventPosition::absolute(v2(0.0, 0.0), v2(0.0,0.0)) }) {
            error!("Widget created that theoretically accepts focus, but ignores mouse events that would give it focus");
            return false;
        }
        true
    }
}

impl Default for Widget {
    fn default() -> Self {
        Widget::none()
    }
}


//pub trait CompoundWidget {
//    fn main_widget(&self) -> &Widget;
//
//    fn main_widget_mut(&mut self) -> &mut Widget;
////    fn on_main_widget<F : FnMut(Widget) -> Widget>(&mut self, f : F) -> &mut Self;
//
//    fn position(&mut self, x: Positioning, y: Positioning) -> &mut Self {
//        self.main_widget_mut().set_position(x,y);
//        self
//    }
//
////    fn parent(&mut self, parent: &Widget) -> &mut Self {
//////        self.on_main_widget(|w| w.parent(parent))
////    }
////
////    fn width(&mut self, width: Sizing) -> &mut Self {
//////        self.on_main_widget(|w| w.width(width))
////    }
//}
//
//impl CompoundWidget for Button {
//    fn main_widget(&self) -> &Widget {
//        &self.body
//    }
//
//
//    fn main_widget_mut(&mut self) -> &mut Widget {
//        &mut self.body
//    }
//
////    fn on_main_widget<F: FnMut(Widget) -> Widget>(&mut self, mut f: F) -> &mut Self {
////        (f)(self.body);
////        self
////    }
//}



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
    pub fn add_callback<State : 'static, U: Fn(&mut State, &UIEvent) + 'static>(&mut self, callback: U) -> u32 {
        let new_id = self.callback_id_counter;
        self.callback_id_counter += 1;
        let mut typed_callbacks = self.sub_callbacks.remove::<TypedWidgetCallbacks<State>>().unwrap_or_else(|| TypedWidgetCallbacks { callbacks: HashMap::new() });

        typed_callbacks.callbacks.insert(new_id, box callback);

        self.sub_callbacks.insert(typed_callbacks);
        new_id
    }

    pub fn add_callback_2<State : 'static, OtherState : 'static, U: Fn(&mut State, &mut OtherState, &UIEvent) + 'static>(&mut self, callback: U) -> u32 {
        let new_id = self.callback_id_counter;
        self.callback_id_counter += 1;
        let mut typed_callbacks = self.sub_callbacks.remove::<TypedWidgetCallbacks2<State, OtherState>>().unwrap_or_else(|| TypedWidgetCallbacks2 { callbacks: HashMap::new() });

        typed_callbacks.callbacks.insert(new_id, box callback);

        self.sub_callbacks.insert(typed_callbacks);
        new_id
    }


    pub fn execute_callback<State : 'static>(&self, id: u32, t: &mut State, evt: &UIEvent) {
        if let Some(typed_callbacks) = self.sub_callbacks.get::<TypedWidgetCallbacks<State>>() {
            if let Some(f) = typed_callbacks.callbacks.get(&id) {
                (f)(t, evt)
            }
        }
    }

    pub fn execute_callback_2<State : 'static, OtherState : 'static>(&self, id: u32, t: &mut State, u : &mut OtherState, evt: &UIEvent) {
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


pub trait AsWidget {

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

    let evt = UIEvent::MousePress { button: MouseButton::Left, pos: EventPosition::absolute(v2(20.0,20.0), v2(20.0,20.0)) };
    callbacks.execute_callback(func_id, &mut test_struct, &evt);

    println!("{:?}", test_struct);
    assert_eq!(test_struct.i, 3);
}