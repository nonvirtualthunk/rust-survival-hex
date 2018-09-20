use camera::*;
use common::color::*;
use common::hex::CartVec;
use common::prelude::*;
use common::Rect;
use find_folder;
use gfx;
use gfx_device_gl;
use itertools::Itertools;
use image as image_lib;
use piston_window::math;
use piston_window::TextureSettings;
use piston_window::types;
use rusttype as rt;
use rusttype::gpu_cache::Cache as RTCache;
use rusttype::gpu_cache::CacheBuilder as RTCacheBuilder;
use std;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use text::TextLayout;
use piston_window::G2dTexture;
use piston_window::Filter;
use piston_window::Texture;
use core::Text;
use std::path::Path;
use std::env::current_dir;
use std::env::current_exe;
use texture_atlas::TextureAtlas;
use texture_atlas::StoredTextureInfo;
use gfx::format::Format;
use image::RgbaImage;
use piston_window::G2d;
use core::FontSize;
use text::ArxFont;
use std::fs::File;
use std::io::BufReader;
use FontInfo;

pub type RTFont = rt::Font<'static>;
pub type RTPositionedGlyph = rt::PositionedGlyph<'static>;

#[derive(Clone, PartialEq, Copy, Hash, Eq, Debug)]
pub struct FontIdentifier(usize);

pub type ImageIdentifier = String;
pub type TextureAtlasIdentifier = &'static str;


#[derive(Clone)]
pub struct TextureInfo {
    pub texture: G2dTexture,
    pub size: Vec2i,
}

#[allow(dead_code)]
pub struct GraphicsAssets {
    fonts: Vec<ArxFont>,
    font_identifiers_by_name: HashMap<Str, FontIdentifier>,
    pub default_font: FontIdentifier,
    images: HashMap<ImageIdentifier, image_lib::RgbaImage>,
    atlases: HashMap<TextureAtlasIdentifier, TextureAtlas>,
    assets_path: PathBuf,
    main_path: PathBuf,
    texture_path: PathBuf,
    pub dpi_scale: f32,
}

impl GraphicsAssets {
    pub fn new(base_path: &'static str) -> GraphicsAssets {
        let mut cur_dir = current_exe().expect("not in a dir?");
        cur_dir.pop();
        cur_dir.pop();
        cur_dir.pop();
        if cur_dir.ends_with("target") {
            cur_dir.pop();
        }
        let assets_path = find_folder::SearchFolder {
            start: cur_dir,
            direction: find_folder::Search::Kids(6),
        }.for_folder("assets").expect("Could not find assets");

        println!("Found assets: {:?}", assets_path);
        let main_path = assets_path.join(base_path);
        let texture_path = main_path.join("textures");

        let mut assets = GraphicsAssets {
            fonts: Vec::new(),
            font_identifiers_by_name: HashMap::new(),
            images: HashMap::new(),
            atlases: HashMap::new(),
            assets_path,
            main_path,
            texture_path,
            dpi_scale: 1.0,
            default_font: FontIdentifier(0),
        };
        let default_font_name = ::std::env::var("DEFAULT_FONT").unwrap_or(strf("thin_pixel-7.ttf"));
        let identifier = assets.load_font(default_font_name.as_str());
        assets.default_font = identifier;
        assets
    }
}

pub struct GraphicsResources {
    pub assets: GraphicsAssets,
    textures: HashMap<ImageIdentifier, TextureInfo>,
    non_present_textures: HashSet<ImageIdentifier>,
    factory: gfx_device_gl::Factory,
    pub glyph_cache: RTCache<'static>,
    pub font_texture_data: Vec<u8>,
    pub font_texture: G2dTexture,
}

impl GraphicsResources {
    pub fn new(factory: gfx_device_gl::Factory, base_path: &'static str) -> GraphicsResources {
        let mut factory = factory;

        let texture_settings = TextureSettings::new().mag(Filter::Nearest).min(Filter::Nearest);

        let w = 512;
        let h = 512;
        let mut font_texture_data: Vec<u8> = Vec::new();
        for _i in 0..w * h {
            font_texture_data.push(126);
        }


        let font_texture = G2dTexture::from_memory_alpha(&mut factory, &font_texture_data[..], 512, 512, &texture_settings).expect("Could not make texture needed");


        GraphicsResources {
            textures: HashMap::new(),
            non_present_textures: HashSet::new(),
            factory,
            glyph_cache: RTCacheBuilder { height: 512, width: 512, pad_glyphs: true, position_tolerance: 0.5, scale_tolerance: 0.5 }.build(),
            font_texture,
            font_texture_data,
            assets: GraphicsAssets::new(base_path),
        }
    }

    pub fn read_texture(assets_path: &PathBuf, factory: &mut gfx_device_gl::Factory, identifier: ImageIdentifier) -> TextureInfo {
        GraphicsResources::read_texture_opt(assets_path, factory, identifier.clone()).expect(format!("Could not load image with identifier {:?}", identifier).as_str())
    }

    pub fn read_texture_opt(assets_path: &PathBuf, factory: &mut gfx_device_gl::Factory, identifier: ImageIdentifier) -> Option<TextureInfo> {
        let identifier = if !identifier.ends_with(".png") {
            format!("{}.png", identifier)
        } else {
            identifier
        };

        let path = assets_path.join(identifier);
        let mut texture_settings = TextureSettings::new();
        texture_settings.set_min(Filter::Nearest);
        texture_settings.set_mag(Filter::Nearest);

        let image = if let Ok(image) = image_lib::open(path.as_path()) {
            image
        } else {
            return None;
        };
        let image = image_lib::imageops::flip_vertical(&image);

        let tex = Texture::from_image(
            factory,
            &image,
            &texture_settings,
        ).expect(format!("Could not create image path {:?}", path).as_str());

        let w = image.width();
        let h = image.height();
        Some(TextureInfo {
            texture: tex,
            size: v2(w as i32, h as i32),
        })
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

    pub fn texture_opt(&mut self, identifier: ImageIdentifier) -> Option<TextureInfo> {
        if self.textures.contains_key(&identifier) {
            Some(self.textures.get(&identifier).unwrap().clone())
        } else if self.non_present_textures.contains(&identifier) {
            None
        } else {
            if let Some(tex) = GraphicsResources::read_texture_opt(&self.assets.texture_path, &mut self.factory, identifier.clone()) {
                self.textures.insert(identifier.clone(), tex);
                Some(self.textures.get(&identifier).unwrap().clone())
            } else {
                self.non_present_textures.insert(identifier.clone());
                None
            }
        }
    }
    pub fn is_valid_texture(&mut self, identifier: ImageIdentifier) -> bool {
        self.texture_opt(identifier).is_some()
    }

    pub fn font(&mut self, identifier: FontIdentifier) -> &ArxFont {
        self.assets.font(identifier)
    }

    /// Takes in the name of a font and returns the identifier by which it can be referenced. Currently that is
    /// just the name of the font, but if we want to change it later we may
    pub fn font_id(&mut self, identifier: &'static str) -> FontIdentifier {
        self.assets.load_font(identifier)
    }

    pub fn layout_text(&mut self, text: &Text) -> TextLayout {
        self.assets.layout_text(text)
    }

    pub fn line_height(&self, font: &ArxFont, size: FontSize) -> f32 {
        self.assets.line_height(font, size)
    }

    pub fn string_dimensions_no_wrap<'b>(&mut self, font_identifier: FontIdentifier, text: &'b str, size: FontSize) -> Vec2f {
        self.assets.string_dimensions_no_wrap(font_identifier, text, size)
    }


    pub fn atlas_texture(&mut self, atlas: TextureAtlasIdentifier) -> &G2dTexture {
        &self.assets.atlases.get(atlas).expect("atlas did not exist").texture
    }

    pub fn texture_from_atlas(&mut self, texture: ImageIdentifier, atlas: TextureAtlasIdentifier) -> StoredTextureInfo {
        self.assets.texture_from_atlas(texture, atlas, &mut self.factory)
    }

    pub fn upload_atlases(&mut self, graphics: &mut G2d) {
        for atlas in self.assets.atlases.values_mut() {
            atlas.update(graphics.encoder);
        }
    }
}

impl GraphicsAssets {
    pub fn texture_from_atlas(&mut self, texture: ImageIdentifier, atlas: TextureAtlasIdentifier, factory: &mut gfx_device_gl::Factory) -> StoredTextureInfo {
        let atlases = &mut self.atlases;
        let atlas = atlases.entry(atlas).or_insert_with(|| {
            let texture_settings = TextureSettings::new().mag(Filter::Nearest).min(Filter::Nearest);

            let w = 1024;
            let h = 1024;
            let mut texture_data: Vec<u8> = Vec::new();
            for _i in 0..w * h {
                texture_data.push(255);
            }

            let texture = G2dTexture::from_memory_alpha(factory, &texture_data[..], w, h, &texture_settings)
                .expect("Could not make texture needed for atlas");
            TextureAtlas::new(texture, w as i32, h as i32)
        });

        match atlas.get(&texture) {
            Some(existing) => existing.clone(),
            None => {
                // --------------------------- Had to inline the image(...) call here to satisfy the borrow checker for som reason -------------
                let img = if self.images.contains_key(&texture) {
                    self.images.get(&texture).unwrap()
                } else {
                    let img = GraphicsAssets::read_image(&self.texture_path, texture.clone());
                    self.images.insert(texture.clone(), img);
                    self.images.get(&texture).unwrap()
                };
                // ---------------------------- rust doesn't like it when you mutate things ----------------------------------------------------

                atlas.load(texture, img).unwrap().clone()
            }
        }
    }

    pub fn load_font(&mut self, name: &str) -> FontIdentifier {
        if let Some(ident) = self.font_identifiers_by_name.get(name) {
            *ident
        } else {
            let font_path = self.assets_path.join("fonts").join(name);
            use std::io::Read;
            let mut file = std::fs::File::open(font_path).expect("Could not open font file");
            let mut file_buffer = Vec::new();
            file.read_to_end(&mut file_buffer).expect("Could not read file to end to load font");
            let identifier = FontIdentifier(self.fonts.len());
            let rt_font = RTFont::from_bytes(file_buffer).expect("Could not load font");

            let info_name: String = strf(name).chars().take_while(|c| *c != '.').collect();
            let font_info_path = self.assets_path.join("fonts").join(format!("{}.info", info_name));
            println!("Attempting to load overrides from {:?}", font_info_path);
            let font_info = if let Ok(file) = File::open(font_info_path) {
                let buf_reader = BufReader::new(file);
                use ron;
                match ron::de::from_reader(buf_reader) {
                    Ok(font_info) => {
                        info!("loaded overrides:\n{:?}", font_info);
                        font_info
                    }
                    Err(error) => {
                        warn!("Could not deserialize font info, falling back on default: {:?}", error);
                        FontInfo::default()
                    }
                }
            } else { FontInfo::default() };

            self.fonts.push(ArxFont { font: rt_font, font_info });
            identifier
        }
    }

    pub fn font(&mut self, identifier: FontIdentifier) -> &ArxFont {
        self.fonts.get(identifier.0).or(self.fonts.get(0)).expect("graphics assets must contain at least one font")
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

    pub fn image(&mut self, identifier: ImageIdentifier) -> &image_lib::RgbaImage {
        if self.images.contains_key(&identifier) {
            self.images.get(&identifier).unwrap()
        } else {
            let img = GraphicsAssets::read_image(&self.texture_path, identifier.clone());
            self.images.insert(identifier.clone(), img);
            self.images.get(&identifier).unwrap()
        }
    }

    pub fn layout_text(&mut self, text: &Text) -> TextLayout {
        let dpi_scale = self.dpi_scale;
        let font = self.font(text.font_identifier.unwrap_or(self.default_font));
        TextLayout::layout_text(text.text.as_str(), font, text.size, dpi_scale, text.wrap_to)
    }

    pub fn line_height(&self, font: &ArxFont, size: FontSize) -> f32 {
        TextLayout::line_height(font, size, self.dpi_scale)
    }


    pub fn string_dimensions<'b>(&mut self, font_identifier: FontIdentifier, text: &'b str, size: FontSize, wrap_at: f32) -> Vec2f {
        if text.is_empty() {
            v2(0.0, 0.0)
        } else {
            let dpi_scale = self.dpi_scale;
            let font = self.font(font_identifier);
            let layout = TextLayout::layout_text(text, font, size, dpi_scale, wrap_at);

            layout.dimensions()
        }
    }

    pub fn string_dimensions_no_wrap<'b>(&mut self, font_identifier: FontIdentifier, text: &'b str, size: FontSize) -> Vec2f {
        self.string_dimensions(font_identifier, text, size, 10000000.0)
    }
}