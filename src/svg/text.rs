use harfrust::{Direction, FontRef, UnicodeBuffer, script};
use kurbo::{BezPath, PathEl, Point};
use svg::node::element::{Group, Path as SvgPath};
use ttf_parser::{Face, GlyphId, OutlineBuilder as TtfOutlineBuilder};

use crate::error::{BadgerError, BadgerResult};

pub const FONT_SIZE: f32 = 20.0;

struct KurboOutlineBuilder {
    path: BezPath,
    scale: f32,
    x_offset: f32,
    y_offset: f32,
}

impl KurboOutlineBuilder {
    fn new(scale: f32, x_offset: f32, y_offset: f32) -> Self {
        Self {
            path: BezPath::new(),
            scale,
            x_offset,
            y_offset,
        }
    }

    fn finish(self) -> BezPath {
        self.path
    }

    fn scaled_point(&self, x: f32, y: f32) -> Point {
        Point::new(
            ((x * self.scale) + self.x_offset) as f64,
            ((-y * self.scale) + self.y_offset) as f64,
        )
    }
}

impl TtfOutlineBuilder for KurboOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(self.scaled_point(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(self.scaled_point(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.path
            .quad_to(self.scaled_point(x1, y1), self.scaled_point(x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.path.curve_to(
            self.scaled_point(x1, y1),
            self.scaled_point(x2, y2),
            self.scaled_point(x, y),
        );
    }

    fn close(&mut self) {
        self.path.close_path();
    }
}

fn bezpath_to_svg_d(path: &BezPath) -> String {
    let mut d = String::new();
    for el in path.iter() {
        match el {
            PathEl::MoveTo(p) => d.push_str(&format!("M{:.2},{:.2}", p.x, p.y)),
            PathEl::LineTo(p) => d.push_str(&format!("L{:.2},{:.2}", p.x, p.y)),
            PathEl::QuadTo(ctrl, to) => d.push_str(&format!(
                "Q{:.2},{:.2} {:.2},{:.2}",
                ctrl.x, ctrl.y, to.x, to.y
            )),
            PathEl::CurveTo(ctrl1, ctrl2, to) => d.push_str(&format!(
                "C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
                ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
            )),
            PathEl::ClosePath => d.push('Z'),
        }
    }
    d
}

pub fn text_to_svg_paths(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    fill_color: &str,
) -> BadgerResult<(Group, f32)> {
    let font_data = include_bytes!(
        "/Users/philocalyst/Library/Fonts/HomeManager/truetype/Charis-BoldItalic.ttf"
    );

    let face = Face::parse(font_data, 0).map_err(|e| BadgerError::FontParse(e.to_string()))?;
    let font = FontRef::new(font_data).map_err(|e| BadgerError::FontParse(e.to_string()))?;

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.set_script(script::LATIN);

    let shaper_data = harfrust::ShaperData::new(&font);
    let shaper = shaper_data.shaper(&font).build();

    let output = shaper.shape(buffer, &[]);
    let glyph_infos = output.glyph_infos();
    let glyph_positions = output.glyph_positions();

    let units_per_em = face.units_per_em() as f32;
    let scale = size / units_per_em;

    let mut text_group = Group::new().set("fill", fill_color);
    let mut cursor_x = x;

    for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
        let glyph_id = GlyphId(info.glyph_id as u16);

        let mut builder = KurboOutlineBuilder::new(
            scale,
            cursor_x + pos.x_offset as f32 * scale,
            y + pos.y_offset as f32 * scale,
        );

        if face.outline_glyph(glyph_id, &mut builder).is_some() {
            let path = builder.finish();
            let path_data = bezpath_to_svg_d(&path);

            if !path_data.is_empty() {
                let svg_path = SvgPath::new()
                    .set("d", path_data)
                    .set("filter", format!("url(#{})", "outlineBehindFilter"));
                text_group = text_group.add(svg_path);
            }
        }

        cursor_x += pos.x_advance as f32 * scale;
    }

    Ok((text_group, cursor_x))
}
