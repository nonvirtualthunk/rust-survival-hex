use anymap::AnyMap;
use backtrace::Backtrace;
use common::Color;
use common::prelude::*;
use common::Rect;
use events::EventPosition;
use events::UIEvent;
use graphics::core::Quad;
use graphics::DrawList;
use graphics::GraphicsAssets;
use graphics::GraphicsResources;
use graphics::GraphicsWrapper;
use graphics::Text;
use graphics::TextLayout;
use gui::GUI;
use widgets::Wid;
use gui::WidgetContext;
use gui::WidgetStatePlaceholder;
use piston_window;
use piston_window::GenericEvent;
use events::MouseButton;
use piston_window::Viewport;
use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;
use widgets::*;
use gui::UIUnits;
use gui::ToGUIUnit;
use widget_delegation::DelegateToWidget;
use std::time::Instant;


#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd)]
pub enum GUILayer {
    Overlay,
    Main,
}

impl GUI {
    pub fn compute_raw_widget_size(&self, size: Sizing, anchor_size: f32) -> f32 {
        let pixels_per_ux = self.pixels_per_ux();
        match size {
            Sizing::Constant(constant) => constant.ux(pixels_per_ux),
            Sizing::PcntOfParent(pcnt) => anchor_size * pcnt,
            Sizing::PcntOfParentAllowingLoop(pcnt) => anchor_size * pcnt,
            Sizing::DeltaOfParent(delta) => anchor_size + delta.ux(pixels_per_ux),
            _ => 0.0
        }
    }

    pub fn update_widget(&mut self, g: &mut GraphicsAssets, wid: Wid, secondary: bool) -> bool {
        let mut internal_state = self.widget_reifications.remove(&wid).expect("every wid called to update must have existing internal state");
        let child_dependent = internal_state.widget.dependent_on_children();
        if child_dependent && ! secondary { internal_state.update_in_progress = true; }

        let widget = &internal_state.widget;

        let (parent_position, parent_size, parent_bounding_box, parent_in_progress) = if let Some(parent_id) = widget.parent_id {
            if let Some(parent) = self.widget_reifications.get(&parent_id) {
                (Some(parent.inner_position), Some(parent.inner_dimensions), parent.inner_bounding_box, parent.update_in_progress)
            } else {
                warn!("Widget {} expected to have parent {}, but that parent was no present in the reifications", internal_state.widget.signifier(), parent_id);
                (None, None, Some(Rect::new(0.0, 0.0, self.gui_size.x, self.gui_size.y)), false)
            }
        } else {
            (None, None, Some(Rect::new(0.0, 0.0, self.gui_size.x, self.gui_size.y)), false)
        };
        let pixels_per_ux = self.pixels_per_ux();

        if (parent_bounding_box.is_some() || widget.ignores_parent_bounds) && widget.showing {
            let parent_bounding_box = parent_bounding_box.unwrap_or_else(|| Rect::new(0.0f32, 0.0f32, 100000.0f32, 100000.0f32));
            for axis in 0..2 {
                let anchor_pos = match widget.position[axis] {
                    Positioning::DeltaOfWidget(other_wid, _, _) => self.widget_reifications.get(&other_wid).expect("dependent wid must exist").position[axis],
                    Positioning::MatchWidget(other_wid) => self.widget_reification(other_wid).position[axis],
                    _ => parent_position.map(|p| p[axis]).unwrap_or(0.0)
                };
                let anchor_size = match widget.position[axis] {
                    Positioning::DeltaOfWidget(other_wid, _, _) => self.widget_reifications.get(&other_wid).expect("dependent wid must exist").dimensions[axis],
                    _ => parent_size.map(|p| p[axis]).unwrap_or(self.gui_size[axis])
                };

                let (alignment_point, inverter, dim_multiplier) = match widget.alignment[axis] {
                    Alignment::Left | Alignment::Top => (anchor_pos, 1.0, 0.0),
                    Alignment::Right | Alignment::Bottom => (anchor_pos + anchor_size, -1.0, -1.0)
                };

                let border_width = widget.border.width;
                let border_width = self.pixel_dim_axis_to_gui(border_width as f32);
                let border_width_near = if widget.border.sides.has_near_side_for_axis(axis) { border_width } else { 0.0 };
                let border_width_far = if widget.border.sides.has_far_side_for_axis(axis) { border_width } else { 0.0 };

                let margin = widget.margin.ux(pixels_per_ux);

                let effective_dim = match widget.size[axis] {
                    Sizing::SurroundChildren => {
                        // the children have to have been computed for this to work, so only do the computation the second time around
                        if secondary {
                            let mut enclosing_rect = None;
                            for child_wid in &internal_state.children {
                                let child_reif = self.widget_reifications.get(child_wid).expect("child must exist");

                                match child_reif.widget.size[axis] {
                                    Sizing::PcntOfParent(_) | Sizing::DeltaOfParent(_) =>
                                        warn!("Loop, parent is dependent on surrounding children, but children are sized relative to parent dim, widget: {:?}", child_reif.widget.signifier()),
                                    _ => ()
                                };

                                match child_reif.widget.size[axis] {
                                    Sizing::PcntOfParentAllowingLoop(_) | Sizing::ExtendToParentEdge => (), // ignore, looping parent based sizing is ignored
                                    _ => {
                                        if child_reif.widget.showing {
                                            let child_bounds = child_reif.bounds();
                                            enclosing_rect = match enclosing_rect {
                                                Some(existing) => Some(Rect::enclosing_both(existing, child_bounds)),
                                                None => Some(child_bounds)
                                            };
                                        }
                                    }
                                }
                            }
                            let comparison_pos = internal_state.inner_position[axis];
                            trace!(target: "gui_redraw", "Computed enclosing rect of {} children : {:?} for axis {} with comp-pos: {}", internal_state.children.len(), enclosing_rect, axis, comparison_pos);
                            enclosing_rect.map(|r| r.max()[axis] - comparison_pos).unwrap_or(1.0) + border_width_near + border_width_far + margin * 2.0
                        } else {
                            0.0
                        }
                    }
                    Sizing::Derived => {
                        match &widget.widget_type {
                            WidgetType::Text { font, text, font_size, wrap, .. } => {
                                let wrap_dist = if let Some(wrap) = wrap {
                                    match wrap {
                                        TextWrap::WithinParent => parent_size.map(|p| p[0]).unwrap_or(self.gui_size[0]) * pixels_per_ux - (match widget.position[0] {
                                            Positioning::Constant(constant) => constant.px(pixels_per_ux) as f32,
                                            _ => 0.0
                                        }),
                                        TextWrap::ToMaximumOf(units) => units.px(pixels_per_ux) as f32
                                    }
                                } else {
                                    100000000.0
                                };
                                let dims = g.string_dimensions(font.unwrap_or(g.default_font), text.as_str(), *font_size, wrap_dist);
                                let dim = self.pixel_dim_axis_to_gui(dims[axis]);
                                trace!(target: "gui_redraw", "Calculated derived dim for text[size {:?}] {} of {:?}", *font_size, text.replace('\n', "\\n"), dim);
                                dim
                            },
                            WidgetType::Image { image } => {
                                let img = g.image(image.clone());
                                if axis == 0 {
                                    self.pixel_dim_axis_to_gui(img.width() as f32)
                                } else {
                                    self.pixel_dim_axis_to_gui(img.height() as f32)
                                }
                            },
                            other => {
                                trace!(target: "gui_redraw", "Widget had derived size, but non-derivable widget type {:?}", other);
                                0.0
                            }
                        }
                    }
                    Sizing::ExtendToParentEdge => {
                        0.0
                    }
                    sizing => self.compute_raw_widget_size(sizing, anchor_size)
                };

                let effective_pos = match widget.position[axis] {
                    Positioning::Constant(constant) => alignment_point + constant.ux(pixels_per_ux) * inverter,
                    Positioning::PcntOfParent(pcnt) => alignment_point + (anchor_size * pcnt) * inverter,
                    Positioning::CenteredInParent => if parent_in_progress { alignment_point } else { alignment_point + (anchor_size - effective_dim) * 0.5 },
                    Positioning::DeltaOfWidget(other_wid, delta, anchor_alignment) => {
                        match anchor_alignment {
                            Alignment::Right | Alignment::Bottom => anchor_pos + anchor_size + delta.ux(pixels_per_ux),
                            _ => anchor_pos - effective_dim - delta.ux(pixels_per_ux)
                        }
                    }
                    Positioning::MatchWidget(other_wid) => alignment_point,
                    Positioning::Absolute(absolute_position) => absolute_position.ux(pixels_per_ux),
                } + effective_dim * dim_multiplier;
                trace!(target: "gui_redraw_quads", "effective_pos {:?}, effective_dim {:?}, dim_multiplier {:?}", effective_pos, effective_dim, dim_multiplier);

                let effective_dim = if Sizing::ExtendToParentEdge == widget.size[axis] {
                    // TODO: support non-left/top aligned
                    if widget.alignment[axis] != Alignment::Top && widget.alignment[axis] != Alignment::Left {
                        error!("extend to parent not currently supported for non top/left aligned widgets");
                        effective_dim
                    } else {
                        let far_parent = parent_position.map(|p| p[axis]).unwrap_or(0.0) + parent_size.map(|p| p[axis]).unwrap_or(self.gui_size[axis]);
                        let new_dim = far_parent - effective_pos;
                        new_dim
                    }
                } else {
                    effective_dim
                };

                internal_state.dimensions[axis] = effective_dim;
                internal_state.inner_dimensions[axis] = effective_dim - border_width_near - border_width_far - margin * 2.0;
                internal_state.position[axis] = effective_pos;
                internal_state.inner_position[axis] = effective_pos + border_width_near + margin;
            }
            let main_rect = Rect::new(internal_state.position.x, internal_state.position.y, internal_state.dimensions.x, internal_state.dimensions.y);
            let inner_rect = Rect::new(internal_state.inner_position.x, internal_state.inner_position.y, internal_state.inner_dimensions.x, internal_state.inner_dimensions.y);
            if widget.ignores_parent_bounds {
                internal_state.bounding_box = Some(main_rect);
                internal_state.inner_bounding_box = Some(inner_rect);
            } else {
                let bounding_box = parent_bounding_box.intersect(main_rect);

                if widget.dependent_on_children() && !secondary {
                    internal_state.bounding_box = Some(Rect::new(0.0, 0.0, 1000.0, 1000.0));
                    internal_state.inner_bounding_box = Some(Rect::new(0.0, 0.0, 1000.0, 1000.0));
                } else {
                    internal_state.bounding_box = bounding_box;
                    internal_state.inner_bounding_box = bounding_box.and_then(|bb| bb.intersect(inner_rect));
                }
            }
        } else {
            internal_state.dimensions = v2(0.0, 0.0);
            internal_state.inner_dimensions = v2(0.0, 0.0);
            internal_state.position = v2(0.0, 0.0);
            internal_state.inner_position = v2(0.0, 0.0);
            internal_state.bounding_box = None;
            internal_state.inner_bounding_box = None;
        }

        if secondary { internal_state.update_in_progress = false; }

        self.widget_reifications.insert(wid, internal_state);

        child_dependent
    }

    pub fn update_widget_draw(&mut self, g: &mut GraphicsAssets, wid: Wid) {
        let mut internal_state = self.widget_reifications.remove(&wid).expect("every wid called to update must have existing internal state");

        {
            let widget = &internal_state.widget;
            let parent_position = widget.parent_id.map(|parent_id| self.widget_reifications.get(&parent_id).expect("parent must exist").inner_position);
            let parent_size = widget.parent_id.map(|parent_id| self.widget_reifications.get(&parent_id).expect("parent must exist").inner_dimensions);
            let pixels_per_ux = self.pixels_per_ux();

            if widget.showing {
                internal_state.draw_list.clear();
                let margin_ux = widget.margin.ux(pixels_per_ux);
                let pixel_offset = self.gui_pos_to_pixel(internal_state.position);
                let inner_pixel_offset = self.gui_pos_to_pixel(internal_state.inner_position - v2(margin_ux, margin_ux)); // - v2(margin_px, -margin_px);
                let effective_dim = self.gui_dim_to_pixel(internal_state.dimensions);
                let effective_internal_dim = self.gui_dim_to_pixel(internal_state.inner_dimensions + v2(margin_ux * 2.0, margin_ux * 2.0)); // + v2(margin_px * 2.0, margin_px * 2.0);
                trace!(target: "gui_redraw_quads", "Drawing at {:?}, {:?} with dimensions {:?}, {:?}", internal_state.position, pixel_offset, internal_state.dimensions, effective_dim);

                if effective_dim.x > 0.0 && effective_dim.y > 0.0 {
                    match widget.widget_type {
                        WidgetType::Text { font, ref text, font_size, ref wrap } => {
                            let wrap_dist = if let Some(wrap) = wrap {
                                match wrap {
                                    TextWrap::WithinParent => parent_size.map(|p| p[0]).unwrap_or(self.gui_size[0]) * pixels_per_ux - (match widget.position[0] {
                                        Positioning::Constant(constant) => constant.px(pixels_per_ux) as f32,
                                        _ => 0.0
                                    }),
                                    TextWrap::ToMaximumOf(units) => units.px(pixels_per_ux) as f32
                                }
                            } else {
                                100000000.0
                            };

                            internal_state.draw_list = internal_state.draw_list.add_text(
                                Text::new(text.clone(), font_size)
                                    .color(widget.color)
                                    .font(font.unwrap_or(g.default_font))
                                    .centered(false, false)
                                    .offset(inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                                    .wrap_to(wrap_dist)
                            );
                        },
                        WidgetType::Image { ref image } => {
                            internal_state.draw_list.add_quad(
                                Quad::new(image.clone(), inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                                    .color(widget.color)
                                    .size(effective_internal_dim)
                            );
                        },
                        WidgetType::Window { ref image, ref segment } => {
                            let color_multiplier = match internal_state.widget_state {
                                WidgetState::Button { pressed } if pressed => Color::new(0.5, 0.5, 0.54, 1.0),
                                _ => Color::new(1.0, 1.0, 1.0, 1.0)
                            };

                            if widget.border.width > 0 {
                                let border_start = pixel_offset - v2(0.0, effective_dim.y);
                                let border_width = widget.border.width;

                                if widget.border.sides.has_side(Alignment::Left) {
                                    internal_state.draw_list = internal_state.draw_list.with_quad(
                                        Quad::new(String::from("ui/blank"), border_start)
                                            .color(widget.border.color)
                                            .size(v2(border_width as f32, effective_dim.y))
                                    )
                                }
                                if widget.border.sides.has_side(Alignment::Right) {
                                    internal_state.draw_list = internal_state.draw_list.with_quad(
                                        Quad::new(String::from("ui/blank"), border_start + v2(effective_dim.x - border_width as f32, 0.0))
                                            .color(widget.border.color)
                                            .size(v2(border_width as f32, effective_dim.y))
                                    )
                                }
                                if widget.border.sides.has_side(Alignment::Top) {
                                    internal_state.draw_list = internal_state.draw_list.with_quad(
                                        Quad::new(String::from("ui/blank"), border_start + v2(0.0, effective_dim.y - border_width as f32))
                                            .color(widget.border.color)
                                            .size(v2(effective_dim.x, border_width as f32))
                                    )
                                }
                                if widget.border.sides.has_side(Alignment::Bottom) {
                                    internal_state.draw_list = internal_state.draw_list.with_quad(
                                        Quad::new(String::from("ui/blank"), border_start)
                                            .color(widget.border.color)
                                            .size(v2(effective_dim.x, border_width as f32))
                                    )
                                }
                            }
                            if widget.color.a() > 0.0 {
                                match image {
                                    Some(image) => {
                                        match segment {
                                            ImageSegmentation::None => {
                                                internal_state.draw_list = internal_state.draw_list.with_quad(
                                                    Quad::new(image.clone(), inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                                                        .color(widget.color * color_multiplier)
                                                        .size(effective_internal_dim)
                                                )
                                            }
                                            ImageSegmentation::Horizontal => {
                                                let start = inner_pixel_offset - v2(0.0, effective_internal_dim.y);
                                                let dim = effective_internal_dim;
                                                let endcap_size = dim.y * 0.5;
                                                let middle_size = dim.x - endcap_size * 2.0;
                                                if middle_size > 0.0 {
                                                    internal_state.draw_list = internal_state.draw_list.with_quad(
                                                        Quad::new(image.clone(), start)
                                                            .color(widget.color * color_multiplier)
                                                            .sub_rect(Rect::new(0.0, 0.0, 0.5, 1.0))
                                                            .size(v2(endcap_size, dim.y))
                                                    ).with_quad(
                                                        Quad::new(image.clone(), start + v2(endcap_size + middle_size, 0.0))
                                                            .color(widget.color * color_multiplier)
                                                            .sub_rect(Rect::new(0.5, 0.0, -0.5, 1.0))
                                                            .size(v2(endcap_size, dim.y))
                                                    ).with_quad(
                                                        Quad::new(image.clone(), start + v2(endcap_size, 0.0))
                                                            .color(widget.color * color_multiplier)
                                                            .sub_rect(Rect::new(0.5, 0.0, 0.5, 1.0))
                                                            .size(v2(middle_size, dim.y))
                                                    );
                                                } else {
                                                    let endcap_size = dim.x * 0.5;
                                                    internal_state.draw_list = internal_state.draw_list.with_quad(
                                                        Quad::new(image.clone(), start)
                                                            .color(widget.color * color_multiplier)
                                                            .sub_rect(Rect::new(0.0, 0.0, 0.5, 1.0))
                                                            .size(v2(endcap_size, dim.y))
                                                    ).with_quad(
                                                        Quad::new(image.clone(), start + v2(endcap_size, 0.0))
                                                            .color(widget.color * color_multiplier)
                                                            .sub_rect(Rect::new(0.5, 0.0, -0.5, 1.0))
                                                            .size(v2(endcap_size, dim.y))
                                                    );
                                                }
                                            }
                                            _ => warn!("unimplemented segmentation {:?}", segment)
                                        };
                                    }
                                    None => {
                                        trace!(target: "gui_redraw_quads", "\tAdding no image quad at {:?} with dimensions {:?}", inner_pixel_offset - v2(0.0, effective_internal_dim.y), effective_internal_dim);
                                        internal_state.draw_list = internal_state.draw_list.with_quad(
                                            Quad::new(String::from("ui/blank"), inner_pixel_offset - v2(0.0, effective_internal_dim.y))
                                                .color(widget.color * color_multiplier)
                                                .size(effective_internal_dim)
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                internal_state.draw_list.clear();
            }
        }

        self.widget_reifications.insert(wid, internal_state);
    }


    fn depth_of_widget(&self, wid: Wid) -> usize {
        let mut count = 0;
        let mut cur_wid = wid;
        loop {
            let parent = self.widget_reification(cur_wid).widget.parent_id;
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
        let signifier = self.widget_reification(wid).widget.signifier();

        let should_update = force_update || self.modified_set.contains(&wid);
        let skip_update = !should_update && !self.widget_reification(wid).widget.showing;
//        if should_update {
//            debug!("Widget {} changed sufficiently to require update", signifier);
//        }

        if !skip_update {
            let child_dependent = if should_update {
                trace!(target: "gui_redraw", "{}Entering update of widget: {}, {:?}", "\t".repeat(widget_depth), signifier, self.widget_reification(wid).widget.widget_type);
                self.update_widget(g, wid, false)
            } else {
                false
            };

            let children = self.widget_reifications.get(&wid).expect("recursive update widget must take valid wid with known state").children.clone();
            for child in &children {
                self.recursive_update_widget(g, *child, should_update);
            }

            if should_update && child_dependent {
                trace!(target: "gui_redraw", "{}Performing post-child update of wid: {}", "\t".repeat(widget_depth + 1), signifier);
                self.update_widget(g, wid, true);
                for child in &children {
                    self.recursive_update_widget(g, *child, should_update);
                }
            }

            if should_update {
                trace!(target: "gui_redraw", "{}Performing draw update of wid: {}", "\t".repeat(widget_depth + 1), signifier);
                self.update_widget_draw(g, wid);
            }

            if should_update {
                trace!(target: "gui_redraw", "{}Closing update of wid: {}", "\t".repeat(widget_depth), signifier);
            }
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

    pub fn recursive_draw_widget(&self, g: &mut GraphicsWrapper, wid: Wid, layer: GUILayer) {
        let widget_state = self.widget_reifications.get(&wid).expect("recursive update widget must take valid wid with known state");
        if widget_state.widget.draw_layer == layer {
            self.render_draw_list(g, &widget_state.draw_list);
        }

        let children = &widget_state.children;
        for child in children {
            self.recursive_draw_widget(g, *child, layer);
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


    pub fn update(&mut self, g: &mut GraphicsAssets, force_update: bool) {
        if self.hover_widget == None && Instant::now().duration_since(self.hover_start) > self.hover_threshold {
            self.hover_widget = self.moused_over_widget;
            if let Some(hover_widget) = self.hover_widget {
                let pixels_per_ux = self.pixels_per_ux();
                let mouse_pos = self.current_mouse_pos.clone();

                let evt = UIEvent::HoverStart { over_widget: hover_widget, pos: EventPosition::absolute(mouse_pos, mouse_pos * pixels_per_ux) };
                self.handle_ui_event_for_self(&evt);
                self.enqueue_event_excepting_self(&evt);
            }
        }

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
            self.recursive_draw_widget(g, *wid, GUILayer::Main);
        }

        for wid in &self.top_level_widgets {
            self.recursive_draw_widget(g, *wid, GUILayer::Overlay);
        }
    }
}