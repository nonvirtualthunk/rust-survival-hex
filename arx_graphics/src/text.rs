use rusttype as rt;

use common::Rect;
use common::prelude::*;
use core::FontSize;
use std::collections::HashMap;

pub type RTFont = rt::Font<'static>;
pub type RTPositionedGlyph = rt::PositionedGlyph<'static>;

#[derive(Serialize,Deserialize,Clone,Debug)]
#[serde(default)]
pub struct FontInfo {
    pub sizing_overrides : HashMap<FontSize, u32>,
    pub line_height_multiplier: f32,
    pub space_multiplier: f32
}

impl Default for FontInfo {
    fn default() -> Self {
        FontInfo {
            sizing_overrides : HashMap::new(),
            line_height_multiplier : 1.0,
            space_multiplier: 1.0,
        }
    }
}

pub struct ArxFont {
    pub font : RTFont,
    pub font_info : FontInfo,
}
impl ArxFont {
    pub fn point_size_for(&self, font_size : FontSize) -> u32 {
        match font_size {
            FontSize::BasePlusPoints(base, pts) => self.point_size_for(FontSize::font_size_by_ordinal(base)) + pts as u32,
            _ => *self.font_info.sizing_overrides.get(&font_size).unwrap_or(&font_size.default_point_size())
        }
    }
}


pub struct TextLayout {
    pub glyphs : Vec<RTPositionedGlyph>,
    bounds : Rect<f32>,
    pub dpi_scale : f32
}

impl TextLayout {
    pub fn new(glyphs : Vec<RTPositionedGlyph>, bounds : Rect<f32>, dpi_scale : f32) -> TextLayout {
        TextLayout {
            glyphs,
            bounds,
            dpi_scale
        }
    }

    pub fn dimensions (&self) -> Vec2f {
        v2(self.bounds.w / self.dpi_scale, self.bounds.h / self.dpi_scale)
    }

    pub fn layout_text<'a,'b>(string : &'a str, arx_font : &'b ArxFont, size : FontSize, dpi_scale : f32, wrap_at: f32) -> TextLayout {
//        use rusttype::unicode_normalization::UnicodeNormalization;
        let effective_size = arx_font.point_size_for(size);
        let font = &arx_font.font;
        let scale = rt::Scale::uniform(((effective_size * 4) as f32 / 3.0) * dpi_scale);
        let vmetrics = font.v_metrics(scale);
        let line_height = TextLayout::line_height(arx_font, size, dpi_scale);

        let mut all_glyphs : Vec<RTPositionedGlyph> = Vec::new();
        let mut max_x : f32 = 0.0;
        let ascent = vmetrics.ascent * arx_font.font_info.line_height_multiplier;
        let space_mult = arx_font.font_info.space_multiplier;
        let mut line_y = ascent;

        let mut last_glyph_id = None;
        let mut line_x = 0.0;

        let mut last_word_break = 0;

        for c in string.chars() {
            if c.is_control() {
                match c {
                    '\n' => {
                        max_x = max_x.max(line_x);
                        line_x = 0.0;
                        line_y += line_height;
                        last_glyph_id = None;
                    },
                    _ => {}
                };
                last_word_break = all_glyphs.len() - 1;
            } else {
                if c == ' ' {
                    last_word_break = all_glyphs.len();
                    max_x = max_x.max(line_x);
                }
                let g = font.glyph(c).scaled(scale);

                let kerning_dist = if let Some(last) = last_glyph_id {
                    font.pair_kerning(scale, last, g.id())
                } else {
                    0.0
                };
                line_x += kerning_dist;
                let mut glyph = g.positioned(rt::point(line_x, line_y));
                if let Some(bb) = glyph.pixel_bounding_box() {
                    if bb.max.x as f32 > wrap_at * dpi_scale {
                        line_x = 0.0;
                        line_y += line_height;

                        if last_word_break < all_glyphs.len()-1 {
                            let shift_back = all_glyphs[last_word_break+1].position().x;

                            let mut new_glyphs = Vec::new();
                            for old_glyph in all_glyphs.drain(last_word_break+1 ..) {
                                let old_pos = old_glyph.position();
                                let new_pos_x = old_pos.x - shift_back;
                                let new_glyph = old_glyph.into_unpositioned().positioned(rt::point(new_pos_x, line_y));
                                line_x = new_pos_x + new_glyph.unpositioned().h_metrics().advance_width;
                                new_glyphs.push(new_glyph);
                            }
                            all_glyphs.extend_from_slice(new_glyphs.as_ref());
                        }

                        glyph = glyph.into_unpositioned().positioned(rt::point(line_x, line_y));
                    }
                }

                let mut advance = glyph.unpositioned().h_metrics().advance_width;
                if c == ' ' { advance *= space_mult; }
                line_x += advance;
                last_glyph_id = Some(glyph.id());
                all_glyphs.push(glyph);
            }
        }
        max_x = max_x.max(line_x);

        TextLayout {
            glyphs : all_glyphs,
            bounds : Rect::new(0.0, 0.0, max_x, line_y - ascent + line_height),
            dpi_scale
        }
    }

    pub fn line_height(arx_font : &ArxFont, size : FontSize, dpi_scale : f32) -> f32 {
        let font = &arx_font.font;
        let effective_size = arx_font.point_size_for(size);
        let scale = rt::Scale::uniform(((effective_size * 4) as f32 / 3.0) * dpi_scale);
        let vmetrics = font.v_metrics(scale);
            (vmetrics.ascent - vmetrics.descent + vmetrics.line_gap) * arx_font.font_info.line_height_multiplier
    }

}