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
//use anymap::any::Any;

use piston_window::MouseButton;
use piston_window::GenericEvent;
use events::EventPosition;
use piston_window;

pub type WidIntern = usize;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug, Display)]
pub struct Wid(WidIntern);

pub static NO_WID: Wid = Wid(0);


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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sizing {
    PcntOfParent(f32),
    DeltaOfParent(UIUnits),
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
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Alignment {
    Top,
    Left,
    Right,
    Bottom,
}

struct WidgetInternal {
    widget: Widget,
    draw_list: DrawList,
    children: Vec<Wid>,
    scroll: Vec2f,
    position: Vec2f,
    inner_position: Vec2f,
    dimensions: Vec2f,
    inner_dimensions: Vec2f,
    bounding_box: Option<Rect<f32>>,
    inner_bounding_box: Option<Rect<f32>>,
    widget_state: WidgetState,
}

impl WidgetInternal {
    pub fn new(widget: Widget) -> WidgetInternal {
        WidgetInternal {
            widget,
            draw_list: DrawList::none(),
            children: vec![],
            scroll: v2(0.0, 0.0),
            position: v2(0.0, 0.0),
            dimensions: v2(0.0, 0.0),
            inner_dimensions: v2(0.0, 0.0),
            inner_position: v2(0.0, 0.0),
            bounding_box: None,
            inner_bounding_box: None,
            widget_state: WidgetState::NoState,
        }
    }

    pub fn bounds(&self) -> Rect<f32> {
        Rect::new(self.position.x, self.position.y, self.dimensions.x, self.dimensions.y)
    }
}


pub struct GUI {
    id_counter: WidIntern,
    widget_states: HashMap<Wid, WidgetInternal>,
    top_level_widgets: HashSet<Wid>,
    gui_size: Vec2f,
    viewport: Viewport,
    modified_set: HashSet<Wid>,
    focused_widget: Option<Wid>,
    current_mouse_pos: Vec2f,
    queued_events: HashMap<TypeId, Vec<UIEvent>>,
    events_by_widget: HashMap<Wid, Vec<UIEvent>>,
    empty_vec: Vec<UIEvent>,
}

impl GUI {
    pub fn new() -> GUI {
        GUI {
            id_counter: 0,
            widget_states: HashMap::new(),
            top_level_widgets: HashSet::new(),
            gui_size: v2(100.0, 100.0),
            viewport: Viewport { window_size: [256, 256], draw_size: [256, 256], rect: [0, 0, 256, 256] },
            modified_set: HashSet::new(),
            focused_widget: None,
            current_mouse_pos: v2(0.0, 0.0),
            queued_events: HashMap::new(),
            events_by_widget: HashMap::new(),
            empty_vec: Vec::new(),
        }
    }

    pub fn new_id(&mut self) -> Wid {
        self.id_counter = self.id_counter + 1;
        Wid(self.id_counter)
    }

    pub fn apply_widget(&mut self, widget: &Widget) {
        let wid = widget.id;

        // ensure that the child is now registered with its parent, if any
        if let Some(parent) = widget.parent_id {
            if let Some(parent_state) = self.widget_states.get_mut(&parent) {
                if !parent_state.children.contains(&wid) {
                    parent_state.children.push(wid);
                }
            } else {
                panic!("Attempted to add widget as child of other non-existent widget")
            }
        } else {
            self.top_level_widgets.insert(wid);
        }

        let existing_state = self.widget_states.remove(&wid);
        if let Some(mut state) = existing_state {
            if state.widget != *widget {
                state.widget = widget.clone();
                // if there's a state override set, update the state to the new value
                if let Some(state_override) = &widget.state_override {
                    if state.widget_state != *state_override {
                        // TODO: trigger event indicating that the state was changed
                        state.widget_state = state_override.clone();
                    }
                }
                self.modified_set.insert(wid);
            }
            self.widget_states.insert(wid, state);
        } else {
            self.modified_set.insert(wid);
            self.widget_states.insert(wid, WidgetInternal::new(widget.clone()));
        }
    }

    pub fn remove_widget(&mut self, widget: &mut Widget) {
        if widget.id != NO_WID {
            let existing_state = self.widget_states.remove(&widget.id).expect("widget state must exist if ID exists");

            for child in existing_state.children {
                self.remove_widget_by_id(child);
            }
            self.remove_widget_by_id(widget.id);

            widget.id = NO_WID;
        } else {
            warn!("Attempted to remove widget that had never been added, that's...fine, but weird");
        }
    }

    pub fn remove_widget_by_id(&mut self, wid : Wid) {
        self.widget_states.remove(&wid);
        self.events_by_widget.remove(&wid);
        if self.focused_widget == Some(wid) {
            self.focused_widget = None;
        }
        self.modified_set.remove(&wid);
        self.top_level_widgets.remove(&wid);
    }


    pub fn compute_raw_widget_size(&self, size: Sizing, anchor_size: f32) -> f32 {
        let pixels_per_ux = self.pixels_per_ux();
        match size {
            Sizing::Constant(constant) => constant.ux(pixels_per_ux),
            Sizing::PcntOfParent(pcnt) => anchor_size * pcnt,
            Sizing::DeltaOfParent(delta) => anchor_size + delta.ux(pixels_per_ux),
            _ => 0.0
        }
    }

    pub fn update_widget_state(&mut self, g: &mut GraphicsAssets, wid: Wid, secondary: bool) -> bool {
        let mut internal_state = self.widget_states.remove(&wid).expect("every wid called to update must have existing internal state");

        let child_dependent = {
            let widget = &internal_state.widget;
            let parent_position = widget.parent_id.map(|parent_id| self.widget_states.get(&parent_id).expect("parent must exist").inner_position);
            let parent_size = widget.parent_id.map(|parent_id| self.widget_states.get(&parent_id).expect("parent must exist").inner_dimensions);
            let parent_size = widget.parent_id.map(|parent_id| self.widget_states.get(&parent_id).expect("parent must exist").inner_dimensions);
            let parent_bounding_box = widget.parent_id.map(|parent_id| self.widget_states.get(&parent_id).expect("parent must exist").inner_bounding_box)
                .unwrap_or_else(|| Some(Rect::new(0.0, 0.0, self.gui_size.x, self.gui_size.y)));
            let pixels_per_ux = self.pixels_per_ux();

            if parent_bounding_box.is_some() && widget.showing {
                let parent_bounding_box = parent_bounding_box.unwrap();
                for axis in 0..2 {
                    let anchor_pos = match widget.position[axis] {
                        Positioning::DeltaOfWidget(other_wid, _, _) => self.widget_states.get(&other_wid).expect("dependent wid must exist").position[axis],
                        _ => parent_position.map(|p| p[axis]).unwrap_or(0.0)
                    };
                    let anchor_size = match widget.position[axis] {
                        Positioning::DeltaOfWidget(other_wid, _, _) => self.widget_states.get(&other_wid).expect("dependent wid must exist").dimensions[axis],
                        _ => parent_size.map(|p| p[axis]).unwrap_or(self.gui_size[axis])
                    };

                    let (alignment_point, inverter, dim_multiplier) = match widget.alignment[axis] {
                        Alignment::Left | Alignment::Top => (anchor_pos, 1.0, 0.0),
                        Alignment::Right | Alignment::Bottom => (anchor_pos + anchor_size, -1.0, -1.0)
                    };

                    let border_width = widget.border.width;
                    let border_width = self.pixel_dim_axis_to_gui(border_width as f32);
                    let margin = widget.margin.ux(pixels_per_ux);

                    let effective_dim = match widget.size[axis] {
                        Sizing::SurroundChildren => {
                            let mut enclosing_rect = None;
                            for child_wid in &internal_state.children {
                                let child_bounds = self.widget_states.get(child_wid).expect("child must exist").bounds();
                                enclosing_rect = match enclosing_rect {
                                    Some(existing) => Some(Rect::enclosing_both(existing, child_bounds)),
                                    None => Some(child_bounds)
                                };
                            }
                            trace!(target: "gui_redraw", "Computed enclosing rect of {} children : {:?} for axis {}", internal_state.children.len(), enclosing_rect, axis);
                            enclosing_rect.map(|r| r.dimensions()[axis]).unwrap_or(1.0) + border_width * 2.0 + margin * 2.0
                        }
                        Sizing::Derived => {
                            match &widget.widget_type {
                                WidgetType::Text { font, text, font_size, wrap: _, .. } => {
                                    let dims = g.string_dimensions_no_wrap(font.unwrap_or(DEFAULT_FONT_IDENTIFIER), text.as_str(), *font_size);
                                    let dim = self.pixel_dim_axis_to_gui(dims[axis]);
                                    trace!(target: "gui_redraw", "Calculated derived dim for text {} of {:?}", text.replace('\n', "\\n"), dim);
                                    dim
                                }
                                other => {
                                    trace!(target: "gui_redraw", "Widget had derived size, but non-derivable widget type {:?}", other);
                                    0.0
                                }
                            }
                        }
                        sizing => self.compute_raw_widget_size(sizing, anchor_size)
                    };

                    let effective_pos = match widget.position[axis] {
                        Positioning::Constant(constant) => alignment_point + constant.ux(pixels_per_ux) * inverter,
                        Positioning::PcntOfParent(pcnt) => alignment_point + (anchor_size * pcnt) * inverter,
                        Positioning::CenteredInParent => alignment_point + (anchor_size - effective_dim) * 0.5,
                        Positioning::DeltaOfWidget(other_wid, delta, anchor_alignment) => {
                            let alignment_point = match anchor_alignment {
                                Alignment::Right | Alignment::Bottom => anchor_pos + anchor_size,
                                _ => anchor_pos
                            };
                            alignment_point + delta.ux(pixels_per_ux) * inverter
                        }
                    } + effective_dim * dim_multiplier;

                    internal_state.dimensions[axis] = effective_dim;
                    internal_state.inner_dimensions[axis] = effective_dim - border_width * 2.0 - margin * 2.0;
                    internal_state.position[axis] = effective_pos;
                    internal_state.inner_position[axis] = effective_pos + border_width + margin;
                }
                let main_rect = Rect::new(internal_state.position.x, internal_state.position.y, internal_state.dimensions.x, internal_state.dimensions.y);
                let inner_rect = Rect::new(internal_state.inner_position.x, internal_state.inner_position.y, internal_state.inner_dimensions.x, internal_state.inner_dimensions.y);
                let bounding_box = parent_bounding_box.intersect(main_rect);
                if widget.dependent_on_children() && !secondary {
                    internal_state.bounding_box = Some(Rect::new(0.0, 0.0, 1000.0, 1000.0));
                    internal_state.inner_bounding_box = Some(Rect::new(0.0, 0.0, 1000.0, 1000.0));
                } else {
                    internal_state.bounding_box = bounding_box;
                    internal_state.inner_bounding_box = bounding_box.and_then(|bb| bb.intersect(inner_rect));
                }
            } else {
                internal_state.dimensions = v2(0.0, 0.0);
                internal_state.inner_dimensions = v2(0.0, 0.0);
                internal_state.position = v2(0.0, 0.0);
                internal_state.inner_position = v2(0.0, 0.0);
                internal_state.bounding_box = None;
                internal_state.inner_bounding_box = None;
            }

            widget.dependent_on_children()
        };

        self.widget_states.insert(wid, internal_state);

        child_dependent
    }

    pub fn update_widget_draw(&mut self, g: &mut GraphicsAssets, wid: Wid) {
        let mut internal_state = self.widget_states.remove(&wid).expect("every wid called to update must have existing internal state");

        {
            let widget = &internal_state.widget;
            let parent_position = widget.parent_id.map(|parent_id| self.widget_states.get(&parent_id).expect("parent must exist").inner_position);
            let parent_size = widget.parent_id.map(|parent_id| self.widget_states.get(&parent_id).expect("parent must exist").inner_dimensions);
            let pixels_per_ux = self.pixels_per_ux();

            if widget.showing {
                internal_state.draw_list.clear();
                let pixel_offset = self.gui_pos_to_pixel(internal_state.position);
                let inner_pixel_offset = self.gui_pos_to_pixel(internal_state.inner_position);
                let effective_dim = self.gui_dim_to_pixel(internal_state.dimensions);
                let effective_internal_dim = self.gui_dim_to_pixel(internal_state.inner_dimensions);
                trace!("Drawing at {:?}, {:?} with dimensions {:?}, {:?}", internal_state.position, pixel_offset, internal_state.dimensions, effective_dim);
                match widget.widget_type {
                    WidgetType::Text { font, ref text, font_size, wrap: _ } => {
                        internal_state.draw_list = internal_state.draw_list.add_text(
                            Text::new(text.clone(), font_size)
                                .color(widget.color)
                                .font(font.unwrap_or(DEFAULT_FONT_IDENTIFIER))
                                .centered(false, false)
                                .offset(inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                        );
                    }
                    WidgetType::Window { ref image } => {
                        let color_multiplier = match internal_state.widget_state {
                            WidgetState::Button { pressed } if pressed => Color::new(0.2, 0.2, 0.2, 1.0),
                            _ => Color::new(1.0, 1.0, 1.0, 1.0)
                        };

                        if widget.border.width > 0 {
                            internal_state.draw_list = internal_state.draw_list.add_quad(
                                Quad::new(String::from("ui/blank"), pixel_offset - v2(0.0, effective_dim.y))
                                    .color(widget.border.color)
                                    .size(effective_dim)
                            )
                        }
                        match image {
                            Some(image) => {
                                internal_state.draw_list = internal_state.draw_list.add_quad(
                                    Quad::new(image.clone(), inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                                        .color(widget.color * color_multiplier)
                                        .size(effective_internal_dim)
                                )
                            }
                            None =>
                                internal_state.draw_list = internal_state.draw_list.add_quad(
                                    Quad::new(String::from("ui/blank"), inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                                        .color(widget.color * color_multiplier)
                                        .size(effective_internal_dim)
                                )
                        }
                    }
                }
            } else {
                internal_state.draw_list.clear();
            }
        }

        self.widget_states.insert(wid, internal_state);
    }


    fn depth_of_widget(&self, wid: Wid) -> usize {
        let mut count = 0;
        let mut cur_wid = wid;
        loop {
            let parent = self.widget_state(cur_wid).widget.parent_id;
            if let Some(parent) = parent {
                cur_wid = parent;
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    pub fn recursive_update_widget(&mut self, g: &mut GraphicsAssets, wid: Wid, force_update: bool) {
        let widget_depth = self.depth_of_widget(wid);

        let should_update = force_update || self.modified_set.contains(&wid);
        let child_dependent = if should_update {
            trace!(target: "gui_redraw", "{}Entering update of wid: {}, {:?}", "\t".repeat(widget_depth), wid, self.widget_state(wid).widget.widget_type);
            self.update_widget_state(g, wid, false)
        } else {
            false
        };

        let children = self.widget_states.get(&wid).expect("recursive update widget must take valid wid with known state").children.clone();
        for child in &children {
            self.recursive_update_widget(g, *child, should_update);
        }

        if should_update && child_dependent {
            trace!(target: "gui_redraw", "{}Performing post-child update of wid: {}", "\t".repeat(widget_depth + 1), wid);
            self.update_widget_state(g, wid, true);
//            for child in &children {
//                self.recursive_update_widget(g, *child, should_update);
//            }
        }

        if should_update {
            trace!(target: "gui_redraw", "{}Performing draw update of wid: {}", "\t".repeat(widget_depth + 1), wid);
            self.update_widget_draw(g, wid);
        }

        if should_update {
            trace!(target: "gui_redraw", "{}Closing update of wid: {}", "\t".repeat(widget_depth), wid);
        }
    }

    pub fn render_draw_list(&self, g: &mut GraphicsWrapper, draw_list: &DrawList) {
        for quad in draw_list.quads.clone() {
            g.draw_quad(quad)
        }
        for text in draw_list.text.clone() {
            g.draw_text(text);
        }
    }

    pub fn recursive_draw_widget(&self, g: &mut GraphicsWrapper, wid: Wid) {
        let widget_state = self.widget_states.get(&wid).expect("recursive update widget must take valid wid with known state");

        self.render_draw_list(g, &widget_state.draw_list);

        let children = widget_state.children.clone();
        for child in children {
            self.recursive_draw_widget(g, child);
        }
    }

    pub fn gui_pos_to_pixel(&self, v: Vec2f) -> Vec2f {
        v2((v.x / self.gui_size.x) * self.viewport.window_size[0] as f32 - self.viewport.window_size[0] as f32 * 0.5,
           self.viewport.window_size[1] as f32 * 0.5 - (v.y / self.gui_size.y) * self.viewport.window_size[1] as f32)
    }
    pub fn pixel_pos_to_gui(&self, v: Vec2f) -> Vec2f {
        let ratio = self.ux_per_pixel();
        v2(v.x * ratio, v.y * ratio)
    }
    pub fn gui_dim_to_pixel(&self, v: Vec2f) -> Vec2f {
        v2((v.x / self.gui_size.x) * self.viewport.window_size[0] as f32, (v.y / self.gui_size.y) * self.viewport.window_size[1] as f32)
    }
    pub fn pixel_dim_to_gui(&self, v: Vec2f) -> Vec2f {
        v2((v.x / self.viewport.window_size[0] as f32) * self.gui_size.x, (v.y / self.viewport.window_size[1] as f32) * self.gui_size.y)
    }
    pub fn pixel_dim_axis_to_gui(&self, v: f32) -> f32 {
        (self.gui_size.x / self.viewport.window_size[0] as f32) * v
    }
    pub fn pixels_per_ux(&self) -> f32 {
        self.viewport.window_size[0] as f32 / self.gui_size.x
    }
    pub fn ux_per_pixel(&self) -> f32 {
        self.gui_size.x / self.viewport.window_size[0] as f32
    }

    pub fn reset_events(&mut self) {
        self.events_by_widget.clear();
    }

    pub fn update(&mut self, g: &mut GraphicsAssets, force_update: bool) {
        let top_level_widgets = self.top_level_widgets.clone();
        for wid in &top_level_widgets {
            self.recursive_update_widget(g, *wid, force_update);
        }
        self.modified_set.clear();
    }

    pub fn draw(&mut self, g: &mut GraphicsWrapper) {
        let size_changed = self.viewport.window_size != g.viewport.window_size;
        if size_changed {
            info!("size changed: {:?}", g.viewport.window_size);
            self.viewport = g.viewport.clone();
            let x_to_y_size_ratio = self.viewport.window_size[0] as f32 / self.viewport.window_size[1] as f32;
            self.gui_size.x = self.gui_size.y * x_to_y_size_ratio;
        }

        self.update(&mut g.resources.assets, size_changed);

        for wid in &self.top_level_widgets {
            self.recursive_draw_widget(g, *wid);
        }
    }


    fn childmost_widget_containing(&self, wid: Wid, v: Vec2f) -> Option<Wid> {
        let state = self.widget_state(wid);
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

    /*
    We need two things. We need to know whether each widget consumes the event or not, and we need to actually process the callbacks, if any. In theory, right now,
    these can accept arbitrary state in the callbacks, potentially multiple different kinds of state. In practice we probably only need... a few. The case of nested
    game modes might complicate that. I wouldn't really want to lock myself down to one. So I think we separate out the processing from the consumption calculation,
    that'll be the easiest thing.
    */

    fn widget_state(&self, wid: Wid) -> &WidgetInternal {
        self.widget_states.get(&wid).expect("Widget state must exist for wid, but none found")
    }


    fn handle_event_widget_intern<F: FnMut(u32, &UIEvent) -> ()>(&mut self, wid: Wid, event: &UIEvent, is_placeholder: bool, mut func: F) -> bool {
        let (parent_id, accepts_focus, consumed) = {
            if is_placeholder {
                self.events_by_widget.entry(wid).or_insert_with(|| Vec::new()).push(event.clone());

                let mut widget_state = self.widget_states.remove(&wid).expect("widget state must exist");

                if widget_state.widget.callbacks.non_empty() {
                    let callbacks = widget_state.widget.callbacks.clone();
                    let mut widget_context = WidgetContext { widget_state: widget_state.widget_state.clone(), triggered_events: Vec::new() };
                    for callback in callbacks {
                        execute_global_widget_registry_callback(callback, &mut widget_context, event);
                    }

                    if widget_state.widget_state != widget_context.widget_state {
                        debug!("Widget state changed from self event, new state is {:?}, marking wid {} for modified", widget_context.widget_state, wid);
                        self.modified_set.insert(wid);
                        self.queued_events.iter_mut()
                            .foreach(|(_, vec)| vec.push(
                                UIEvent::WidgetStateChanged { old_state: widget_state.widget_state.clone(), new_state: widget_context.widget_state.clone() }));
                        widget_state.widget_state = widget_context.widget_state.clone();
                    }

                    for evt in widget_context.triggered_events {
                        self.queued_events.iter_mut().foreach(|(_, vec)| vec.push(evt.clone()));
                    }
                }

                self.widget_states.insert(wid, widget_state);
            } else {
                let widget_state = self.widget_state(wid);
                if widget_state.widget.callbacks.non_empty() {
                    for callback in &widget_state.widget.callbacks {
                        trace!("Executing callback for widget {}", wid);
                        (func)(*callback, event);
                    }
                }
            }

            let widget_state = self.widget_state(wid);
            let consumed = if widget_state.widget.event_consumption.consumes_event(event) {
                trace!("Event consumed, returning");
                Some(true)
            } else if widget_state.widget.parent_id.is_none() {
                trace!("Event not consumed, but no parent, returning");
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
            consumed
        } else if let Some(parent_id) = parent_id {
            trace!("Event not consumed, parent present, passing event to parent");
            self.handle_event_widget_intern(parent_id, event, is_placeholder, func)
        } else {
            false
        }
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
//        let (parent_id, accepts_focus, consumed) = {
//            if TypeId::of::<State>() == TypeId::of::<WidgetStatePlaceholder>() {
//                self.events_by_widget.entry(wid).or_insert_with(|| Vec::new()).push(event.clone());
//
//                let mut widget_state = self.widget_states.remove(&wid).expect("widget state must exist");
//
//                if widget_state.widget.callbacks.non_empty() {
//                    let callbacks = widget_state.widget.callbacks.clone();
//                    let mut widget_context = WidgetContext { widget_state: widget_state.widget_state.clone(), triggered_events: Vec::new() };
//                    for callback in callbacks {
//                        execute_global_widget_registry_callback(callback, &mut widget_context, event);
//                    }
//
//                    if widget_state.widget_state != widget_context.widget_state {
//                        debug!("Widget state changed from self event, new state is {:?}, marking wid {} for modified", widget_context.widget_state, wid);
//                        self.modified_set.insert(wid);
//                        self.queued_events.iter_mut()
//                            .foreach(|(_, vec)| vec.push(
//                                UIEvent::WidgetStateChanged { old_state: widget_state.widget_state.clone(), new_state: widget_context.widget_state.clone() }));
//                        widget_state.widget_state = widget_context.widget_state.clone();
//                    }
//
//                    for evt in widget_context.triggered_events {
//                        self.queued_events.iter_mut().foreach(|(_, vec)| vec.push(evt.clone()));
//                    }
//                }
//
//                self.widget_states.insert(wid, widget_state);
//            } else {
//                let widget_state = self.widget_state(wid);
//                if widget_state.widget.callbacks.non_empty() {
//                    for callback in &widget_state.widget.callbacks {
//                        trace!("Executing callback for widget {}", wid);
//                        execute_global_widget_registry_callback(*callback, state, event);
//                    }
//                }
//            }
//
//            let widget_state = self.widget_state(wid);
//            let consumed = if widget_state.widget.event_consumption.consumes_event(event) {
//                trace!("Event consumed, returning");
//                Some(true)
//            } else if widget_state.widget.parent_id.is_none() {
//                trace!("Event not consumed, but no parent, returning");
//                Some(false)
//            } else {
//                None
//            };
//
//            (widget_state.widget.parent_id, widget_state.widget.accepts_focus, consumed)
//        };
//
//        if let UIEvent::MouseRelease { .. } = event {
//            if accepts_focus {
//                self.focused_widget = Some(wid);
//            } else {
//                self.focused_widget = None;
//            }
//        };
//
//        if let Some(consumed) = consumed {
//            consumed
//        } else if let Some(parent_id) = parent_id {
//            trace!("Event not consumed, parent present, passing event to parent");
//            self.handle_event_widget(parent_id, event, state)
//        } else {
//            false
//        }
    }

    pub fn handle_ui_event_for_self(&mut self, ui_event: &UIEvent) -> bool {
        self.handle_ui_event::<WidgetStatePlaceholder>(ui_event, &mut WidgetStatePlaceholder {})
    }

    pub fn handle_ui_event<'a, 'b, State: 'static>(&'a mut self, ui_event: &UIEvent, state: &'b mut State) -> bool {
        let type_id = TypeId::of::<State>();

        let mut queued_events = self.queued_events.remove(&type_id).unwrap_or_else(|| Vec::new());
        for queued_event in &queued_events {
            self.handle_ui_event(queued_event, state);
        }
        queued_events.clear();
        self.queued_events.insert(type_id, queued_events);

        self.update_mouse_pos(ui_event);

        if let Some(wid) = self.widget_to_route_to(ui_event) {
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
        }
    }

    pub fn handle_ui_event_2<State: 'static, OtherState: 'static>(&mut self, ui_event: &UIEvent, state: &mut State, other_state: &mut OtherState) -> bool {
        let type_id = TypeId::of::<(State, OtherState)>();

        let mut queued_events = self.queued_events.remove(&type_id).unwrap_or_else(|| Vec::new());
        for queued_event in &queued_events {
            self.handle_ui_event_2(queued_event, state, other_state);
        }
        queued_events.clear();
        self.queued_events.insert(type_id, queued_events);

        self.update_mouse_pos(ui_event);

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

    fn update_mouse_pos(&mut self, ui_event: &UIEvent) {
        self.current_mouse_pos = match ui_event {
            UIEvent::MousePosition { pos } => pos.absolute_pos,
            UIEvent::MouseMove { pos, .. } => pos.absolute_pos,
            UIEvent::MouseRelease { pos, .. } => pos.absolute_pos,
            UIEvent::MousePress { pos, .. } => pos.absolute_pos,
            UIEvent::Drag { pos, .. } => pos.absolute_pos,
            _ => self.current_mouse_pos
        };
    }

    fn widget_to_route_to(&self, ui_event: &UIEvent) -> Option<Wid> {
        use self::EventRouting::*;
        let routing = match ui_event {
            UIEvent::Text { .. } => FocusedWidget,
            UIEvent::KeyPress { .. } => FocusedWidget,
            UIEvent::KeyRelease { .. } => FocusedWidget,
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
            }
        }
    }

    pub fn convert_event<E: GenericEvent>(&mut self, event: E) -> Option<UIEvent> {
        UIEvent::from_piston_event(self.ux_per_pixel(), self.current_mouse_pos * self.pixels_per_ux(), event)
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
        self.events_by_widget.get(&widget.id).unwrap_or_else(|| &self.empty_vec)
    }
}

struct WidgetStatePlaceholder {}

enum EventRouting {
    FocusedWidget,
    MousedWidget,
}

pub struct WidgetContext {
    widget_state: WidgetState,
    triggered_events: Vec<UIEvent>,
}

impl WidgetContext {
    pub fn trigger_event(&mut self, event: UIEvent) {
        self.triggered_events.push(event);
    }
    pub fn update_state(&mut self, new_state: WidgetState) {
        self.widget_state = new_state;
    }
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
            .event_consumption(EventConsumption::all())
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
        assert_that(&gui.widget_state(widget.id).position).is_equal_to(&v2(10.0, 10.0));

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
            .event_consumption(EventConsumption::mouse_events())
            .with_callback(|state: &mut TestState, evt: &UIEvent| { state.i = 2; })
            .apply(&mut gui);

        // update and check that position is reasonable
        gui.update(&mut graphics_assets, false);
        assert_that(&gui.widget_state(sub_widget.id).position).is_equal_to(&v2(10.0 + 0.2 * 20.0, 10.0 + 0.2 * 20.0));

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
        assert_that(&gui.focused_widget).is_equal_to(&Some(sub_widget.id));
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
            .event_consumption(EventConsumption::all())
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