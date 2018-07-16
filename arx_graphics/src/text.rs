use rusttype as rt;

use common::Rect;
use common::prelude::*;

pub type RTFont = rt::Font<'static>;
pub type RTPositionedGlyph = rt::PositionedGlyph<'static>;


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

    pub fn layout_text<'a,'b>(string : &'a str, font : &'b RTFont, size : u32, dpi_scale : f32) -> TextLayout {
        let scale = rt::Scale::uniform(((size * 4) as f32 / 3.0) * dpi_scale);
        let vmetrics = font.v_metrics(scale);
        let line_height = TextLayout::line_height(font, size, dpi_scale);

        let mut all_glyphs : Vec<RTPositionedGlyph> = Vec::new();
        let mut max_x : f32 = 0.0;
        let lines = string.split('\n').collect_vec();
        let max_y = lines.len() as f32 * line_height;
        let mut line_y = vmetrics.ascent;
        for line in &lines {
            all_glyphs.extend(
                font.layout(line, scale, rt::Point { x : 0.0, y : line_y})
                    .map(|g| g.standalone()));
            line_y += line_height;
            if let Some(last) = all_glyphs.last() {
                max_x = max_x.max(last.pixel_bounding_box().map(|p| p.max.x).unwrap_or(0) as f32);
            }
        }
        TextLayout::new(all_glyphs, Rect::new(0.0, 0.0, max_x, max_y), dpi_scale)
    }

    pub fn line_height(font : &RTFont, size : u32, dpi_scale : f32) -> f32 {
        let scale = rt::Scale::uniform(((size * 4) as f32 / 3.0) * dpi_scale);
        let vmetrics = font.v_metrics(scale);
        vmetrics.ascent - vmetrics.descent + vmetrics.line_gap
    }

}