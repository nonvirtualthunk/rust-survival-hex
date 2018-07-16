use graphics::core::Quad;
use common::prelude::*;
use std::collections::HashMap;
use backtrace::Backtrace;
use graphics::GraphicsResources;
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

pub type WidIntern = usize;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct Wid(WidIntern);

pub static NO_WID: Wid = Wid(0);



#[derive(Clone, Copy, PartialEq, Neg)]
pub enum UIUnits {
    Pixels(i32),
    Units(f32)
}
impl UIUnits {
    pub fn px(&self, pixels_per_ux : f32) -> i32 {
        match self {
            UIUnits::Pixels(px) => *px,
            UIUnits::Units(ux) => (ux * pixels_per_ux).round() as i32
        }
    }
    pub fn ux(&self, pixels_per_ux : f32) -> f32 {
        match self {
            UIUnits::Pixels(px) => *px as f32 / pixels_per_ux,
            UIUnits::Units(ux) => *ux
        }
    }
}

pub trait ToGUIPixels {
    fn px (&self) -> UIUnits;
}
pub trait ToGUIUnit {
    fn ux (&self) -> UIUnits;
}
/*impl ToGUIPixels for i32 {
    fn px(&self) -> UIUnits {
        UIUnits::Pixels(*self)
    }
}*/
impl <T : ToPrimitive> ToGUIPixels for T {
    fn px(&self) -> UIUnits {
        UIUnits::Pixels(self.to_f32().unwrap().round() as i32)
    }
}
impl <T : ToPrimitive> ToGUIUnit for T {
    fn ux(&self) -> UIUnits {
        UIUnits::Units(self.to_f32().unwrap())
    }
}
//impl ToGUIUnit for f32 {
//    fn ux(&self) -> UIUnits {
//        UIUnits::Units(*self)
//    }
//}

#[derive(Clone, Copy, PartialEq)]
pub enum Sizing {
    PcntOfParent(f32),
    DeltaOfParent(UIUnits),
    Constant(UIUnits),
    Derived,
    SurroundChildren,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Positioning {
    PcntOfParent(f32),
    CenteredInParent,
    Constant(UIUnits),
    DeltaOfWidget(Wid, UIUnits, Alignment),
}

#[derive(Clone, Copy, Eq, PartialEq)]
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
    inner_dimensions: Vec2f
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
            inner_position: v2(0.0, 0.0)
        }
    }

    pub fn bounds(&self) -> Rect<f32> {
        Rect::new(self.position.x, self.position.y, self.dimensions.x, self.dimensions.y)
    }
}

//pub struct GUICore {
pub struct GUI {
    id_counter: WidIntern,
    widget_states: HashMap<Wid, WidgetInternal>,
    top_level_widgets: HashSet<Wid>,
    gui_size: Vec2f,
    viewport: Viewport,
    modified_set: HashSet<Wid>,
}

//pub struct GUI<'a, 'b> {
//    graphics_resources : &'a mut GraphicsResources,
//    core : &'b mut GUICore
//}
//
//impl <'a,'b> Deref for GUI<'a, 'b> {
//    type Target = GUICore;
//
//    fn deref(&self) -> &GUICore {
//        self.core
//    }
//}
//impl <'a,'b> DerefMut for GUI<'a, 'b> {
//    fn deref_mut(&mut self) -> &mut GUICore {
//        self.core
//    }
//}

//impl <'a,'b> GUI<'a,'b> {
impl GUI {
    pub fn new() -> GUI {
        GUI {
            id_counter: 0,
            widget_states: HashMap::new(),
            top_level_widgets: HashSet::new(),
            gui_size: v2(100.0, 100.0),
            viewport: Viewport { window_size : [256, 256], draw_size: [256, 256], rect : [0,0,256,256] },
            modified_set : HashSet::new()
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
                self.modified_set.insert(wid);
            }
            self.widget_states.insert(wid, state);
        } else {
            self.modified_set.insert(wid);
            self.widget_states.insert(wid, WidgetInternal::new(widget.clone()));
        }
    }


    pub fn compute_raw_widget_size(&self, size : Sizing, anchor_size : f32) -> f32 {
        let pixels_per_ux = self.pixels_per_ux();
        match size {
            Sizing::Constant(constant) => constant.ux(pixels_per_ux),
            Sizing::PcntOfParent(pcnt) => anchor_size * pcnt,
            Sizing::DeltaOfParent(delta) => anchor_size + delta.ux(pixels_per_ux),
            _ => 0.0
        }
    }

    pub fn update_widget_state(&mut self, g: &mut GraphicsWrapper, wid: Wid) -> bool {
        let mut internal_state = self.widget_states.remove(&wid).expect("every wid called to update must have existing internal state");

        let child_dependent = {
            let widget = &internal_state.widget;
            let parent_position = widget.parent_id.map(|parent_id| self.widget_states.get(&parent_id).expect("parent must exist").inner_position);
            let parent_size = widget.parent_id.map(|parent_id| self.widget_states.get(&parent_id).expect("parent must exist").inner_dimensions);
            let pixels_per_ux = self.pixels_per_ux();

            if widget.showing {
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

                    let border_width = match widget.widget_type {
                        WidgetType::Window { border_width, .. } => border_width,
                        _ => 0
                    };
                    let border_width = self.pixel_dim_axis_to_gui(border_width as f32);

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
                            println!("Looked at {} children, enclosing rect found to be {:?}", internal_state.children.len(), enclosing_rect);
                            enclosing_rect.map(|r| r.dimensions()[axis]).unwrap_or(1.0) + border_width * 2.0
                        },
                        Sizing::Derived => {
                            match &widget.widget_type {
                                WidgetType::Text { font, text, font_size, wrap: _, .. } => {
                                    let dims = g.resources.string_dimensions_no_wrap(font.unwrap_or(DEFAULT_FONT_IDENTIFIER), text.as_str(), *font_size);
                                    let dim = self.pixel_dim_axis_to_gui(dims[axis]);
                                    dim
                                }
                                _ => 0.0
                            }
                        },
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
                        },
                    } + effective_dim * dim_multiplier;

                    internal_state.dimensions[axis] = effective_dim;
                    internal_state.inner_dimensions[axis] = effective_dim - border_width * 2.0;
                    internal_state.position[axis] = effective_pos;
                    internal_state.inner_position[axis] = effective_pos + border_width;
                }
            } else {
                internal_state.dimensions = v2(0.0, 0.0);
                internal_state.inner_dimensions = v2(0.0, 0.0);
                internal_state.position = v2(0.0, 0.0);
                internal_state.inner_position = v2(0.0, 0.0);
            }

            widget.dependent_on_children()
        };

        self.widget_states.insert(wid, internal_state);

        child_dependent
    }

    pub fn update_widget_draw(&mut self, g: &mut GraphicsWrapper, wid: Wid) {
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
                println!("{:?}, {:?}       {:?}, {:?}", internal_state.position, pixel_offset, internal_state.dimensions, effective_dim);
                match widget.widget_type {
                    WidgetType::Text { font, ref text, font_size, wrap: _, color } => {
                        internal_state.draw_list = internal_state.draw_list.add_text(
                            Text::new(text.clone(), font_size)
                                .color(color)
                                .font(font.unwrap_or(DEFAULT_FONT_IDENTIFIER))
                                .centered(false, false)
                                .offset(inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                        );
                    }
                    WidgetType::Window { ref image, color, border_width, border_color, .. } => {
                        if border_width > 0 {
                            internal_state.draw_list = internal_state.draw_list.add_quad(
                                Quad::new(String::from("ui/blank"), pixel_offset - v2(0.0, effective_dim.y))
                                    .color(border_color)
                                    .size(effective_dim)
                            )
                        }
                        match image {
                            Some(image) => {
                                internal_state.draw_list = internal_state.draw_list.add_quad(
                                    Quad::new(image.clone(), inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                                        .color(color)
                                        .size(effective_internal_dim)
                                )
                            }
                            None =>
                                internal_state.draw_list = internal_state.draw_list.add_quad(
                                    Quad::new(String::from("ui/blank"), inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                                        .color(color)
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


    pub fn recursive_update_widget(&mut self, g: &mut GraphicsWrapper, wid: Wid, force_update : bool) {
        let should_update = force_update || self.modified_set.contains(&wid);
        let child_dependent = if should_update {
            self.update_widget_state(g, wid)
        } else {
            false
        };

        let children = self.widget_states.get(&wid).expect("recursive update widget must take valid wid with known state").children.clone();
        for child in &children {
            self.recursive_update_widget(g, *child, should_update);
        }

        if should_update && child_dependent {
            self.update_widget_state(g, wid);
//            for child in &children {
//                self.recursive_update_widget(g, *child, should_update);
//            }
        }

        if should_update {
            self.update_widget_draw(g, wid);
        }
    }

    pub fn render_draw_list(&self, g: &mut GraphicsWrapper, draw_list : &DrawList) {
        for quad in draw_list.quads.clone() {
            g.draw_quad(quad)
        }
        for text in draw_list.text.clone() {
            g.draw_text(text);
        }
    }

    pub fn recursive_draw_widget(&self, g: &mut GraphicsWrapper, wid : Wid) {
        let widget_state = self.widget_states.get(&wid).expect("recursive update widget must take valid wid with known state");

        self.render_draw_list(g, &widget_state.draw_list);

        let children = widget_state.children.clone();
        for child in children {
            self.recursive_draw_widget(g, child);
        }
    }

    fn gui_pos_to_pixel (&self, v : Vec2f) -> Vec2f {
        v2((v.x / self.gui_size.x) * self.viewport.window_size[0] as f32 - self.viewport.window_size[0] as f32 * 0.5,
           self.viewport.window_size[1] as f32 * 0.5 - (v.y / self.gui_size.y) * self.viewport.window_size[1] as f32)
    }
    fn gui_dim_to_pixel (&self, v : Vec2f) -> Vec2f {
        v2((v.x / self.gui_size.x) * self.viewport.window_size[0] as f32, (v.y / self.gui_size.y) * self.viewport.window_size[1] as f32)
    }
    fn pixel_dim_to_gui(&self, v : Vec2f) -> Vec2f {
        v2((v.x / self.viewport.window_size[0] as f32) * self.gui_size.x, (v.y / self.viewport.window_size[1] as f32) * self.gui_size.y)
    }
    fn pixel_dim_axis_to_gui(&self, v : f32) -> f32{
        (self.gui_size.x / self.viewport.window_size[0] as f32) * v
    }
    fn pixels_per_ux(&self) -> f32 {
        self.viewport.window_size[0] as f32 / self.gui_size.x
    }

    pub fn draw(&mut self, g: &mut GraphicsWrapper) {
        let size_changed = self.viewport.window_size != g.viewport.window_size;
        if size_changed {
            println!("size changed: {:?}", g.viewport.window_size);
            self.viewport = g.viewport.clone();
            let x_to_y_size_ratio = self.viewport.window_size[0] as f32 / self.viewport.window_size[1] as f32;
            self.gui_size.x = self.gui_size.y * x_to_y_size_ratio;
        }

        let top_level_widgets = self.top_level_widgets.clone();
        for wid in &top_level_widgets {
            self.recursive_update_widget(g, *wid, size_changed);
        }
        self.modified_set.clear();

        for wid in &top_level_widgets {
            self.recursive_draw_widget(g, *wid);
        }
    }
}

impl Default for GUI {
    fn default() -> Self {
        GUI::new()
    }
}