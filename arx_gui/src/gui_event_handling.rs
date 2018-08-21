use anymap::AnyMap;
use backtrace::Backtrace;
use common::Color;
use common::prelude::*;
use common::Rect;
use events::EventPosition;
use events::UIEvent;
use graphics::core::Quad;
use graphics::DEFAULT_FONT_IDENTIFIER;
use graphics::DrawList;
use graphics::GraphicsAssets;
use graphics::GraphicsResources;
use graphics::GraphicsWrapper;
use graphics::Text;
use graphics::TextLayout;
use gui::GUI;
use widgets::Wid;
use gui::WidgetStatePlaceholder;
use piston_window;
use piston_window::GenericEvent;
use piston_window::MouseButton;
use piston_window::Viewport;
use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;
use widgets::*;
use gui::Modifiers;
use widget_delegation::DelegateToWidget;
use std::time::Instant;


pub struct WidgetAlteration {
    pub widget_id : Wid,
    pub alteration : Box<Fn(&mut Widget)>
}

pub struct WidgetContext {
    pub widget_id: Wid,
    pub widget_state: WidgetState,
    pub triggered_events: Vec<UIEvent>,
    pub widget_alterations : Vec<WidgetAlteration>
}

impl WidgetContext {
    pub fn trigger_event(&mut self, event: UIEvent) {
        self.triggered_events.push(event);
    }
    pub fn update_state(&mut self, new_state: WidgetState) {
        self.widget_state = new_state;
    }
    pub fn alter_widget<F : Fn(&mut Widget) + 'static>(&mut self, wid : Wid, f : F) {
        self.widget_alterations.push(WidgetAlteration { widget_id : wid, alteration : box f });
    }
}

//use anymap::any::Any;

impl GUI {
    pub fn active_modifiers(&self) -> &Modifiers {
        &self.active_modifiers
    }
    pub fn active_mouse_button(&self) -> Option<MouseButton> {
        self.active_mouse_button
    }

    pub fn reset_events(&mut self) {
        self.events_by_widget.clear();
    }

    fn childmost_widget_containing(&self, wid: Wid, v: Vec2f) -> Option<Wid> {
        let state = self.widget_reification(wid);
        trace!("Checking for collision with target {:?} with widget {} against bb: {:?}", v, wid, state.bounding_box);
        let contains = state.bounding_box.map(|bb| bb.contains(v)).unwrap_or(false);
        if contains {
            for child in &state.children {
                if let Some(containing_child) = self.childmost_widget_containing(*child, v) {
                    return Some(containing_child);
                }
            }
            Some(wid)
        } else {
            None
        }
    }


    pub fn alter_widget<F: FnMut(&mut Widget)>(&mut self, wid: Wid, mut func : F) {
        {
            let reification = self.widget_reifications.get_mut(&wid).expect("cannot alter nonexistent widget");
            (func)(&mut reification.widget);
        }
        self.mark_widget_modified(wid);
    }

    fn handle_event_widget_intern<F: FnMut(u32, &UIEvent) -> ()>(&mut self, wid: Wid, event: &UIEvent, is_placeholder: bool, mut func: F) -> bool {
        let (parent_id, accepts_focus, consumed) = {
            if is_placeholder {
                trace!("Pushing event {:?} into vec for widget {:?}", event, self.widget_reification(wid).widget.signifier());
                self.events_by_widget.entry(wid).or_insert_with(|| Vec::new()).push(event.clone());

                let mut mark_modified = false;
                let mut widget_state = if let Some(reification) = self.widget_reifications.remove(&wid) {
                    reification
                } else {
                    error!("Reification did not exist for widget {:?} when trying to handle event {:?}", wid, event);
                    return false;
                };

                let alterations = if widget_state.widget.callbacks.non_empty() {
                    let callbacks = widget_state.widget.callbacks.clone();
                    let mut widget_context = WidgetContext { widget_state: widget_state.widget_state.clone(), triggered_events: Vec::new(), widget_id : widget_state.widget.id(), widget_alterations : Vec::new() };
                    for callback in callbacks {
                        execute_global_widget_registry_callback(callback.id(), &mut widget_context, event);
                        execute_global_widget_registry_callback_2(callback.id(), self, &mut widget_context, event);
                    }

                    if widget_state.widget_state != widget_context.widget_state {
                        trace!("Widget state changed from self event, new state is {:?}, marking wid {} for modified", widget_context.widget_state, wid);
                        mark_modified = true;
                        self.enqueue_event(&UIEvent::WidgetStateChanged { old_state: widget_state.widget_state.clone(), new_state: widget_context.widget_state.clone() });
                        widget_state.widget_state = widget_context.widget_state.clone();
                    }

                    for evt in widget_context.triggered_events {
                        self.enqueue_event(&evt);
                    }

                    widget_context.widget_alterations
                } else {
                    Vec::new()
                };

                self.widget_reifications.insert(wid, widget_state);

                for alteration in alterations {
                    {
                        let reification = self.widget_reifications.get_mut(&alteration.widget_id).expect("cannot alter nonexistent widget");
                        (alteration.alteration)(&mut reification.widget);
                    }
                    self.mark_widget_modified(wid);
                }

                if mark_modified {
                    self.mark_widget_modified(wid);
                }
            } else {
                let widget_state = self.widget_reification(wid);
                if widget_state.widget.callbacks.non_empty() {
                    for callback in &widget_state.widget.callbacks {
                        trace!("Executing callback for widget {}", wid);
                        (func)(callback.id(), event);
                    }
                }
            }

            let widget_state = self.widget_reification(wid);
            let consumed = if widget_state.widget.event_consumption.consumes_event(event) {
                trace!("Event {:?} consumed by {}, returning", event, widget_state.widget.signifier());
                Some(true)
            } else if widget_state.widget.parent_id.is_none() {
                trace!("Event {:?} not consumed, but no parent, returning", event);
                Some(false)
            } else {
                None
            };

            (widget_state.widget.parent_id, widget_state.widget.accepts_focus, consumed)
        };

        if let UIEvent::MouseRelease { .. } = event {
            if accepts_focus {
                self.focused_widget = Some(wid);
            } else {
                self.focused_widget = None;
            }
        };

        if let Some(consumed) = consumed {
            trace!("Event {:?} was consumed by widget {:?}", event, self.widget_reification(wid).widget.signifier());
            consumed
        } else if let Some(parent_id) = parent_id {
            trace!("Event not consumed, parent present, passing event to parent : {:?}", self.widget_reification(parent_id).widget.signifier());
            match event {
                UIEvent::WidgetEvent { .. } => self.handle_event_widget_intern(parent_id, &event.clone_with_most_recently_from_widget(wid), is_placeholder, func),
                _ => self.handle_event_widget_intern(parent_id, event, is_placeholder, func),
            }
        } else {
            false
        }
    }

    pub(crate) fn enqueue_event(&mut self, evt : &UIEvent) {
        self.queued_events.iter_mut().foreach(|(t, vec)| {
            vec.push(evt.clone());
        });
    }

    pub(crate) fn enqueue_event_excepting_self(&mut self, evt : &UIEvent) {
        self.queued_events.iter_mut().foreach(|(t, vec)| {
            if t != &TypeId::of::<WidgetStatePlaceholder>() {
                vec.push(evt.clone());
            }
        });
    }

    fn handle_event_widget_2<State: 'static, OtherState: 'static>(&mut self, wid: Wid, event: &UIEvent, state: &mut State, other_state: &mut OtherState) -> bool {
        self.handle_event_widget_intern(wid, event, false, |callback, evt| {
            execute_global_widget_registry_callback_2(callback, state, other_state, evt)
        })
    }

    fn handle_event_widget<State: 'static>(&mut self, wid: Wid, event: &UIEvent, state: &mut State) -> bool {
        let is_placeholder = TypeId::of::<State>() == TypeId::of::<WidgetStatePlaceholder>();
        self.handle_event_widget_intern(wid, event, is_placeholder, |callback, evt| {
            execute_global_widget_registry_callback(callback, state, evt)
        })
    }

    pub fn handle_ui_event_for_self(&mut self, ui_event: &UIEvent) -> bool {
        self.handle_ui_event::<WidgetStatePlaceholder>(ui_event, &mut WidgetStatePlaceholder {})
    }

    pub fn handle_ui_event<'a, 'b, State: 'static>(&'a mut self, ui_event: &UIEvent, state: &'b mut State) -> bool {
        let type_id = TypeId::of::<State>();

        self.update_mouse_state(ui_event);

        self.update_key_modifiers(ui_event);

        let route_to = self.widget_to_route_to(ui_event);

        let ret = if let Some(wid) = route_to {
            match ui_event {
                UIEvent::MouseMove { .. } | UIEvent::MousePosition { .. } => (),
                other => trace!("Routing event {:?} to widget {}", ui_event, wid)
            };
            self.handle_event_widget(wid, &ui_event, state)
        } else {
            match ui_event {
                UIEvent::MouseMove { .. } | UIEvent::MousePosition { .. } => (),
                other => trace!("Could not route event {:?}, no routeable widget", ui_event)
            };

            false
        };

        // if we're doing the self-update, check to see if we've switched which widget we're mousing over
        if type_id == TypeId::of::<WidgetStatePlaceholder>() {
            let altered_pos = match ui_event {
                UIEvent::MousePosition { pos } => Some(pos.clone()),
                UIEvent::MouseDrag { pos, .. } => Some(pos.clone()),
                _ => None
            };

            if let Some(pos) = altered_pos {
                let now = Instant::now();
                // if it was hovering before, send an event indicating its end
                if let Some(hover_widget) = self.hover_widget {
                    self.enqueue_event(&UIEvent::HoverEnd { pos : pos.clone(), over_widget : hover_widget });
                }
                // the mouse has been moved, restart the hover
                self.hover_start = now;
                self.hover_widget = None;

                if route_to != self.moused_over_widget {
                    let cur_moused_over = self.moused_over_widget;
                    if let Some(prev) = cur_moused_over {
                        self.enqueue_event(&UIEvent::MouseExited { pos : pos.clone(), from_widget : prev, to_widget : route_to });
                    }
                    if let Some(new) = route_to {
                        self.enqueue_event(&UIEvent::MouseEntered { pos : pos.clone() , from_widget : cur_moused_over, to_widget : new });
                    }
                    self.moused_over_widget = route_to;
                }
            }
        }

        // each queued events can then enque more events, so we keep checking, up to a maximum depth of 10 (don't want infiniloops)
        for i in 0..10 {
            if i >= 9 {
                warn!("Possible infiniloop in event queueing, cutting off after one more iteration.");
            }
            let queued_events = self.queued_events.remove(&type_id).unwrap_or_else(|| Vec::new());
            self.queued_events.insert(type_id, Vec::new());
            if queued_events.is_empty() {
                break;
            }
            for queued_event in &queued_events {
                self.handle_ui_event(queued_event, state);
            }
        }

        ret
    }

    pub fn handle_ui_event_2<State: 'static, OtherState: 'static>(&mut self, ui_event: &UIEvent, state: &mut State, other_state: &mut OtherState) -> bool {
        let type_id = TypeId::of::<(State, OtherState)>();

        let mut queued_events = self.queued_events.remove(&type_id).unwrap_or_else(|| Vec::new());
        for queued_event in &queued_events {
            self.handle_ui_event_2(queued_event, state, other_state);
        }
        queued_events.clear();
        self.queued_events.insert(type_id, queued_events);

        self.update_mouse_state(ui_event);

        if let Some(wid) = self.widget_to_route_to(ui_event) {
            match ui_event {
                UIEvent::MouseMove { .. } | UIEvent::MousePosition { .. } => (),
                other => trace!("Routing event {:?} to widget {}", ui_event, wid)
            };
            self.handle_event_widget_2(wid, &ui_event, state, other_state)
        } else {
            match ui_event {
                UIEvent::MouseMove { .. } | UIEvent::MousePosition { .. } => (),
                other => trace!("Could not route event {:?}, no routeable widget", ui_event)
            };

            false
        }
    }

    fn update_key_modifiers(&mut self, ui_event: &UIEvent) {
        match ui_event {
            UIEvent::KeyPress { key } => {
                match key {
                    piston_window::Key::LCtrl | piston_window::Key::RCtrl => self.active_modifiers.ctrl = true,
                    piston_window::Key::LShift | piston_window::Key::RShift => self.active_modifiers.shift = true,
                    piston_window::Key::LAlt | piston_window::Key::RAlt => self.active_modifiers.alt = true,
                    _ => ()
                };
            },
            UIEvent::KeyRelease { key } => {
                match key {
                    piston_window::Key::LCtrl | piston_window::Key::RCtrl => self.active_modifiers.ctrl = false,
                    piston_window::Key::LShift | piston_window::Key::RShift => self.active_modifiers.shift = false,
                    piston_window::Key::LAlt | piston_window::Key::RAlt => self.active_modifiers.alt = false,
                    _ => ()
                };
            },
            _ => ()
        };
    }

    fn update_mouse_state(&mut self, ui_event: &UIEvent) {
        self.current_mouse_pos = match ui_event {
            UIEvent::MousePosition { pos } => pos.absolute_pos,
            UIEvent::MouseMove { pos, .. } => pos.absolute_pos,
            UIEvent::MouseRelease { pos, .. } => pos.absolute_pos,
            UIEvent::MousePress { pos, .. } => pos.absolute_pos,
            UIEvent::MouseDrag { pos, .. } => pos.absolute_pos,
            _ => self.current_mouse_pos
        };

        self.active_mouse_button = match ui_event {
            UIEvent::MousePress { button, .. } => Some(*button),
            UIEvent::MouseRelease { .. } => None,
            _ => self.active_mouse_button
        };
    }

    fn widget_to_route_to(&self, ui_event: &UIEvent) -> Option<Wid> {
        use gui::EventRouting::*;
        let routing = match ui_event {
            UIEvent::Text { .. } => FocusedWidget,
            UIEvent::KeyPress { .. } => FocusedWidget,
            UIEvent::KeyRelease { .. } => FocusedWidget,
            UIEvent::MouseEntered { to_widget, .. } => SpecificWidget(*to_widget),
            UIEvent::MouseExited { from_widget, .. } => SpecificWidget(*from_widget),
            UIEvent::WidgetEvent { originating_widget, .. } => SpecificWidget(*originating_widget),
            UIEvent::CustomEvent { originating_widget, .. } => SpecificWidget(*originating_widget),
            _ => MousedWidget
        };

        match routing {
            FocusedWidget => self.focused_widget,
            MousedWidget => {
                trace!("Trying to find moused over widget for pos {:?}", self.current_mouse_pos);
                let mut hit_wid = None;
                for wid in &self.top_level_widgets {
                    hit_wid = self.childmost_widget_containing(*wid, self.current_mouse_pos);
                    if hit_wid.is_some() {
                        break;
                    }
                }
                trace!("Hit wid: {:?}", hit_wid);
                hit_wid
            },
            SpecificWidget(widget) => Some(widget),
            NoWidget => None
        }
    }

    pub fn convert_event<E: GenericEvent>(&mut self, event: E) -> Option<UIEvent> {
        UIEvent::from_piston_event(self.ux_per_pixel(), self.current_mouse_pos * self.pixels_per_ux(), self.active_mouse_button(), event)
    }

    pub fn handle_event_for_self<E: GenericEvent>(&mut self, event: E) -> bool {
        self.handle_event(event, &mut WidgetStatePlaceholder {})
    }

    pub fn handle_event<E: GenericEvent, State: 'static>(&mut self, event: E, state: &mut State) -> bool {
        if let Some(ui_event) = self.convert_event(event) {
            self.handle_ui_event(&ui_event, state)
        } else {
            false
        }
    }

    pub fn events_for(&self, widget: &Widget) -> &Vec<UIEvent> {
        self.events_by_widget.get(&widget.id()).unwrap_or_else(|| &self.empty_vec)
    }

    pub fn signifier_for(&self, widget : Wid) -> String {
        self.widget_reification(widget).widget.signifier()
    }

}