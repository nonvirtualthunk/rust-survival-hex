use piston_window::MouseButton;
use piston_window::Key;
use piston_window::GenericEvent;
use piston_window::Button;

use common::prelude::*;

use gui::GUI;
use widgets::WidgetState;

#[derive(Clone, Debug)]
pub struct EventPosition {
    pub pixel_pos : Vec2f,
    pub absolute_pos : Vec2f,
    pub local_pos : Vec2f
}
impl EventPosition {
    pub fn absolute(v : Vec2f, p : Vec2f) -> EventPosition {
        EventPosition {
            absolute_pos : v,
            pixel_pos : p,
            local_pos : v
        }
    }

    pub fn new(absolute: Vec2f, p : Vec2f, local: Vec2f) -> EventPosition {
        EventPosition {
            absolute_pos : absolute,
            pixel_pos : p,
            local_pos : local
        }
    }

    pub fn relative_to(&self, other_pos : Vec2f) -> EventPosition {
        EventPosition {
            absolute_pos : self.absolute_pos,
            local_pos : self.absolute_pos - other_pos,
            pixel_pos : self.pixel_pos
        }
    }
}

#[derive(Clone, Debug)]
pub enum WidgetEvent {
    ButtonClicked,
    RadioChanged{ new_index : i32 }
}

#[derive(Clone, Debug)]
pub enum UIEvent {
    MousePress { pos : EventPosition, button : MouseButton },
    MouseRelease { pos : EventPosition, button : MouseButton },
    Drag { pos : EventPosition, delta : Vec2f, button : MouseButton},
    MouseMove { pos : EventPosition, delta : Vec2f },
    MousePosition { pos : EventPosition },
    MouseEntered { pos : EventPosition },
    MouseExited { pos : EventPosition },
    Scroll { delta : Vec2f },
    KeyPress { key : Key },
    KeyRelease { key : Key },
    Text { text : String },
    Resize { size : Vec2f },
    Focus { has_focus : bool },
    WidgetStateChanged { old_state : WidgetState, new_state : WidgetState },
    WidgetEvent (WidgetEvent)
}

#[derive(Clone, Copy)]
pub struct UIEventType {
    pub bit_flag : u32
}
impl UIEventType {
    pub fn new(bit_flag : u32) -> UIEventType { UIEventType { bit_flag } }
}

pub mod ui_event_types {
    use super::*;
    pub const MOUSE_PRESS : UIEventType =           UIEventType { bit_flag : 0b00000000000000000000000000000001 };
    pub const MOUSE_RELEASE : UIEventType =         UIEventType { bit_flag : 0b00000000000000000000000000000010 };
    pub const DRAG : UIEventType =                  UIEventType { bit_flag : 0b00000000000000000000000000000100 };
    pub const MOUSE_MOVE : UIEventType =            UIEventType { bit_flag : 0b00000000000000000000000000001000 };
    pub const MOUSE_POSITION : UIEventType =        UIEventType { bit_flag : 0b00000000000000000000000000010000 };
    pub const SCROLL : UIEventType =                UIEventType { bit_flag : 0b00000000000000000000000000100000 };
    pub const KEY_PRESS : UIEventType =             UIEventType { bit_flag : 0b00000000000000000000000001000000 };
    pub const KEY_RELEASE : UIEventType =           UIEventType { bit_flag : 0b00000000000000000000000010000000 };
    pub const TEXT : UIEventType =                  UIEventType { bit_flag : 0b00000000000000000000000100000000 };
    pub const RESIZE : UIEventType =                UIEventType { bit_flag : 0b00000000000000000000001000000000 };
    pub const FOCUS : UIEventType =                 UIEventType { bit_flag : 0b00000000000000000000010000000000 };
    pub const WIDGET_STATE_CHANGED : UIEventType =  UIEventType { bit_flag : 0b00000000000000000000100000000000 };
    pub const MOUSE_ENTERED : UIEventType =         UIEventType { bit_flag : 0b00000000000000000001000000000000 };
    pub const MOUSE_EXITED : UIEventType =          UIEventType { bit_flag : 0b00000000000000000010000000000000 };
    pub const WIDGET_EVENT : UIEventType =          UIEventType { bit_flag : 0b00000000000000000100000000000000 };
    pub const MOUSE_EVENT_TYPES : [UIEventType; 7] = [MOUSE_PRESS, MOUSE_RELEASE, DRAG, MOUSE_MOVE, MOUSE_POSITION, MOUSE_ENTERED, MOUSE_EXITED];
    pub const KEY_EVENTS : [UIEventType; 2] = [KEY_PRESS, KEY_RELEASE];
}


impl UIEvent {
    pub fn event_type(&self) -> UIEventType {
        use ui_event_types::*;

        match self {
            UIEvent::MousePress { .. } =>           MOUSE_PRESS,
            UIEvent::MouseRelease { .. } =>         MOUSE_RELEASE,
            UIEvent::Drag { .. } =>                 DRAG,
            UIEvent::MouseMove { .. } =>            MOUSE_MOVE,
            UIEvent::MousePosition { .. } =>        MOUSE_POSITION,
            UIEvent::MouseEntered { .. } =>         MOUSE_ENTERED,
            UIEvent::MouseExited { .. } =>          MOUSE_EXITED,
            UIEvent::Scroll { .. } =>               SCROLL,
            UIEvent::KeyPress { .. } =>             KEY_PRESS,
            UIEvent::KeyRelease { .. } =>           KEY_RELEASE,
            UIEvent::Text { .. } =>                 TEXT,
            UIEvent::Resize { .. } =>               RESIZE,
            UIEvent::Focus { .. } =>                FOCUS,
            UIEvent::WidgetStateChanged { .. } =>   WIDGET_STATE_CHANGED,
            UIEvent::WidgetEvent { .. } =>          WIDGET_EVENT
        }
    }
    pub fn bit_flag(&self) -> u32 {
        self.event_type().bit_flag
    }

    pub fn from_piston_event<E : GenericEvent>(units_per_pixel :f32, current_mouse_pixel_pos : Vec2f, event : E) -> Option<UIEvent> {
        let translate_coords = |xy: [f64; 2]| v2(xy[0] as f32 * units_per_pixel, xy[1] as f32 * units_per_pixel);
        let current_mouse_pos = current_mouse_pixel_pos * units_per_pixel;

        if let Some(xy) = event.mouse_cursor_args() {
            let v = translate_coords(xy);
            return Some(UIEvent::MousePosition { pos : EventPosition::absolute(v, v2(xy[0] as f32, xy[1] as f32)) });
        }

        // Note, see what the hell this actually is doing
        if let Some(rel_xy) = event.mouse_relative_args() {
            let rel_v = translate_coords(rel_xy);
//            println!("Relative mouse movement arg: {:?}", rel_v);
            return Some(UIEvent::MouseMove { pos : EventPosition::absolute(current_mouse_pos, current_mouse_pixel_pos), delta : rel_v });
        }

        if let Some(xy) = event.mouse_scroll_args() {
            return Some(UIEvent::Scroll { delta : v2(xy[0] as f32, xy[1] as f32) });
        }

        if let Some(button) = event.press_args() {
            let v = current_mouse_pos;
            match button {
                Button::Keyboard(key) => return Some(UIEvent::KeyPress { key }),
                Button::Mouse(button) => return Some(UIEvent::MousePress { pos : EventPosition::absolute(v, current_mouse_pixel_pos), button }),
                _ => return None
            }
        }

        if let Some(button) = event.release_args() {
            let v = current_mouse_pos;
            match button {
                Button::Keyboard(key) => return Some(UIEvent::KeyRelease { key }),
                Button::Mouse(button) => return Some(UIEvent::MouseRelease { pos : EventPosition::absolute(v, current_mouse_pixel_pos), button }),
                _ => return None
            }
        }

        if let Some(text) = event.text_args() {
            return Some(UIEvent::Text { text });
        }

        if let Some(dim) = event.resize_args() {
            return Some(UIEvent::Resize{ size : v2(dim[0] as f32, dim[1] as f32) });
        }

        if let Some(b) = event.focus_args() {
            return Some(UIEvent::Focus { has_focus : b });
        }

        None
    }
}