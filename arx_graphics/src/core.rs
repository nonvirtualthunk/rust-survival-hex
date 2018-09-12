use camera::*;
use common::color::*;
use common::hex::CartVec;
use common::prelude::*;
use common::Rect;
use find_folder;
use gfx;
use gfx_device_gl;
use itertools::Itertools;
use piston_window::*;
use piston_window::math;
use piston_window::TextureSettings;
use piston_window::types;
use rusttype as rt;
use rusttype::gpu_cache::Cache as RTCache;
use rusttype::gpu_cache::CacheBuilder as RTCacheBuilder;
use std;
use std::collections::HashMap;
use std::path::PathBuf;
use text::TextLayout;

pub use resources::*;
use graphics::types::SourceRectangle;
use graphics::types::Rectangle as GraphicsRectangle;
use texture_atlas::StoredTextureInfo;

#[derive(Clone)]
pub struct Quad {
    pub image: Image,
    pub texture_identifier: ImageIdentifier,
    pub offset: Vec2f,
    pub rotation: f32,
    pub centered: bool,
    pub size: Option<Vec2f>,
    pub sub_rect: Option<Rect<f32>>
}

impl Quad {
    pub fn new(texture_identifier: ImageIdentifier, offset: Vec2f) -> Quad {
        Quad {
            texture_identifier,
            offset,
            rotation: 0.0,
            image: Image::new(),
            centered: false,
            size: None,
            sub_rect: None
        }
    }

    pub fn new_cart(texture_identifier: ImageIdentifier, offset: CartVec) -> Quad {
        Quad::new(texture_identifier, offset.0)
    }

    pub fn color(mut self, color: Color) -> Self {
        self.image = self.image.color(color.0);
        self
    }
    pub fn offset(mut self, offset: Vec2f) -> Self {
        self.offset = offset;
        self
    }
    //    pub fn hex_pos(mut self, offset : AxialCoord, ) -> Self {
//        self.offset = offset.
//    }
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
    pub fn size(mut self, size: Vec2f) -> Self {
        self.size = Some(size);
        self
    }
    pub fn sub_rect(mut self, rect : Rect<f32>) -> Self {
        self.sub_rect = Some(rect);
        self
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub enum FontSize {
    Small,
    Standard,
    HeadingMinor,
    HeadingMajor,
    Large,
    ExtraLarge,
    Points(u32),
    BasePlusPoints(u8, u16)
}
impl FontSize {

    pub fn font_size_by_ordinal(ordinal : u8) -> FontSize {
        use FontSize::*;
        match ordinal {
            0 => Small,
            1 => Standard,
            2 => HeadingMinor,
            3 => HeadingMajor,
            4 => Large,
            5 => ExtraLarge,
            _ => {
                warn!("Invalid ordinal provided for font size {}", ordinal);
                FontSize::Standard
            },
        }
    }

    pub fn ordinal(&self) -> u8 {
        use FontSize::*;
        match self {
            Small => 0,
            Standard => 1,
            HeadingMinor => 2,
            HeadingMajor => 3,
            Large => 4,
            ExtraLarge => 5,
            _ => {
                warn!("Only named font sizes can have an ordinal");
                1
            }
        }
    }

    pub fn default_point_size(&self) -> u32 {
        match self {
            FontSize::Small => 11,
            FontSize::Standard => 12,
            FontSize::HeadingMinor => 14,
            FontSize::HeadingMajor => 16,
            FontSize::Large => 20,
            FontSize::ExtraLarge => 26,
            FontSize::Points(pts) => *pts,
            FontSize::BasePlusPoints(base, pts) => FontSize::font_size_by_ordinal(*base).default_point_size() + (*pts as u32),
        }
    }

    pub fn next_larger(&self) -> FontSize {
        match self {
            FontSize::Small => FontSize::Standard,
            FontSize::Standard => FontSize::HeadingMinor,
            FontSize::HeadingMinor => FontSize::HeadingMajor,
            FontSize::HeadingMajor => FontSize::Large,
            FontSize::Large => FontSize::ExtraLarge,
            FontSize::ExtraLarge => FontSize::ExtraLarge,
            FontSize::Points(pts) => FontSize::Points(*pts + 2),
            FontSize::BasePlusPoints(base, pts) => FontSize::BasePlusPoints(*base, *pts + 2)
        }
    }

    pub fn plus_points(&self, plus_pts: u32) -> FontSize {
        match self {
            FontSize::BasePlusPoints(base, pts) => FontSize::BasePlusPoints(*base, (*pts as u32 + plus_pts) as u16),
            FontSize::Points(pts) => FontSize::Points(*pts + plus_pts),
            other => FontSize::BasePlusPoints(other.ordinal(), plus_pts as u16)
        }
    }
}

pub struct Text {
    pub text: String,
    pub font_identifier: Option<FontIdentifier>,
    pub offset: Vec2f,
    pub size: FontSize,
    pub color: Color,
    pub outline_color: Option<Color>,
    pub rounded: bool,
    pub centered_y: bool,
    pub centered_x: bool,
    pub wrap_to: f32
}

impl Clone for Text {
    fn clone(&self) -> Self {
        Text {
            text: self.text.clone(),
            font_identifier: self.font_identifier,
            offset: self.offset.clone(),
            size: self.size.clone(),
            color: self.color.clone(),
            outline_color : self.outline_color.clone(),
            rounded: self.rounded,
            centered_y: self.centered_y,
            centered_x: self.centered_x,
            wrap_to: self.wrap_to
        }
    }
}

impl Text {
    pub fn new(text: String, size: FontSize) -> Text {
        Text {
            text,
            size,
            font_identifier: None,
            offset: v2(0.0f32, 0.0f32),
            color: Color([0.0, 0.0, 0.0, 1.0]),
            rounded: true,
            centered_y: false,
            centered_x: true,
            wrap_to: 100000000.0,
            outline_color: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn outline_color(mut self, color : Option<Color>) -> Self {
        self.outline_color = color;
        self
    }

    pub fn colord(mut self, color: [f64; 4]) -> Self {
        self.color = Color([color[0] as f32, color[1] as f32, color[2] as f32, color[3] as f32]);
        self
    }

    pub fn offset(mut self, offset: Vec2f) -> Self {
        self.offset = offset;
        self
    }

    pub fn font(mut self, font_identifier: FontIdentifier) -> Self {
        self.font_identifier = Some(font_identifier);
        self
    }

    pub fn centered(mut self, x: bool, y: bool) -> Self {
        self.centered_x = x;
        self.centered_y = y;
        self
    }

    pub fn wrap_to(mut self, wrap_to : f32) -> Self {
        self.wrap_to = wrap_to;
        self
    }
}

#[derive(Default, Clone)]
pub struct DrawList {
    pub quads: Vec<Quad>,
    pub text: Vec<Text>,
}

impl DrawList {
    pub fn of_text(text: Text) -> DrawList {
        DrawList {
            quads: vec![],
            text: vec![text],
        }
    }
    pub fn of_quad(quad: Quad) -> DrawList {
        DrawList {
            quads: vec![quad],
            text: vec![],
        }
    }
    pub fn none() -> DrawList {
        DrawList {
            quads: vec![],
            text: vec![],
        }
    }

    pub fn add_quad(&mut self, quad: Quad) -> &mut Self {
        self.quads.push(quad);
        self
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.quads.push(quad);
        self
    }

    pub fn add_text(&mut self, text: Text) -> &mut Self {
        self.text.push(text);
        self
    }

    pub fn append(&mut self, other: &mut DrawList) {
        self.quads.append(&mut other.quads);
        self.text.append(&mut other.text);
    }

    pub fn extend(&mut self, other: DrawList) {
        self.quads.extend(other.quads);
        self.text.extend(other.text);
    }

    pub fn clear(&mut self) {
        self.quads.clear();
        self.text.clear();
    }
}

pub struct GraphicsWrapper<'a, 'b : 'a> {
    pub resources: &'a mut GraphicsResources,
    pub graphics: &'a mut G2d<'b>,
    pub context: Context,
    pub draw_state: DrawState,
    pub viewport: Viewport
}

impl<'a, 'b : 'a> GraphicsWrapper<'a, 'b> {
    pub fn new(context: Context, resources: &'a mut GraphicsResources, graphics: &'a mut G2d<'b>) -> GraphicsWrapper<'a, 'b> {
        GraphicsWrapper {
            context,
            resources,
            graphics,
            draw_state: DrawState::default(),
            viewport: context.viewport.unwrap_or_else(|| Viewport {
                window_size: [256, 256],
                draw_size: [256, 256],
                rect: [0, 0, 256, 256],
            })
        }
    }

    pub fn draw_quad(&mut self, quad: Quad) {
        let tex_info = self.resources.texture(String::from(quad.texture_identifier));
        let (w, h) = if let Some(size) = quad.size {
            (size.x as f64, size.y as f64)
        } else {
            (tex_info.size.x as f64, tex_info.size.y as f64)
        };

        let image = if quad.centered {
            quad.image.rect([-w / 2.0, -h / 2.0, w, h])
        } else {
            quad.image.rect([0.0, 0.0, w, h])
        }.maybe_src_rect(quad.sub_rect.map(|r| [r.x as f64 * tex_info.size.x as f64, r.y as f64 * tex_info.size.y as f64, r.w as f64 * tex_info.size.x as f64, r.h as f64 * tex_info.size.y as f64]));

        let pos = as_f64(quad.offset);
        let transform = math::multiply(self.context.view, math::translate(pos));
        let transform = if quad.rotation != 0.0 {
            math::multiply(transform, math::rotate_radians(quad.rotation as f64))
        } else {
            transform
        };
        image.draw(&tex_info.texture, &self.draw_state, transform, self.graphics);
    }

    pub fn draw_quads(&mut self, quads: &Vec<Quad>, atlas_identifier : TextureAtlasIdentifier) {
        let white = [1.0f32,1.0f32,1.0f32,1.0f32];
        let mut color = [1.0f32,1.0f32,1.0f32,1.0f32];
        let mut rects : Vec<(GraphicsRectangle, SourceRectangle)> = Vec::new();

        let transform = self.context.view.clone();

        for quad in quads {
            // rotation is stupid and their api is terrible, so.
            if quad.rotation != 0.0 {
                self.draw_quad(quad.clone());
                continue;
            }

            let (rect, source_rect) = {
                let tex_info = self.resources.texture_from_atlas(quad.texture_identifier.clone(), atlas_identifier);
                let (w, h) = if let Some(size) = quad.size {
                    (size.x as f32, size.y as f32)
                } else {
                    (tex_info.pixel_rect.width() as f32, tex_info.pixel_rect.height() as f32)
                };

                let rect = if quad.centered {
                    [(quad.offset.x - w / 2.0) as f64, (quad.offset.y - h / 2.0) as f64, w as f64, h as f64]
                } else {
                    [quad.offset.x as f64, quad.offset.y as f64, w as f64, h as f64]
                };

                let tx = tex_info.pixel_rect.x as f32;
                let ty = tex_info.pixel_rect.y as f32;
                let tw = tex_info.pixel_rect.w as f32;
                let th = tex_info.pixel_rect.h as f32;
                let source_rect = if let Some(sub_rect) = quad.sub_rect {
                     [(tx + sub_rect.x * tw) as f64, (ty + sub_rect.y * th) as f64, (tw * sub_rect.w) as f64, (th * sub_rect.h) as f64]
                } else {
                    [tx as f64, ty as f64, tw as f64, th as f64]
                };

                (rect, source_rect)
            };

            let eff_color = quad.image.color.unwrap_or(white);
            if color != eff_color {
                if rects.non_empty() {
                    self.resources.upload_atlases(self.graphics);
                    let atlas = self.resources.atlas_texture(atlas_identifier);
                    image::draw_many(rects.as_slice(), color, atlas, &self.draw_state, transform, self.graphics);
                    rects.clear();
                }
                color = eff_color;
            }

            rects.push((rect, source_rect));
        }
        if rects.non_empty() {
            self.resources.upload_atlases(self.graphics);
            let atlas = self.resources.atlas_texture(atlas_identifier);
            image::draw_many(rects.as_slice(), color, atlas, &self.draw_state, transform, self.graphics);
            rects.clear();
        }
    }

    fn dpi_scale(&self) -> f32 {
        self.viewport.draw_size[0] as f32 / self.viewport.window_size[0] as f32
    }


    pub fn draw_text(&mut self, text : Text) {
        let cache_id = 1;

        let dpi_scale = self.dpi_scale();
        let layout = self.resources.layout_text(&text);
        let layout_dims = layout.dimensions();
        let glyphs = layout.glyphs;

        let glyph_cache = &mut self.resources.glyph_cache;
        for glyph in &glyphs {
            glyph_cache.queue_glyph(cache_id, glyph.clone())
        }

        {
            let encoder = &mut self.graphics.encoder;
            let font_texture_data = &mut self.resources.font_texture_data;
            let font_texture = &mut self.resources.font_texture;

            glyph_cache.cache_queued(|rect, data| {
                font_texture_data.clear();
                font_texture_data.extend(data.iter().flat_map(|&b| vec![255,255,255,b]));
                let offset = [rect.min.x, rect.min.y];
                let size = [rect.width(), rect.height()];
                texture::UpdateTexture::update(font_texture, encoder, texture::Format::Rgba8, &font_texture_data[..], offset, size)
                                    .expect("Failed to update texture");
            }).expect("Could not update glyph cache");
        }

        let dpi_factor = dpi_scale;
        let (tex_w, tex_h) = self.resources.font_texture.get_size();

        let offset_x = if text.centered_x { layout_dims.x as f64 * -0.5 } else { 0.0 };
        let offset_y = if text.centered_y { layout_dims.y as f64 * -0.5 } else { 0.0 };

        let rectangles = glyphs.into_iter()
            .filter_map(|g| glyph_cache.rect_for(cache_id, &g).ok().unwrap_or(None))
            .map(|(uv_rect, screen_rect)| {
                let rectangle = {
                    let div_dpi_factor = |s| (s as f32 / dpi_factor as f32) as f64;
                    let left = div_dpi_factor(screen_rect.min.x) + offset_x;
                    let top = div_dpi_factor(screen_rect.min.y) + offset_y;
                    let right = div_dpi_factor(screen_rect.max.x) + offset_x;
                    let bottom = div_dpi_factor(screen_rect.max.y) + offset_y;
                    let w = right - left;
                    let h = bottom - top;
                    [left, top, w, h]
                };
                let source_rectangle = {
                    let x = (uv_rect.min.x * tex_w as f32) as f64;
                    let y = (uv_rect.min.y * tex_h as f32) as f64;
                    let w = ((uv_rect.max.x - uv_rect.min.x) * tex_w as f32) as f64;
                    let h = ((uv_rect.max.y - uv_rect.min.y) * tex_h as f32) as f64;
                    [x, y, w, h]
                };
                (rectangle, source_rectangle)
            });
//        glyph_rectangles.clear();
//        glyph_rectangles.extend(rectangles);
        let glyph_rectangles = rectangles.collect_vec();

        let offset_x = 0.0;
        let offset_y = layout_dims.y as f64;
        let pos = [text.offset.x as f64 + offset_x, text.offset.y as f64 + offset_y];
        let transform = math::multiply(math::multiply(self.context.view, math::translate(pos)), math::scale(1.0, -1.0));

        if let Some(outline_color) = text.outline_color {
            for dx in -1 ..= 1 {
                for dy in -1 ..= 1 {
                    let outline_rects = glyph_rectangles.iter().cloned()
                        .map(|(rect, src_rect)| {
                            let new_rect = [rect[0] + dx as f64, rect[1] + dy as f64, rect[2], rect[3]];
                            (new_rect, src_rect)
                        }).collect_vec();
                    image::draw_many(&outline_rects,
                                     outline_color.0,
                                     &self.resources.font_texture,
                                     &self.draw_state,
                                     transform,
                                     self.graphics);
                }
            }
        }

        image::draw_many(&glyph_rectangles,
                                          text.color.0,
                                          &self.resources.font_texture,
                                          &self.draw_state,
                                          transform,
                                          self.graphics);
    }

    pub fn quad(&mut self, img: String, transform: math::Matrix2d) {
        let sprite = self.resources.texture(img).texture;
        let img = Image::new();//.rect([0.0,0.0,72.0,72.0]).src_rect([0.0,0.0,32.0,32.0]);
        img.draw(&sprite, &self.draw_state, transform, self.graphics);
    }
}

