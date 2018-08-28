use graphics::core::Quad;
use common::prelude::*;
use std::collections::HashMap;
use backtrace::Backtrace;
use graphics::GraphicsResources;
use graphics::GraphicsAssets;
use graphics::GraphicsWrapper;
use graphics::DrawList;
use graphics::TextLayout;
use common::Color;
use std::ops::Deref;
use std::ops::DerefMut;
use graphics::Text;
use std::collections::HashSet;
use std::collections::BTreeSet;
use piston_window::Viewport;
use graphics::DEFAULT_FONT_IDENTIFIER;
use num::Float;
use num::ToPrimitive;
use common::Rect;

use widgets::*;
use std::any::TypeId;
use std::any::Any;
use events::UIEvent;
use core::mem;

use anymap::any::UncheckedAnyExt;
use anymap::AnyMap;

use std::collections::VecDeque;
//use anymap::any::Any;

use piston_window::MouseButton;
use piston_window::GenericEvent;
use events::EventPosition;
use piston_window;

pub use gui_event_handling::*;
pub use gui_rendering::*;

use widget_delegation::DelegateToWidget;
use std::time::Instant;
use std::time::Duration;
use events::ui_event_types::HOVER_START;
use events::ui_event_types::HOVER_END;
use widgets::TextDisplayWidget;


#[derive(Clone, Copy, PartialEq, Neg, Debug)]
pub enum UIUnits {
    Pixels(i32),
    Units(f32),
}

impl UIUnits {
    pub fn px(&self, pixels_per_ux: f32) -> i32 {
        match self {
            UIUnits::Pixels(px) => *px,
            UIUnits::Units(ux) => (ux * pixels_per_ux).round() as i32
        }
    }
    pub fn ux(&self, pixels_per_ux: f32) -> f32 {
        match self {
            UIUnits::Pixels(px) => *px as f32 / pixels_per_ux,
            UIUnits::Units(ux) => *ux
        }
    }
}

impl Into<Positioning> for UIUnits {
    fn into(self) -> Positioning {
        Positioning::constant(self)
    }
}
impl Into<Sizing> for UIUnits {
    fn into(self) -> Sizing {
        Sizing::constant(self)
    }
}

pub trait ToGUIPixels {
    fn px(&self) -> UIUnits;
}

pub trait ToGUIUnit {
    fn ux(&self) -> UIUnits;
}

impl<T: ToPrimitive> ToGUIPixels for T {
    fn px(&self) -> UIUnits {
        UIUnits::Pixels(self.to_f32().unwrap().round() as i32)
    }
}

impl<T: ToPrimitive> ToGUIUnit for T {
    fn ux(&self) -> UIUnits {
        UIUnits::Units(self.to_f32().unwrap())
    }
}

#[derive(Clone,Copy,Default)]
pub struct Modifiers {
    pub ctrl : bool,
    pub shift : bool,
    pub alt : bool
}

pub(crate) struct WidgetReification {
    pub(crate) widget: Widget,
    pub(crate) draw_list: DrawList,
    pub(crate) children: VecDeque<Wid>,
    pub(crate) scroll: Vec2f,
    pub(crate) position: Vec2f,
    pub(crate) inner_position: Vec2f,
    pub(crate) dimensions: Vec2f,
    pub(crate) inner_dimensions: Vec2f,
    pub(crate) bounding_box: Option<Rect<f32>>,
    pub(crate) inner_bounding_box: Option<Rect<f32>>,
    pub(crate) widget_state: WidgetState,
    pub(crate) tooltip: Option<(Wid, Wid)>,
    pub(crate) internal_callbacks: Vec<u32>
}

impl WidgetReification {
    pub fn new(widget: Widget) -> WidgetReification {
        WidgetReification {
            widget,
            draw_list: DrawList::none(),
            children: VecDeque::new(),
            scroll: v2(0.0, 0.0),
            position: v2(0.0, 0.0),
            dimensions: v2(0.0, 0.0),
            inner_dimensions: v2(0.0, 0.0),
            inner_position: v2(0.0, 0.0),
            bounding_box: None,
            inner_bounding_box: None,
            widget_state: WidgetState::NoState,
            tooltip: None,
            internal_callbacks: Vec::new()
        }
    }

    pub fn bounds(&self) -> Rect<f32> {
        Rect::new(self.position.x, self.position.y, self.dimensions.x, self.dimensions.y)
    }
}


pub struct GUI {
    pub(crate) widget_reifications: HashMap<Wid, WidgetReification>,
    pub(crate) top_level_widgets: BTreeSet<Wid>,
    pub(crate) gui_size: Vec2f,
    pub(crate) viewport: Viewport,
    pub(crate) modified_set: HashSet<Wid>,
    pub(crate) focused_widget: Option<Wid>,
    pub(crate) current_mouse_pos: Vec2f,
    pub(crate) queued_events: HashMap<TypeId, Vec<UIEvent>>,
    pub(crate) events_by_widget: HashMap<Wid, Vec<UIEvent>>,
    pub(crate) empty_vec: Vec<UIEvent>,
    pub(crate) active_modifiers: Modifiers,
    pub(crate) moused_over_widget: Option<Wid>,
    pub(crate) active_mouse_button: Option<MouseButton>,
    pub(crate) hover_start : Instant,
    pub(crate) hover_threshold: Duration,
    pub(crate) hover_widget: Option<Wid>,
}

impl GUI {
    pub fn new() -> GUI {
        GUI {
            widget_reifications: HashMap::new(),
            top_level_widgets: BTreeSet::new(),
            gui_size: v2(100.0, 100.0),
            viewport: Viewport { window_size: [256, 256], draw_size: [256, 256], rect: [0, 0, 256, 256] },
            modified_set: HashSet::new(),
            focused_widget: None,
            current_mouse_pos: v2(0.0, 0.0),
            queued_events: HashMap::new(),
            events_by_widget: HashMap::new(),
            empty_vec: Vec::new(),
            active_modifiers: Modifiers::default(),
            moused_over_widget: None,
            active_mouse_button: None,
            hover_threshold: Duration::from_secs(1),
            hover_start: Instant::now(),
            hover_widget: None,
        }
    }

    pub fn mark_all_modified(&mut self) {
        for key in self.widget_reifications.keys() {
            self.modified_set.insert(*key);
        }
    }

    pub fn new_id(&mut self) -> Wid {
        create_wid()
    }

    pub fn append_child(&self, children : VecDeque<Wid>, new_child : &Widget) -> VecDeque<Wid> {
        let mut ret = VecDeque::with_capacity(children.len() + 1);

        let all_handled = |wids : &Vec<Wid>, ret : &VecDeque<Wid>| -> bool { wids.iter().all(|wid| ret.contains(wid)) };
        let handled = |wid : &Wid, ret : &VecDeque<Wid>| -> bool { ret.contains(wid) };
        let all_handled_or_not_present = |wids : &Vec<Wid>, ret : &VecDeque<Wid>, remaining : &VecDeque<Wid>| -> bool { !wids.iter().any(|wid| remaining.contains(wid)) };
        let depends_on = |wid : &Wid| -> Vec<Wid> { if wid == &new_child.id() { new_child.depends_on() } else { self.widget_reification(*wid).widget.depends_on() } };

        ret.extend(children.iter().filter(|c| depends_on(c).is_empty()));


        let mut max_iter = 1000;
        let mut remaining : VecDeque<Wid> = children.into_iter().filter(|c| ! handled(c, &ret)).collect();
        if new_child.depends_on().is_empty() {
            ret.push_back(new_child.id());
        } else {
            remaining.push_back(new_child.id());
        }
        loop {
            if let Some(next) = remaining.pop_back() {
                let depends_on = depends_on(&next);
                if depends_on.is_empty() || all_handled(&depends_on, &ret) {
                    ret.push_back(next);
                } else {
                    if ! all_handled_or_not_present(&depends_on, &ret, &remaining) {
                        remaining.push_front(next);
                    } else {
                        ret.push_back(next);
                    }
                }
            } else {
                break;
            }
            max_iter -= 1;
            if max_iter <= 3 {
                let w = remaining.front().unwrap();
                let signifier = if w == &new_child.id() { format!("new_child: {:?}", new_child.signifier()) } else { format!("existing child: {:?}", self.widget_reification(*w).widget.signifier()) };
                error!("[{}] Got into infinite topological sorting loop, breaking : {}", max_iter, signifier);
            }
            if max_iter <= 0 {
                break;
            }
        }



//        let (no_dep_children, dep_children) = children.into_iter()
//            .map(|c| (c,self.widget_reification(c).widget.depends_on()))
//            .partition(|(c,dep)| dep.is_empty());
//
//        let new_dep = new_child.depends_on();
//        if new_dep.is_empty() {
//            ret.push_front(new_child.id());
//        }
//        ret.extend(no_dep_children.map(|(c,a_)| c));
//
//        let mut dep_children = dep_children.collect_vec();
//        if new_dep.non_empty() {
//            dep_children.push((new_child.id(), new_dep));
//        }
//        while dep_children.non_empty() {
//            let mut i = 0;
//            while i < dep_children.len() {
//                if let Some((child, deps)) = dep_children.get(i) {
////                    if deps.iter().all
//                }
//            }
//        }

        ret
    }

    pub fn apply_widget(&mut self, widget: &mut Widget) {
        let wid = widget.id();

        // ensure that the child is now registered with its parent, if any
        if let Some(parent) = widget.parent_id {
            let new_children = if let Some(parent_state) = self.widget_reifications.get(&parent) {
                if !parent_state.children.contains(&wid) {
                    Some(self.append_child(parent_state.children.clone(), widget))
                } else {
                    None
                }
            } else {
                error!("Attempted to add widget as child of other non-existent widget: {:?}, parent: {:?}", widget.signifier(), parent);
                panic!("Attempted to add widget as child of other non-existent widget")
            };

            if let Some(new_children) = new_children {
                if let Some(parent_state) = self.widget_reifications.get_mut(&parent) {
                    parent_state.children = new_children;
                }
            }
        } else {
            self.top_level_widgets.insert(wid);
        }

        let existing_state = self.widget_reifications.remove(&wid);
        if let Some(mut state) = existing_state {
            let mut mark_modified = false;
            let new_tooltip = if state.widget != *widget {
                let new_tooltip = self.handle_tooltip(&state.tooltip, &state.widget.tooltip, widget);

                state.widget = widget.clone();

                if let Some(ref new_tooltip) = new_tooltip {
                    state.tooltip = Some((new_tooltip.body.id(), new_tooltip.text.id()));
                }

                // clear the override on the stored widget, don't want that continuously retrigger
                state.widget.state_override = None;
                // if there's a state override set, update the state to the new value
                if let Some(state_override) = &widget.state_override {
                    if state.widget_state != *state_override {
                        // TODO: trigger event indicating that the state was changed
                        state.widget_state = state_override.clone();
                    }
                }
                mark_modified = true;
                new_tooltip
            } else {
                None
            };

            self.widget_reifications.insert(wid, state);
            if let Some(mut new_tooltip) = new_tooltip {
                new_tooltip.reapply_all(self);
            }
            if mark_modified {
                self.mark_widget_modified(wid);
            }
        } else {
            let new_tooltip = self.handle_tooltip(&None, &None, widget);

            let state = WidgetReification::new(widget.clone());

            self.widget_reifications.insert(wid, state);

            if let Some(mut new_tooltip) = new_tooltip {
                new_tooltip.reapply_all(self);
            }

            self.mark_widget_modified(wid);
        }
    }

    fn handle_tooltip(&mut self, existing : &Option<(Wid,Wid)>, old_tooltip: &Option<String>, widget : &mut Widget) -> Option<TextDisplayWidget> {
        if old_tooltip == &widget.tooltip {
            return None;
        }
        if let Some(ref tooltip_string) = widget.tooltip {
            if let Some((tooltip_body_id, tooltip_text_id)) = existing {
                info!("Modifying existing tooltip state, from {} to {}", old_tooltip.as_ref().unwrap_or(&strf("N/A")), tooltip_string);
                let mut tooltip_text = self.widget_reifications.remove(&tooltip_text_id).expect("we recorded the presence of a tooltip, but it could not be found");
                tooltip_text.widget.widget_type.set_text(tooltip_string.clone());
                self.widget_reifications.insert(*tooltip_text_id, tooltip_text);

                None
            } else {
                let mut tdw = TextDisplayWidget::new(tooltip_string.clone(), 14, None, ImageSegmentation::None)
                    .parent_id(widget.id())
                    .draw_layer_for_all(GUILayer::Overlay)
                    .ignore_parent_bounds()
                    .position(Positioning::Constant(1.ux()), Positioning::Constant(1.ux()))
                    .named("tooltip")
                    .showing(false);

                if tooltip_string.len() > 40 {
                    tdw = tdw.wrapped(Sizing::Constant(20.ux()));
                    tdw.text.set_x(Positioning::centered());
                }

                let tdw_id = tdw.id();
                let tdw_text_id = tdw.text.id();

                widget.add_inherent_callback(move |ctxt: &mut WidgetContext, evt: &UIEvent| {
                    match evt {
                        UIEvent::HoverStart { pos, .. } => {
                            let (x, y) = ((pos.absolute_pos.x + 1.0).ux(), (pos.absolute_pos.y + 1.0).ux());
                            ctxt.alter_widget(tdw_id, move |w| {
                                w.set_showing(true);
                                w.set_position(Positioning::absolute(x), Positioning::absolute(y));
                            })
                        }
                        UIEvent::HoverEnd { .. } => ctxt.alter_widget(tdw_id, |w| { w.set_showing(false); }),
                        _ => ()
                    }
                }).add_consumption(EventConsumption::EventTypes(HOVER_START.bit_flag | HOVER_END.bit_flag));

                Some(tdw)
            }
        } else {
            None
        }
    }

    pub fn mark_widget_modified(&mut self, wid : Wid) {
        self.modified_set.insert(wid);

        let mut cur_parent = self.widget_reification(wid).widget.parent_id;
        let mut to_mark = Vec::new();
        loop {
            if let Some(parent_id) = cur_parent {
                if let Some(reification) = self.widget_reification_opt(parent_id) {
                    let widget = &reification.widget;
                    if widget.dependent_on_children() {
                        to_mark.push(parent_id);
                    }
                    cur_parent = widget.parent_id;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        self.modified_set.extend(to_mark);
    }

    pub  fn remove_widget<D : DelegateToWidget>(&mut self, widget: &mut D) {
        let widget = widget.as_widget();
        if widget.has_id() {
            self.remove_widget_by_id(widget.id());
            widget.clear_id();
        } else {
            warn!("Attempted to remove widget that had never been added, that's...fine, but weird");
        }
    }

    pub fn remove_widget_by_id(&mut self, wid : Wid) {
        self.remove_widget_by_id_intern(wid, true);
    }

    fn remove_widget_by_id_intern(&mut self, wid : Wid, remove_from_parent : bool) {
        if ! self.widget_reifications.contains_key(&wid) {
            return;
        }

        let existing_state = self.widget_reifications.remove(&wid).expect("widget state must exist if ID exists");

        for child in existing_state.children {
            self.remove_widget_by_id_intern(child, false); // don't remove from parent, because we're already in the midst of doing that right now
        }

        trace!("Removing widget by id: {}", wid);
        self.widget_reifications.remove(&wid);
        self.events_by_widget.remove(&wid);
        if self.focused_widget == Some(wid) {
            self.focused_widget = None;
        }
        if self.moused_over_widget == Some(wid) {
            self.moused_over_widget = None;
        }
        if self.hover_widget == Some(wid) {
            self.hover_widget = None;
        }

        self.modified_set.remove(&wid);
        self.top_level_widgets.remove(&wid);

        if remove_from_parent {
            if let Some(parent_id) = existing_state.widget.parent_id {
                let mut parent_state = self.widget_reifications.remove(&parent_id).expect("Trying to remove child after parent has been removed (or never existed)");
                if let Some(index_in_parent) = parent_state.children.iter().position(|c| c == &wid) {
                    parent_state.children.remove(index_in_parent);
                    self.modified_set.insert(parent_id);
                }
                self.widget_reifications.insert(parent_id, parent_state);
            }
        }
    }

    /*
    We need two things. We need to know whether each widget consumes the event or not, and we need to actually process the callbacks, if any. In theory, right now,
    these can accept arbitrary state in the callbacks, potentially multiple different kinds of state. In practice we probably only need... a few. The case of nested
    game modes might complicate that. I wouldn't really want to lock myself down to one. So I think we separate out the processing from the consumption calculation,
    that'll be the easiest thing.
    */

    pub(crate) fn widget_reification(&self, wid: Wid) -> &WidgetReification {
        if let Some(reification) = self.widget_reifications.get(&wid) {
            reification
        } else {
            panic!("Could not find reification for given widget id: {:?}", wid);
        }
    }

    pub(crate) fn widget_reification_opt(&self, wid: Wid) -> Option<&WidgetReification> {
        self.widget_reifications.get(&wid)
    }

    pub(crate) fn child_widgets_of(&self, wid : Wid) -> Vec<&Widget> {
        self.widget_reification(wid).children.iter().map(|w| &self.widget_reification(*w).widget).collect_vec()
    }

    pub fn widget_state(&self, wid : Wid) -> &WidgetState {
        self.widget_reifications.get(&wid).map(|reif| &reif.widget_state).unwrap_or(&WidgetState::NoState)
    }
}

pub(crate) struct WidgetStatePlaceholder {}

pub(crate) enum EventRouting {
    FocusedWidget,
    MousedWidget,
    SpecificWidget(Wid),
    NoWidget,
}

impl Default for GUI {
    fn default() -> Self {
        GUI::new()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use spectral::prelude::*;

    struct TestState {
        pub i: i32
    }

    struct FooState {
        pub f: f32
    }

    #[test]
    pub fn test_ui_event_handling() {
        use GUI;
        let mut gui = GUI::new();
        gui.viewport = Viewport {
            draw_size: [500, 500],
            window_size: [1000, 1000],
            rect: [0, 0, 1000, 1000],
        };
        gui.gui_size = v2(100.0, 100.0);

        let mut graphics_assets = GraphicsAssets::new("survival");

        // create a main widget
        let widget = Widget::new(WidgetType::window())
            .position(Positioning::Constant(10.ux()), Positioning::Constant(10.ux()))
            .size(Sizing::Constant(20.ux()), Sizing::Constant(20.ux()))
            .only_consume(EventConsumption::all())
            .with_callback(|state: &mut TestState, evt: &UIEvent| {
                state.i = match evt {
                    UIEvent::KeyPress { .. } => 9,
                    UIEvent::MousePress { .. } => 1,
                    _ => 2
                };
            })
            .apply(&mut gui);

        // get the gui updated and positions computed
        gui.update(&mut graphics_assets, false);
        assert_that(&gui.widget_reification(widget.id()).position).is_equal_to(&v2(10.0, 10.0));

        // set up the mutable state we'll be referring to
        let mut test_state = TestState { i: 0 };
        // trigger a mouse press event, this should intersect with the bounds of the widget and be handled, triggering the callback
        let handled = gui.handle_ui_event(&UIEvent::MousePress { pos: EventPosition::absolute(v2(20.0, 20.0), v2(20.0, 20.0)), button: MouseButton::Left }, &mut test_state);
        assert_that(&handled).is_equal_to(&true);
        assert_that(&test_state.i).is_equal_to(&1);

        test_state.i = 0;
        // trigger a mouse press event outside the bounds of the widget, this should not be handled and no callback executed
        let handled = gui.handle_ui_event(&UIEvent::MousePress { pos: EventPosition::absolute(v2(40.0, 20.0), v2(20.0, 20.0)), button: MouseButton::Left }, &mut test_state);
        assert_that(&handled).is_equal_to(&false);
        assert_that(&test_state.i).is_equal_to(&0);

        // trigger a key press event. Since only mouse presses have occurred, not releases, nothing should have focus and the key press should not be handled
        let handled = gui.handle_ui_event(&UIEvent::KeyPress { key: piston_window::Key::L }, &mut test_state);
        assert_that(&handled).is_equal_to(&false);
        assert_that(&test_state.i).is_equal_to(&0);

        // create a sub widget under the first widget
        let mut sub_widget = Widget::new(WidgetType::window())
            .parent(&widget)
            .position(Positioning::PcntOfParent(0.2), Positioning::PcntOfParent(0.2))
            .size(Sizing::PcntOfParent(0.6), Sizing::PcntOfParent(0.6))
            .only_consume(EventConsumption::mouse_events())
            .with_callback(|state: &mut TestState, evt: &UIEvent| { state.i = 2; })
            .apply(&mut gui);

        // update and check that position is reasonable
        gui.update(&mut graphics_assets, false);
        assert_that(&gui.widget_reification(sub_widget.id()).position).is_equal_to(&v2(10.0 + 0.2 * 20.0, 10.0 + 0.2 * 20.0));

        // trigger the same mouse press as before, that should now go tot he sub widget
        let handled = gui.handle_ui_event(&UIEvent::MousePress { pos: EventPosition::absolute(v2(20.0, 20.0), v2(20.0, 20.0)), button: MouseButton::Left }, &mut test_state);
        assert_that(&handled).is_equal_to(&true);
        assert_that(&test_state.i).is_equal_to(&2);

        // still nothing should have any focus, trigger a mouse release over the sub widget, it is not marked as accepting focus so nothing should happen
        assert_that(&gui.focused_widget).is_equal_to(&None);
        let handled = gui.handle_ui_event(&UIEvent::MouseRelease { pos: EventPosition::absolute(v2(20.0, 20.0), v2(20.0, 20.0)), button: MouseButton::Left }, &mut test_state);
        assert_that(&gui.focused_widget).is_equal_to(&None);

        // mark the sub widget as accepting focus
        sub_widget = sub_widget.accepts_focus(true).apply(&mut gui);

        // do the mouse release again, now that the sub widget accepts focus it should get it
        let handled = gui.handle_ui_event(&UIEvent::MouseRelease { pos: EventPosition::absolute(v2(20.0, 20.0), v2(20.0, 20.0)), button: MouseButton::Left }, &mut test_state);
        assert_that(&gui.focused_widget).is_equal_to(&Some(sub_widget.id()));
        assert_that(&handled).is_equal_to(&true);

        // now the same key press from before should get picked up by the focus of subwidget, the subwidget doesn't actually consume key events though, so it
        // should bubble up to the higher widget
        let handled = gui.handle_ui_event(&UIEvent::KeyPress { key: piston_window::Key::L }, &mut test_state);
        assert_that(&handled).is_equal_to(&true);
        assert_that(&test_state.i).is_equal_to(&9);
    }


    #[test]
    pub fn test_multi_state_callbacks() {
        use GUI;
        let mut gui = GUI::new();
        gui.viewport = Viewport {
            draw_size: [500, 500],
            window_size: [1000, 1000],
            rect: [0, 0, 1000, 1000],
        };
        gui.gui_size = v2(100.0, 100.0);

        let mut graphics_assets = GraphicsAssets::new("survival");

        // create a main widget
        let widget = Widget::window(Color::white(), 1)
            .position(Positioning::Constant(10.ux()), Positioning::Constant(10.ux()))
            .size(Sizing::Constant(20.ux()), Sizing::Constant(20.ux()))
            .only_consume(EventConsumption::all())
            .with_callback_2(|state: &mut TestState, other : &mut FooState, evt: &UIEvent| {
                state.i = other.f as i32;
            })
            .apply(&mut gui);

        // get the gui updated and positions computed
        gui.update(&mut graphics_assets, false);

        let mut test_state = TestState {
            i : 0
        };
        let mut foo_state = FooState {
            f : 1.0
        };

        gui.handle_ui_event_2(&UIEvent::MousePress { pos: EventPosition::absolute(v2(20.0, 20.0), v2(20.0, 20.0)), button: MouseButton::Left }, &mut test_state, &mut foo_state);

        assert_that(&test_state.i).is_equal_to(&1);
    }

    #[test]
    pub fn test_button_and_event_transmutation() {
        use pretty_env_logger;
        pretty_env_logger::init();

        use GUI;
        let mut gui = GUI::new();
        gui.viewport = Viewport {
            draw_size: [500, 500],
            window_size: [1000, 1000],
            rect: [0, 0, 1000, 1000],
        };
        gui.gui_size = v2(100.0, 100.0);

        let mut graphics_assets = GraphicsAssets::new("survival");

        // create a main widget
        let widget = Button::new("Test Button Here")
            .position(Positioning::Constant(10.ux()), Positioning::Constant(10.ux()))
            .with_on_click_2(|a: &mut TestState, b : &mut FooState| { a.i = b.f as i32; } )
            .apply(&mut gui);

        // get the gui updated and positions computed
        gui.update(&mut graphics_assets, false);

        let mut test_state = TestState {
            i : 0
        };
        let mut foo_state = FooState {
            f : 1.0
        };

        let evt = UIEvent::MousePress { pos: EventPosition::absolute(v2(11.0, 11.0), v2(20.0, 20.0)), button: MouseButton::Left };
        gui.handle_ui_event_for_self(&evt);
        gui.handle_ui_event_2(&evt, &mut test_state, &mut foo_state);

        let evt = UIEvent::MouseRelease { pos: EventPosition::absolute(v2(11.0, 11.0), v2(20.0, 20.0)), button: MouseButton::Left };
        gui.handle_ui_event_for_self(&evt);
        gui.handle_ui_event_2(&evt, &mut test_state, &mut foo_state);

        assert_that(&test_state.i).is_equal_to(&1);
    }
}