use piston_window::*;
use piston_window::types;
use piston_window::TextureSettings;

use find_folder;
use std::path::PathBuf;

use common::prelude::*;
use common::color::*;
use common::Rect;
use common::hex::CartVec;

use std::collections::HashMap;

use gfx_device_gl;
use itertools::Itertools;

use piston_window::math;

use image as image_lib;
use image::GenericImage;

use rusttype as rt;
use rusttype::gpu_cache::Cache as RTCache;
use rusttype::gpu_cache::CacheBuilder as RTCacheBuilder;
use std;

pub type RTFont = rt::Font<'static>;
pub type RTPositionedGlyph = rt::PositionedGlyph<'static>;

use gfx;

use camera::*;
use text::TextLayout;



pub type FontIdentifier = &'static str;
pub type ImageIdentifier = String;

#[derive(Clone)]
pub struct Quad {
    pub image: Image,
    pub texture_identifier: ImageIdentifier,
    pub offset: Vec2f,
    pub rotation: f32,
    pub centered: bool,
    pub size: Option<Vec2f>,
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
}

pub const DEFAULT_FONT_IDENTIFIER: FontIdentifier = "NotoSans-Regular.ttf";

pub struct Text {
    pub text: String,
    pub font_identifier: FontIdentifier,
    pub offset: Vec2f,
    pub size: u32,
    pub color: Color,
    pub rounded: bool,
    pub centered_y: bool,
    pub centered_x: bool,
}

impl Clone for Text {
    fn clone(&self) -> Self {
        Text {
            text: self.text.clone(),
            font_identifier: self.font_identifier,
            offset: self.offset.clone(),
            size: self.size.clone(),
            color: self.color.clone(),
            rounded: self.rounded,
            centered_y: self.centered_y,
            centered_x: self.centered_x,
        }
    }
}

impl Text {
    pub fn new(text: String, size: u32) -> Text {
        Text {
            text,
            size,
            font_identifier: DEFAULT_FONT_IDENTIFIER,
            offset: v2(0.0f32, 0.0f32),
            color: Color([0.0, 0.0, 0.0, 1.0]),
            rounded: true,
            centered_y: false,
            centered_x: true,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
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
        self.font_identifier = font_identifier;
        self
    }

    pub fn centered(mut self, x: bool, y: bool) -> Self {
        self.centered_x = x;
        self.centered_y = y;
        self
    }
}

#[derive(Default)]
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

    pub fn add_quad(mut self, quad: Quad) -> Self {
        self.quads.push(quad);
        self
    }

    pub fn add_text(mut self, text: Text) -> Self {
        self.text.push(text);
        self
    }

    pub fn append(&mut self, other: &mut DrawList) {
        self.quads.append(&mut other.quads);
        self.text.append(&mut other.text);
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
        };

        let pos = as_f64(quad.offset);
        let transform = math::multiply(self.context.view, math::translate(pos));
        image.draw(&tex_info.texture, &self.draw_state, transform, self.graphics);
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
                let img_info = gfx::texture::NewImageInfo {
                    xoffset : rect.min.x as u16,
                    yoffset : rect.min.y as u16,
                    zoffset : 0,
                    width : rect.width() as u16,
                    height : rect.height() as u16,
                    depth: 0,
                    format: (),
                    mipmap: 0

                };

                let offset = [rect.min.x, rect.min.y];
                let size = [rect.width(), rect.height()];
                texture::UpdateTexture::update(font_texture, encoder, texture::Format::Rgba8, &font_texture_data[..], offset, size)
                                    .expect("Failed to update texture");
//                let data = gfx::memory::cast_slice(&font_texture_data[..]);
//                encoder.update_texture::<_, gfx::format::Rgba8>(&font_surface, None, img_info, data)
//                    .expect("Failed to update texture");
            }).expect("Could not update glyph cache");
        }

        let dpi_factor = dpi_scale;
        let (tex_w, tex_h) = self.resources.font_texture.get_size();

        let rectangles = glyphs.into_iter()
            .filter_map(|g| glyph_cache.rect_for(cache_id, &g).ok().unwrap_or(None))
            .map(|(uv_rect, screen_rect)| {
                let rectangle = {
                    let div_dpi_factor = |s| (s as f32 / dpi_factor as f32) as f64;
                    let left = div_dpi_factor(screen_rect.min.x);
                    let top = div_dpi_factor(screen_rect.min.y);
                    let right = div_dpi_factor(screen_rect.max.x);
                    let bottom = div_dpi_factor(screen_rect.max.y);
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
//        let transform = math::multiply(self.context.view, math::translate(pos));

        image::draw_many(&glyph_rectangles,
                                          text.color.0,
                                          &self.resources.font_texture,
                                          &self.draw_state,
                                          transform,
                                          self.graphics);


//        let transform = math::multiply(math::multiply(self.context.view, math::translate([-512.0,256.0])), math::scale(1.0, -1.0));
//        image::Image::new().draw(&self.resources.font_texture, &self.draw_state, transform, self.graphics);
    }

    pub fn draw_text_old(&mut self, text: Text) {
        let dimensions_no_wrap = self.resources.string_dimensions_no_wrap(text.font_identifier, text.text.as_str(), text.size);
        let line_height = text.size; //self.resources.line_height(text.font_identifier, text.size);

        let lines : Vec<&str> = text.text.split('\n').collect_vec();
        let glyphs = self.resources.glyphs(text.font_identifier);

        for (i,line) in lines.iter().enumerate() {
            let offset_x = if text.centered_x {
                dimensions_no_wrap.x as f64 * -0.5
            } else {
                0.0
            };
            let offset_y = if text.centered_y {
                dimensions_no_wrap.y as f64 * -0.5
            } else {
                0.0
            };
            let offset_y = offset_y + (dimensions_no_wrap.y as f64 - (line_height as f64 * (i + 1) as f64));

            let pos = [text.offset.x as f64 + offset_x, text.offset.y as f64 + offset_y];
            let transform = math::multiply(math::multiply(self.context.view, math::translate(pos)), math::scale(1.0, -1.0));

            let mut raw = text::Text::new_color(text.color.0, text.size);
            raw.round = text.rounded;
            raw.draw(line, glyphs, &self.draw_state, transform, self.graphics).unwrap();
        }
    }

    pub fn quad(&mut self, img: String, transform: math::Matrix2d) {
        let sprite = self.resources.texture(img).texture;
        let img = Image::new();//.rect([0.0,0.0,72.0,72.0]).src_rect([0.0,0.0,32.0,32.0]);
        img.draw(&sprite, &self.draw_state, transform, self.graphics);
    }
}

#[derive(Clone)]
pub struct TextureInfo {
    texture: G2dTexture,
    size: Vec2i,
}

#[allow(dead_code)]
pub struct GraphicsResources {
    images: HashMap<ImageIdentifier, TextureInfo>,
    assets_path: PathBuf,
    main_path: PathBuf,
    texture_path: PathBuf,
    factory: gfx_device_gl::Factory,
    glyphs: HashMap<FontIdentifier, Glyphs>,
    fonts: HashMap<FontIdentifier, RTFont>,
    glyph_cache : RTCache<'static>,
    pub font_texture_data : Vec<u8>,
    pub font_texture : G2dTexture,
    pub dpi_scale : f32
}

impl GraphicsResources {
    pub fn new(factory: gfx_device_gl::Factory, base_path: &'static str) -> GraphicsResources {
        let mut factory = factory;
        let assets_path = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let main_path = assets_path.join(base_path);
        let texture_path = main_path.join("textures");

        let texture_settings = TextureSettings::new().mag(Filter::Nearest).min(Filter::Nearest);

        let w = 512;
        let h = 512;
        let mut font_texture_data : Vec<u8> = Vec::new();
        for _i in 0..w * h {
            font_texture_data.push(126);
        }


        let font_texture = G2dTexture::from_memory_alpha(&mut factory, &font_texture_data[..], 512, 512, &texture_settings).expect("Could not make texture needed");


        GraphicsResources {
            images: HashMap::new(),
            assets_path,
            main_path,
            texture_path,
            factory,
            glyphs: HashMap::new(),
            fonts: HashMap::new(),
            glyph_cache : RTCacheBuilder { height : 512, width : 512, pad_glyphs : true, position_tolerance: 0.5, scale_tolerance : 0.5 }.build(),
            font_texture,
            font_texture_data,
            dpi_scale: 1.0
        }
    }

    pub fn read_texture(assets_path: &PathBuf, factory: &mut gfx_device_gl::Factory, identifier: ImageIdentifier) -> TextureInfo {
        let identifier = if !identifier.ends_with(".png") {
            format!("{}.png", identifier)
        } else {
            identifier
        };

        let path = assets_path.join(identifier);
        let mut texture_settings = TextureSettings::new();
        texture_settings.set_min(Filter::Nearest);
        texture_settings.set_mag(Filter::Nearest);

        let image = image_lib::open(path.as_path()).expect(format!("Could not find path {:?}", path).as_str());
        let image = image_lib::imageops::flip_vertical(&image);

        let tex = Texture::from_image(
            factory,
            &image,
            &texture_settings,
        ).expect(format!("Could not find path {:?}", path).as_str());

        let w = image.width();
        let h = image.height();
        TextureInfo {
            texture: tex,
            size: v2(w as i32, h as i32),
        }
    }

    pub fn texture(&mut self, identifier: ImageIdentifier) -> TextureInfo {
        if self.images.contains_key(&identifier) {
            self.images.get(&identifier).unwrap().clone()
        } else {
            let tex = GraphicsResources::read_texture(&self.texture_path, &mut self.factory, identifier.clone());
            self.images.insert(identifier.clone(), tex);
            self.images.get(&identifier).unwrap().clone()
        }
    }

    pub fn font(&mut self, identifier: FontIdentifier) -> &RTFont {
        if !self.fonts.contains_key(identifier) {
            let font_path = self.assets_path.join("fonts").join(identifier);
            use std::io::Read;
            let mut file = std::fs::File::open(font_path).expect("Could not open font file");
            let mut file_buffer = Vec::new();
            file.read_to_end(&mut file_buffer).expect("Could not read file to end to load font");
            self.fonts.insert(identifier, RTFont::from_bytes(file_buffer).expect("Could not load font"));
        }
        self.fonts.get(identifier).unwrap()
    }

    pub fn glyphs(&mut self, identifier: FontIdentifier) -> &mut Glyphs {
        if !self.glyphs.contains_key(identifier) {
            let font = self.assets_path.join("fonts").join(identifier);
            let texture_settings = TextureSettings::new().min(Filter::Linear).mag(Filter::Linear);
            let glyphs = Glyphs::new(font, self.factory.clone(), texture_settings)
                .expect(format!("Could not load font at {:?}", self.assets_path.join("fonts").join(identifier)).as_str());
            self.glyphs.insert(identifier, glyphs);
        }

        self.glyphs.get_mut(identifier).unwrap()
    }

    pub fn with_glyphs<F: Fn(&mut Glyphs)>(&mut self, identifier: FontIdentifier, func: F) {
        if !self.glyphs.contains_key(identifier) {
            let font = self.assets_path.join("fonts").join(&identifier);
            self.glyphs.insert(identifier, Glyphs::new(font, self.factory.clone(), TextureSettings::new()).unwrap());
        }

        let mut glyphs = self.glyphs.get_mut(&identifier).unwrap();
        func(&mut glyphs);
    }

    pub fn layout_text(&mut self, text : &Text) -> TextLayout {
        let dpi_scale = self.dpi_scale;
        let font = self.font(text.font_identifier);
        TextLayout::layout_text(text.text.as_str(), font, text.size, dpi_scale)
    }

    pub fn line_height(&self, font: &RTFont, size :u32) -> f32 {
        TextLayout::line_height(font, size, self.dpi_scale)
    }

    pub fn string_dimensions_no_wrap<'b>(&mut self, font_identifier: FontIdentifier, text: &'b str, size: u32) -> Vec2f {
        if text.is_empty() {
            v2(0.0,0.0)
        } else {
            let dpi_scale = self.dpi_scale;
            let font = self.font(font_identifier);
            let layout = TextLayout::layout_text(text, font, size, dpi_scale);

            layout.dimensions()
        }
    }
}