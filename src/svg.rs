use css_style::unit::{em, px};
use rustybuzz::{Direction, Face, Language, Script, UnicodeBuffer, script};
use ttf_parser::GlyphId;
// use css_style;
use crate::colors::COLORS;
use std::error::Error;
use svg::Document;
use svg::node::Text as TextNode;
use svg::node::element::{Group, Image, Path as SvgPath, Rectangle, Text, Title};

use lyon::math::{Point, point};
use lyon::path::Event;
use lyon::path::{Path as LyonPath, builder::*};
use ttf_parser::OutlineBuilder as TtfOutlineBuilder;

const TEXT_HEIGHT: f32 = 20.0;

#[derive(Clone)]

pub struct BadgerOptions {
    pub status: String,               // The "right side" of the k/v THIS IS NEEDED!!
    pub status_color: Option<String>, // A color override on the default status color (blue)
    pub label: Option<String>,        // The "left side" of the k/v, describing the status
    pub label_color: Option<String>,  // A color override of the default status color (gray)
    pub icon: Option<String>,         // A name of a supported icon
    pub scale: Option<f64>,           // The scale of the entire badge
}

fn calc_width(text: &str, size: f32) -> Result<f32, Box<dyn Error>> {
    let font_data = include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
    let face = Face::from_slice(font_data, 0).ok_or("Failed to parse font")?;

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.set_script(script::LATIN);

    let output = rustybuzz::shape(&face, &[], buffer);
    let glyph_positions = output.glyph_positions();

    let mut total_width = 0.0;
    for pos in glyph_positions {
        let width = pos.x_advance as f32 / 64.0;
        // Apply the original scaling factor
        total_width += width * 0.0295 * (size / 10.0); // Adjust scaling
    }

    Ok(total_width)
}

// Struct to implement ttf_parser's OutlineBuilder, building a lyon path
struct LyonOutlineBuilder {
    builder: lyon::path::Builder,
    scale: f32,
    x_offset: f32,
    y_offset: f32,
}

impl LyonOutlineBuilder {
    fn new(scale: f32, x_offset: f32, y_offset: f32) -> Self {
        Self {
            builder: LyonPath::builder(),
            scale,
            x_offset,
            y_offset,
        }
    }

    fn finish(self) -> LyonPath {
        self.builder.build()
    }

    fn scaled_point(&self, x: f32, y: f32) -> Point {
        // Scale and flip Y for SVG (glyph Y is positive up, SVG positive down)
        point(
            (x * self.scale) + self.x_offset,
            (-y * self.scale) + self.y_offset, // Flip Y
        )
    }
}

impl TtfOutlineBuilder for LyonOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.begin(self.scaled_point(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(self.scaled_point(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.builder
            .quadratic_bezier_to(self.scaled_point(x1, y1), self.scaled_point(x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.builder.cubic_bezier_to(
            self.scaled_point(x1, y1),
            self.scaled_point(x2, y2),
            self.scaled_point(x, y),
        );
    }

    fn close(&mut self) {
        self.builder.close();
    }
}

// Function to convert lyon Path to SVG 'd' string
fn lyon_path_to_svg_d(path: &LyonPath) -> String {
    let mut d = String::new();
    for event in path {
        match event {
            Event::Begin { at } => d.push_str(&format!("M{:.2},{:.2}", at.x, at.y)),
            Event::Line { from: _, to } => d.push_str(&format!("L{:.2},{:.2}", to.x, to.y)),
            Event::Quadratic { from: _, ctrl, to } => d.push_str(&format!(
                "Q{:.2},{:.2} {:.2},{:.2}",
                ctrl.x, ctrl.y, to.x, to.y
            )),
            Event::Cubic {
                from: _,
                ctrl1,
                ctrl2,
                to,
            } => d.push_str(&format!(
                "C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
                ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
            )),
            Event::End {
                last: _,
                first: _,
                close,
            } => {
                if close {
                    d.push('Z');
                }
            }
        }
    }
    d
}

// Core function: Convert text to SVG paths using shaping and outlining
fn text_to_svg_paths(
    text: &str,
    x: f32,
    y: f32, // Baseline y-position
    size: f32,
    fill_color: &str,
) -> Result<(Group, f32), Box<dyn Error>> {
    let font_data = include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
    let face = Face::from_slice(font_data, 0).ok_or("Failed to parse font")?;

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.set_script(script::LATIN); // Customize for script if needed

    let output = rustybuzz::shape(&face, &[], buffer);
    let glyph_infos = output.glyph_infos();
    let glyph_positions = output.glyph_positions();

    let units_per_em = face.units_per_em() as f32;
    let scale = size / units_per_em;

    let mut text_group = Group::new().set("fill", fill_color);

    let mut cursor_x = x;

    for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
        let glyph_id = GlyphId(info.glyph_id as u16);

        let mut builder = LyonOutlineBuilder::new(
            scale,
            cursor_x + pos.x_offset as f32 * scale,
            y + pos.y_offset as f32 * scale,
        );

        if face.outline_glyph(glyph_id, &mut builder).is_some() {
            let path = builder.finish();
            let path_data = lyon_path_to_svg_d(&path);

            if !path_data.is_empty() {
                let svg_path = SvgPath::new().set("d", path_data);
                text_group = text_group.add(svg_path);
            }
        }

        // Advance cursor
        cursor_x += pos.x_advance as f32 * scale;
    }

    Ok((text_group, cursor_x))
}

fn create_accessible_text(label: &str, status: &str) -> String {
    format!("{}: {}", label, status)
}

pub fn badgen(options: BadgerOptions) -> Result<Document, Box<dyn Error>> {
    // We need at least a status
    if options.status.is_empty() {
        return Err("<status> must be non-empty string".into());
    }

    let label = options.label;

    // Check for the case where a label isn't specified, and pipe
    // to a specific styling for that particular use
    if label.is_none() {
        return bare(BadgerOptions {
            status: options.status,
            label_color: options.label_color,
            scale: options.scale,
            ..Default::default()
        });
    }

    let label = label.expect("If it was none bare would have handled it by now");
    let status = options.status;

    let color_presets = &COLORS;

    let status_background_color = options
        .status_color // Fixed: was label_color
        .and_then(|c| color_presets.get(c.as_str()))
        .unwrap_or(&"blue"); // Fallback color is blue (corrected from your code)

    let label_background_color = options
        .label_color // Fixed: was status_color
        .and_then(|c| color_presets.get(c.as_str()))
        .unwrap_or(&"SLATEGRAY"); // Fallback color is slate gray

    let icon_width = 30.0; // How large an icon is (the height will be capped though)
    let _scale = options.scale.unwrap_or(1.0);
    let icon_right_margin = 10.0;

    let label_text_width = calc_width(&label, TEXT_HEIGHT)?;
    let status_text_width = calc_width(&status, TEXT_HEIGHT)?;

    let icon_span_width = if options.icon.is_some() {
        icon_width + icon_right_margin // Icon width + some right margin
    } else {
        0.0 // No icon no problem
    };

    const SPACER: f32 = 10.0;

    let accessible_text = create_accessible_text(&label, &status);

    // Create boilerplate svg shell
    let mut document = Document::new()
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("role", "img") // The badge is functionally an image
        .set("aria-label", accessible_text.clone()); // We label it the status..

    if options.icon.is_some() {
        document = document.set("xmlns:xlink", "http://www.w3.org/1999/xlink");
    }

    // Add title
    document = document.add(Title::new(accessible_text));

    // Add icon if present
    if let Some(icon) = options.icon {
        let image = Image::new()
            .set("x", 0)
            .set("y", 40)
            .set("width", icon_width)
            .set("xlink:href", icon);

        document = document.add(image);
    }

    // We're putting the label right after the icon_span
    let label_text_begin: f32 = icon_span_width;
    let status_text_begin: f32 = label_text_begin + label_text_width;

    const MARGIN_SMALL: f32 = 10.0;

    let label_width = label_text_width + icon_span_width + MARGIN_SMALL;
    let status_width = status_text_width + MARGIN_SMALL;

    let (label_paths, label_end) = text_to_svg_paths(
        &label,
        label_text_begin + 5.0,
        TEXT_HEIGHT * 0.8,
        TEXT_HEIGHT,
        "#fff",
    )?;
    let (status_paths, status_end) = text_to_svg_paths(
        &status,
        status_text_begin + 5.0,
        TEXT_HEIGHT * 0.8,
        TEXT_HEIGHT,
        "#fff",
    )?;

    let bg_group = Group::new()
        .add(
            Rectangle::new()
                .set("fill", label_background_color.to_string())
                .set("width", label_end) // Margin to space out the distance between this and the edges
                .set("height", (TEXT_HEIGHT * 1.2) as i32),
        )
        .add(
            Rectangle::new()
                .set("fill", status_background_color.to_string())
                .set("x", label_end)
                .set("width", status_end)
                .set("height", (TEXT_HEIGHT * 1.2) as i32),
        );

    document = document.add(bg_group);

    document = document.add(label_paths).add(status_paths);

    // Styling
    let total_width = icon_span_width + label_text_width * 1.1 + status_width + SPACER;
    let style = css_style::style()
        .and_size(|conf| conf.max_width(px(total_width as i32)))
        .and_border(|conf| conf.radius(px(20)));

    let style = format!(r#"svg {{{}}}"#, style);
    document = document.add(svg::node::element::Style::new(style));

    // For testing/output (unchanged)
    let output = format!("{:#}", document);
    let output = output.replace("\n", "");
    use std::fs;
    fs::write("./test.svg", output)?;

    Ok(document)
}

pub fn bare(options: BadgerOptions) -> Result<Document, Box<dyn Error>> {
    let color_presets = &COLORS;
    let color = options
        .label_color
        .as_ref()
        .and_then(|c| color_presets.get(c.as_str()))
        .unwrap();

    let scale = options.scale.unwrap_or(1.0);

    let st_text_width = calc_width(&options.status, 0.0)?;
    let st_rect_width = st_text_width + 115.0;

    let sanitized_status = &options.status;

    // Create boilerplate svg shell
    let mut document = Document::new()
        .set("width", scale * st_rect_width as f64 / 10.0)
        .set("height", scale * 20.0)
        .set("viewBox", format!("0 0 {} 200", st_rect_width))
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("role", "img")
        .set("aria-label", sanitized_status.clone());

    document = document.add(Title::new("").add(TextNode::new(sanitized_status.clone())));

    Ok(document)
}

impl Default for BadgerOptions {
    fn default() -> Self {
        Self {
            status: String::new(),
            label: None,
            label_color: None,
            status_color: None,
            icon: None,
            scale: None,
        }
    }
}

// match style {
//     StyleOption::Flat => {
//         // Background rectangles
//         let bg_group = Group::new()
//             .add(
//                 Rectangle::new()
//                     .set("fill", format!("#{}", label_color))
//                     .set("width", sb_rect_width)
//                     .set("height", 200),
//             )
//             .add(
//                 Rectangle::new()
//                     .set("fill", format!("#{}", status_color))
//                     .set("x", sb_rect_width)
//                     .set("width", st_rect_width)
//                     .set("height", 200),
//             );

//         document = document.add(bg_group);

//         // Text group
//         let mut text_group = Group::new()
//             .set("aria-hidden", "true")
//             .set("fill", "#fff")
//             .set("text-anchor", "start")
//             .set("font-family", "Verdana,DejaVu Sans,sans-serif")
//             .set("font-size", "110");

//         if !sanitized_label.is_empty() {
//             text_group = text_group
//                 .add(
//                     Text::new("")
//                         .set("x", sb_text_start + 10.0)
//                         .set("y", 148)
//                         .set("textLength", sb_text_width)
//                         .set("fill", "#000")
//                         .set("opacity", "0.1")
//                         .add(TextNode::new(sanitized_label.clone())),
//                 )
//                 .add(
//                     Text::new("")
//                         .set("x", sb_text_start)
//                         .set("y", 138)
//                         .set("textLength", sb_text_width)
//                         .add(TextNode::new(sanitized_label)),
//                 );
//         }

//         text_group = text_group
//             .add(
//                 Text::new("")
//                     .set("x", sb_rect_width + 55.0)
//                     .set("y", 148)
//                     .set("textLength", st_text_width)
//                     .set("fill", "#000")
//                     .set("opacity", "0.1")
//                     .add(TextNode::new(sanitized_status.clone())),
//             )
//             .add(
//                 Text::new("")
//                     .set("x", sb_rect_width + 45.0)
//                     .set("y", 138)
//                     .set("textLength", st_text_width)
//                     .add(TextNode::new(sanitized_status)),
//             );

//         document = document.add(text_group);
//     }
//     StyleOption::Classic => {
//         let gradient_id = generate_random_id(5);
//         let mask_id = generate_random_id(5);

//         // Add definitions
//         let mut defs = Definitions::new();

//         let gradient = LinearGradient::new()
//             .set("id", gradient_id.clone())
//             .set("x2", "0")
//             .set("y2", "100%")
//             .add(
//                 Stop::new()
//                     .set("offset", "0")
//                     .set("stop-opacity", ".1")
//                     .set("stop-color", "#EEE"),
//             )
//             .add(Stop::new().set("offset", "1").set("stop-opacity", ".1"));

//         let mask = Mask::new().set("id", mask_id.clone()).add(
//             Rectangle::new()
//                 .set("width", width)
//                 .set("height", 200)
//                 .set("rx", 30)
//                 .set("fill", "#FFF"),
//         );

//         defs = defs.add(gradient).add(mask);
//         document = document.add(defs);

//         // Masked group
//         let masked_group = Group::new()
//             .set("mask", format!("url(#{})", mask_id))
//             .add(
//                 Rectangle::new()
//                     .set("width", sb_rect_width)
//                     .set("height", 200)
//                     .set("fill", format!("#{}", label_color)),
//             )
//             .add(
//                 Rectangle::new()
//                     .set("width", st_rect_width)
//                     .set("height", 200)
//                     .set("fill", format!("#{}", status_color))
//                     .set("x", sb_rect_width),
//             )
//             .add(
//                 Rectangle::new()
//                     .set("width", width)
//                     .set("height", 200)
//                     .set("fill", format!("url(#{})", gradient_id)),
//             );

//         document = document.add(masked_group);

//         // Text group (similar to flat but with different opacity)
//         let mut text_group = Group::new()
//             .set("aria-hidden", "true")
//             .set("fill", "#fff")
//             .set("text-anchor", "start")
//             .set("font-family", "Verdana,DejaVu Sans,sans-serif")
//             .set("font-size", "110");

//         if !sanitized_label.is_empty() {
//             text_group = text_group
//                 .add(
//                     Text::new("")
//                         .set("x", sb_text_start + 10.0)
//                         .set("y", 148)
//                         .set("textLength", sb_text_width)
//                         .set("fill", "#000")
//                         .set("opacity", "0.25")
//                         .add(TextNode::new(sanitized_label.clone())),
//                 )
//                 .add(
//                     Text::new("")
//                         .set("x", sb_text_start)
//                         .set("y", 138)
//                         .set("textLength", sb_text_width)
//                         .add(TextNode::new(sanitized_label)),
//                 );
//         }

//         text_group = text_group
//             .add(
//                 Text::new("")
//                     .set("x", sb_rect_width + 55.0)
//                     .set("y", 148)
//                     .set("textLength", st_text_width)
//                     .set("fill", "#000")
//                     .set("opacity", "0.25")
//                     .add(TextNode::new(sanitized_status.clone())),
//             )
//             .add(
//                 Text::new("")
//                     .set("x", sb_rect_width + 45.0)
//                     .set("y", 138)
//                     .set("textLength", st_text_width)
//                     .add(TextNode::new(sanitized_status)),
//             );

//         document = document.add(text_group);
//     }
// }
//
// //     match style {
//         StyleOption::Flat => {
//             let bg_group = Group::new().add(
//                 Rectangle::new()
//                     .set("fill", format!("#{}", color))
//                     .set("x", 0)
//                     .set("width", st_rect_width)
//                     .set("height", 200),
//             );

//             let text_group = Group::new()
//                 .set("aria-hidden", "true")
//                 .set("fill", "#fff")
//                 .set("text-anchor", "start")
//                 .set("font-family", "Verdana,DejaVu Sans,sans-serif")
//                 .set("font-size", "110")
//                 .add(
//                     Text::new("")
//                         .set("x", 65)
//                         .set("y", 148)
//                         .set("textLength", st_text_width)
//                         .set("fill", "#000")
//                         .set("opacity", "0.1")
//                         .add(TextNode::new(sanitized_status.clone())),
//                 )
//                 .add(
//                     Text::new("")
//                         .set("x", 55)
//                         .set("y", 138)
//                         .set("textLength", st_text_width)
//                         .add(TextNode::new(sanitized_status)),
//                 );

//             document = document.add(bg_group).add(text_group);
//         }
//         StyleOption::Classic => {
//             let gradient_id = generate_random_id(5);
//             let mask_id = generate_random_id(5);

//             let mut defs = Definitions::new();

//             let gradient = LinearGradient::new()
//                 .set("id", gradient_id.clone())
//                 .set("x2", "0")
//                 .set("y2", "100%")
//                 .add(
//                     Stop::new()
//                         .set("offset", "0")
//                         .set("stop-opacity", ".1")
//                         .set("stop-color", "#EEE"),
//                 )
//                 .add(Stop::new().set("offset", "1").set("stop-opacity", ".1"));

//             let mask = Mask::new().set("id", mask_id.clone()).add(
//                 Rectangle::new()
//                     .set("width", st_rect_width)
//                     .set("height", 200)
//                     .set("rx", 30)
//                     .set("fill", "#FFF"),
//             );

//             defs = defs.add(gradient).add(mask);
//             document = document.add(defs);

//             let masked_group = Group::new()
//                 .set("mask", format!("url(#{})", mask_id))
//                 .add(
//                     Rectangle::new()
//                         .set("width", st_rect_width)
//                         .set("height", 200)
//                         .set("fill", format!("#{}", color))
//                         .set("x", 0),
//                 )
//                 .add(
//                     Rectangle::new()
//                         .set("width", st_rect_width)
//                         .set("height", 200)
//                         .set("fill", format!("url(#{})", gradient_id)),
//                 );

//             let text_group = Group::new()
//                 .set("aria-hidden", "true")
//                 .set("fill", "#fff")
//                 .set("text-anchor", "start")
//                 .set("font-family", "Verdana,DejaVu Sans,sans-serif")
//                 .set("font-size", "110")
//                 .add(
//                     Text::new("")
//                         .set("x", 65)
//                         .set("y", 148)
//                         .set("textLength", st_text_width)
//                         .set("fill", "#000")
//                         .set("opacity", "0.25")
//                         .add(TextNode::new(sanitized_status.clone())),
//                 )
//                 .add(
//                     Text::new("")
//                         .set("x", 55)
//                         .set("y", 138)
//                         .set("textLength", st_text_width)
//                         .add(TextNode::new(sanitized_status)),
//                 );

//             document = document.add(masked_group).add(text_group);
//         }
//     }
