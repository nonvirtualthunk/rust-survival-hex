use piston_window::*;

use find_folder;
use std::path::PathBuf;

use common::prelude::*;

use std::collections::HashMap;

use gfx_device_gl;

use piston_window::types::*;
use piston_window::math;

use image as image_lib;
use image::GenericImage;

use camera::*;

pub struct Quad {
    image : Image,
    texture_identifier : String,
    offset : Vec2f,
    rotation : f32,
    centered : bool,
    size : Option<Vec2f>

}

impl Quad {
    pub fn new(texture_identifier : String, offset : Vec2f) -> Quad {
        Quad {
            texture_identifier,
            offset,
            rotation : 0.0,
            image : Image::new(),
            centered : false,
            size : None
        }
    }

    pub fn color(mut self, color : Color) -> Self {
        self.image = self.image.color(color);
        self
    }
    pub fn offset(mut self, offset : Vec2f) -> Self {
        self.offset = offset;
        self
    }
//    pub fn hex_pos(mut self, offset : AxialCoord, ) -> Self {
//        self.offset = offset.
//    }
    pub fn rotation(mut self, rotation : f32) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
    pub fn size(mut self, size : Vec2f) -> Self {
        self.size = Some(size);
        self
    }
}

const DEFAULT_FONT_IDENTIFIER : &'static str = "NotoSerif-Regular.ttf";

pub struct Text <'a> {
    text : &'a str,
    font_identifier : String,
    offset : Vec2f,
    size : u32,
    color : Color,
    rounded : bool,
    centered_y : bool,
    centered_x : bool
}

impl <'a> Text <'a> {
    pub fn new(text : &str, size : u32) -> Text {
        Text {
            text,
            size,
            font_identifier : String::from(DEFAULT_FONT_IDENTIFIER),
            offset : v2(0.0f32, 0.0f32),
            color : [0.0,0.0,0.0,1.0],
            rounded : true,
            centered_y : false,
            centered_x : true
        }
    }

    pub fn color(mut self, color : Color) -> Self {
        self.color = color;
        self
    }

    pub fn colord(mut self, color : [f64; 4]) -> Self {
        self.color = [color[0] as f32, color[1] as f32, color[2] as f32, color[3] as f32];
        self
    }

    pub fn offset(mut self, offset : Vec2f) -> Self {
        self.offset = offset;
        self
    }

    pub fn font(mut self, font_identifier : String) -> Self {
        self.font_identifier = font_identifier;
        self
    }


}

pub struct GraphicsWrapper<'a, 'b : 'a> {
    pub resources : &'a mut GraphicsResources,
    pub graphics : &'a mut G2d<'b>,
    pub context : Context,
    pub draw_state : DrawState,
    pub viewport : Viewport
}

impl <'a, 'b : 'a> GraphicsWrapper<'a,'b> {
    pub fn new(context : Context, resources : &'a mut GraphicsResources, graphics : &'a mut G2d<'b>) -> GraphicsWrapper<'a,'b> {
        GraphicsWrapper {
            context,
            resources,
            graphics,
            draw_state : DrawState::default(),
            viewport : context.viewport.unwrap_or_else(|| Viewport {
                window_size : [256,256],
                draw_size : [256,256],
                rect : [0,0,256,256]
            })
        }
    }

    pub fn draw_quad(&mut self, quad : Quad){
        let tex_info = self.resources.texture(String::from(quad.texture_identifier));
        let (w,h) = if let Some(size) = quad.size {
            (size.x as f64, size.y as f64)
        } else {
            (tex_info.size.x as f64, tex_info.size.y as f64)
        };

        let image = if quad.centered {
            quad.image.rect([-w/2.0,-h/2.0,w,h])
        } else {
            quad.image.rect([0.0,0.0,w,h])
        };

        let pos = as_f64(quad.offset);
        let transform = math::multiply(self.context.view, math::translate(pos));
        image.draw(&tex_info.texture, &self.draw_state, transform, self.graphics);
    }

    pub fn draw_text(&mut self, text : Text) {
        let glyphs = self.resources.glyphs(&text.font_identifier);

        let (offset_x, offset_y) = if text.centered_x || text.centered_y {
            use piston_window::character::CharacterCache;

            let mut x = 0.0;
            let mut y = 0.0;
            for ch in text.text.chars() {
                let character = glyphs.character(text.size, ch).unwrap();
                x += character.width();
                y = character.height().max(y);
            }
            if text.centered_x {
                x *= -0.5
            } else {
                x = 0.0
            }

            if text.centered_y {
                y *= -0.5
            } else {
                y = 0.0
            }
            (x,y)
        } else {
            (0.0, 0.0)
        };

        let pos = [text.offset.x as f64 + offset_x, text.offset.y as f64 + offset_y];
        let transform = math::multiply(math::multiply(self.context.view, math::translate(pos)), math::scale(1.0,-1.0));

        let mut raw = text::Text::new_color(text.color, text.size);
        raw.round = text.rounded;
        raw.draw(text.text, glyphs, &self.draw_state, transform, self.graphics).unwrap();
    }

    pub fn quad(&mut self, img : String, transform : math::Matrix2d) {
        let sprite = self.resources.texture(img).texture;
        let img = Image::new();//.rect([0.0,0.0,72.0,72.0]).src_rect([0.0,0.0,32.0,32.0]);
        img.draw(&sprite, &self.draw_state, transform, self.graphics);
    }
}

#[derive(Clone)]
pub struct TextureInfo {
    texture : G2dTexture,
    size : Vec2i
}

#[allow(dead_code)]
pub struct GraphicsResources {
    images : HashMap<String, TextureInfo>,
    assets_path : PathBuf,
    main_path : PathBuf,
    texture_path : PathBuf,
    factory : gfx_device_gl::Factory,
    glyphs : HashMap<String, Glyphs>
}

impl GraphicsResources {
    pub fn new(factory : gfx_device_gl::Factory, base_path : &'static str) -> GraphicsResources {
        let assets_path = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let main_path = assets_path.join(base_path);
        let texture_path = main_path.join("textures");
        GraphicsResources {
            images : HashMap::new(),
            assets_path,
            main_path,
            texture_path,
            factory,
            glyphs : HashMap::new()
        }
    }

     pub fn read_texture(assets_path : &PathBuf, factory : &mut gfx_device_gl::Factory, identifier : String) -> TextureInfo {
         let identifier = if !identifier.ends_with(".png") {
             format!("{}.png",identifier)
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
             &texture_settings
         ).expect(format!("Could not find path {:?}", path).as_str());

         let w = image.width();
         let h = image.height();
         TextureInfo {
             texture : tex,
             size : v2(w as i32,h as i32)
         }
     }

    pub fn texture(&mut self, identifier: String) -> TextureInfo {
        if self.images.contains_key(&identifier) {
            self.images.get(&identifier).unwrap().clone()
        } else {
            let tex = GraphicsResources::read_texture(&self.texture_path, &mut self.factory, identifier.clone());
            self.images.insert(identifier.clone(), tex);
            self.images.get(&identifier).unwrap().clone()
        }

    }

    pub fn glyphs(&mut self, identifier: &String) -> &mut Glyphs {
        if !self.glyphs.contains_key(identifier) {
            let font = self.assets_path.join("fonts").join(identifier);
            let texture_settings = TextureSettings::new().min(Filter::Linear).mag(Filter::Linear);
            let glyphs = Glyphs::new(font, self.factory.clone(), texture_settings)
                .expect(format!("Could not load font at {:?}", self.assets_path.join("fonts").join(identifier)).as_str());
            self.glyphs.insert(identifier.clone(), glyphs);
        }

        self.glyphs.get_mut(identifier).unwrap()
    }

    pub fn with_glyphs<F: Fn(&mut Glyphs)>(&mut self, identifier: String, func : F) {
        if !self.glyphs.contains_key(&identifier) {
            let font = self.assets_path.join("fonts").join(&identifier);
            self.glyphs.insert(identifier.clone(), Glyphs::new(font, self.factory.clone(), TextureSettings::new()).unwrap());
        }

        let mut glyphs = self.glyphs.get_mut(&identifier).unwrap();
        func(&mut glyphs);
    }
}