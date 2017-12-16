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
    centered : bool
}

impl Quad {
    pub fn new(texture_identifier : String, offset : Vec2f) -> Quad {
        Quad {
            texture_identifier,
            offset,
            rotation : 0.0,
            image : Image::new(),
            centered : false
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
    pub fn rotation(mut self, rotation : f32) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn centered(mut self) -> Self {
        self.centered = true;
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
        let image = if quad.centered {
            let w = tex_info.size.x as f64;
            let h = tex_info.size.y as f64;
            quad.image.rect([-w/2.0,-h/2.0,w,h])
        } else {
            quad.image
        };

        let pos = as_f64(quad.offset);
        let transform = math::multiply(self.context.view, math::translate(pos));
        image.draw(&tex_info.texture, &self.draw_state, transform, self.graphics);
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
    texture_path : PathBuf,
    factory : gfx_device_gl::Factory
}

impl GraphicsResources {
    pub fn new(factory : gfx_device_gl::Factory, base_path : &'static str) -> GraphicsResources {
        let main_path = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap().join(base_path);
        GraphicsResources {
            images : HashMap::new(),
            assets_path : main_path.clone(),
            texture_path : main_path.join("textures"),
            factory
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
}