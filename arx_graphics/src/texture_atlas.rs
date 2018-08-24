use common::prelude::*;
use common::math::Rect as ArxRect;

use rect_packer::Packer;
use rect_packer as packer;
use piston_window::G2dTexture;
use std::collections::HashMap;
use resources::ImageIdentifier;

use image as image_lib;

use gfx;
use gfx_device_gl;
use std::path::Path;

#[derive(Clone)]
pub struct StoredTextureInfo {
    pub pixel_rect: ArxRect<i32>,
    pub normalized_rect: ArxRect<f32>,
}

pub struct TextureAtlas {
    packer: Packer,
    pub texture: G2dTexture,
    info_by_identifier: HashMap<ImageIdentifier, StoredTextureInfo>,
    dimensions: Vec2i,
    image: image_lib::RgbaImage,
    dirty: bool,
}

impl TextureAtlas {
    pub fn new(texture: G2dTexture, width: i32, height: i32) -> TextureAtlas {
        let packer_config = packer::Config { width, height, border_padding: 1, rectangle_padding: 1 };

        TextureAtlas {
            packer: Packer::new(packer_config),
            texture,
            info_by_identifier: HashMap::new(),
            dimensions: v2(width, height),
            image: image_lib::RgbaImage::new(width as u32, height as u32),
            dirty: true
        }
    }


    pub fn get(&self, ident: &ImageIdentifier) -> Option<&StoredTextureInfo> {
        self.info_by_identifier.get(ident)
    }

    pub fn load(&mut self, ident: ImageIdentifier, image: &image_lib::RgbaImage) -> Option<&StoredTextureInfo> {
        if let Some(packed_at) = self.packer.pack(image.width() as i32, image.height() as i32, false) {
//            (self.dimensions.y - image.height() as i32 - packed_at.y - 1)
            image_lib::imageops::replace(&mut self.image, &image, packed_at.x as u32, packed_at.y as u32);

            self.dirty = true;

            let x = packed_at.x;
            let y = packed_at.y;
            let w = packed_at.width;
            let h = packed_at.height;
            let self_w = self.dimensions.x;
            let self_h = self.dimensions.y;
            self.info_by_identifier.insert(ident.clone(), StoredTextureInfo {
                pixel_rect: ArxRect::new(x, y, w, h),
                normalized_rect: ArxRect::new(x as f32 / self_w as f32, y as f32 / self_h as f32,
                                              w as f32 / self_w as f32, h as f32 / self_h as f32),
            });

            self.get(&ident)
        } else {
            None
        }
    }

    pub fn update<C> (&mut self, encoder: &mut gfx::Encoder<gfx_device_gl::Resources, C>) where C: gfx::CommandBuffer<gfx_device_gl::Resources> {
        if self.dirty {
            self.dirty = false;
            self.texture.update(encoder, &self.image).expect("Could not update atlas");
        }
    }
}