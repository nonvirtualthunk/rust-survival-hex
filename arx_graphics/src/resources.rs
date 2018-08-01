use camera::*;
use common::color::*;
use common::hex::CartVec;
use common::prelude::*;
use common::Rect;
use find_folder;
use gfx;
use gfx_device_gl;
use image as image_lib;
use image::GenericImage;
use itertools::Itertools;
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
use piston_window::G2dTexture;
use piston_window::Filter;
use piston_window::Texture;
use core::Text;

pub type RTFont = rt::Font<'static>;
pub type RTPositionedGlyph = rt::PositionedGlyph<'static>;

pub type FontIdentifier = &'static str;
pub type ImageIdentifier = String;

#[derive(Clone)]
pub struct TextureInfo {
    pub texture: G2dTexture,
    pub size: Vec2i,
}

#[allow(dead_code)]
pub struct GraphicsAssets {
    fonts: HashMap<FontIdentifier, RTFont>,
    images: HashMap<ImageIdentifier, image_lib::RgbaImage>,
    assets_path: PathBuf,
    main_path: PathBuf,
    texture_path: PathBuf,
    pub dpi_scale : f32
}
impl GraphicsAssets {
    pub fn new(base_path: &'static str) -> GraphicsAssets {
        let assets_path = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let main_path = assets_path.join(base_path);
        let texture_path = main_path.join("textures");

        GraphicsAssets {
            fonts : HashMap::new(),
            images : HashMap::new(),
            assets_path,
            main_path,
            texture_path,
            dpi_scale : 1.0
        }
    }
}

pub struct GraphicsResources {
    pub assets : GraphicsAssets,
    textures: HashMap<ImageIdentifier, TextureInfo>,
    factory: gfx_device_gl::Factory,
    pub glyph_cache : RTCache<'static>,
    pub font_texture_data : Vec<u8>,
    pub font_texture : G2dTexture,
}

impl GraphicsResources {
    pub fn new(factory: gfx_device_gl::Factory, base_path: &'static str) -> GraphicsResources {
        let mut factory = factory;

        let texture_settings = TextureSettings::new().mag(Filter::Nearest).min(Filter::Nearest);

        let w = 512;
        let h = 512;
        let mut font_texture_data : Vec<u8> = Vec::new();
        for _i in 0..w * h {
            font_texture_data.push(126);
        }


        let font_texture = G2dTexture::from_memory_alpha(&mut factory, &font_texture_data[..], 512, 512, &texture_settings).expect("Could not make texture needed");


        GraphicsResources {
            textures: HashMap::new(),
            factory,
            glyph_cache : RTCacheBuilder { height : 512, width : 512, pad_glyphs : true, position_tolerance: 0.5, scale_tolerance : 0.5 }.build(),
            font_texture,
            font_texture_data,
            assets : GraphicsAssets::new(base_path)
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
        if self.textures.contains_key(&identifier) {
            self.textures.get(&identifier).unwrap().clone()
        } else {
            let tex = GraphicsResources::read_texture(&self.assets.texture_path, &mut self.factory, identifier.clone());
            self.textures.insert(identifier.clone(), tex);
            self.textures.get(&identifier).unwrap().clone()
        }
    }

    pub fn font(&mut self, identifier: FontIdentifier) -> &RTFont {
        self.assets.font(identifier)
    }

    /// Takes in the name of a font and returns the identifier by which it can be referenced. Currently that is
    /// just the name of the font, but if we want to change it later we may
    pub fn font_id(&mut self, identifier : &'static str) -> FontIdentifier {
        self.font(identifier);
        identifier
    }

    pub fn layout_text(&mut self, text : &Text) -> TextLayout {
        self.assets.layout_text(text)
    }

    pub fn line_height(&self, font: &RTFont, size :u32) -> f32 {
        self.assets.line_height(font, size)
    }

    pub fn string_dimensions_no_wrap<'b>(&mut self, font_identifier: FontIdentifier, text: &'b str, size: u32) -> Vec2f {
        self.assets.string_dimensions_no_wrap(font_identifier, text, size)
    }
}

impl GraphicsAssets {
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

    pub fn read_image(assets_path: &PathBuf, identifier: String) -> image_lib::RgbaImage {
        let identifier = if !identifier.ends_with(".png") {
            format!("{}.png", identifier)
        } else {
            identifier
        };

        let path = assets_path.join(identifier);

        let image = image_lib::open(path.as_path()).expect(format!("Could not find path {:?}", path).as_str());
        let image = image_lib::imageops::flip_vertical(&image);

        image
    }

    pub fn image(&mut self, identifier : ImageIdentifier) -> &image_lib::RgbaImage {
        if self.images.contains_key(&identifier) {
            self.images.get(&identifier).unwrap()
        } else {
            let img = GraphicsAssets::read_image(&self.assets_path, identifier.clone());
            self.images.insert(identifier.clone(), img);
            self.images.get(&identifier).unwrap()
        }
    }

    pub fn layout_text(&mut self, text : &Text) -> TextLayout {
        let dpi_scale = self.dpi_scale;
        let font = self.font(text.font_identifier);
        TextLayout::layout_text(text.text.as_str(), font, text.size, dpi_scale, text.wrap_to)
    }

    pub fn line_height(&self, font: &RTFont, size :u32) -> f32 {
        TextLayout::line_height(font, size, self.dpi_scale)
    }


    pub fn string_dimensions<'b>(&mut self, font_identifier: FontIdentifier, text: &'b str, size: u32, wrap_at : f32) -> Vec2f {
        if text.is_empty() {
            v2(0.0,0.0)
        } else {
            let dpi_scale = self.dpi_scale;
            let font = self.font(font_identifier);
            let layout = TextLayout::layout_text(text, font, size, dpi_scale, wrap_at);

            layout.dimensions()
        }
    }

    pub fn string_dimensions_no_wrap<'b>(&mut self, font_identifier: FontIdentifier, text: &'b str, size: u32) -> Vec2f {
        self.string_dimensions(font_identifier, text, size, 10000000.0)
    }
}