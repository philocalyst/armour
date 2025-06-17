use ab_glyph::{Font, FontRef};
// use css_style;
use fontdb;
use std::collections::HashMap;
use std::error::Error;
use svg::node::element::{
    Definitions, Group, Image, LinearGradient, Mask, Rectangle, Stop, Text, Title,
};
use svg::node::Text as TextNode;
use svg::Document;

use crate::colors::COLORS;

#[derive(Clone)]
pub enum StyleOption {
    Flat,
    Classic,
}

pub struct BadgerOptions {
    pub status: String,               // The "right side" of the k/v THIS IS NEEDED!!
    pub status_color: Option<String>, // A color override on the default status color (blue)
    pub label: Option<String>,        // The "left side" of the k/v, describing the status
    pub label_color: Option<String>,  // A color override of the default status color (gray)
    pub icon: Option<String>,         // A name of a supported icon
    pub scale: Option<f64>,           // The scale of the entire badge
}

// Placeholder for text width calculation
fn calc_width(text: &str) -> Result<f32, Box<dyn Error>> {
    let font = FontRef::try_from_slice(include_bytes!(
        "/home/jack/.local/share/fonts/FiraCodeNerdFont-Regular.ttf" // "/Users/philocalyst/Library/Fonts/HackNerdFont-Regular.ttf"
    ))?;

    // Get the total width of the entire string
    // We don't need to worry about newlines here because they
    // SHOULDNT EXIST
    Ok(text
        .chars()
        .into_iter()
        .map(|c| {
            // TODO: Allow font size to be configurable
            let glyph = font.glyph_id(c).with_scale(30.0);

            // Getting the outline of the precise glyph rather than just its bounding box
            let outline = font.outline_glyph(glyph).unwrap();

            outline.px_bounds().width()
        })
        .sum())
}

fn generate_random_id(length: usize) -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect()
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

    let status_color = options
        .label_color
        .and_then(|c| color_presets.get(c.as_str()))
        .unwrap_or(&"black"); // Fallback color is black

    let label_color = options
        .status_color
        .and_then(|c| color_presets.get(c.as_str()))
        .unwrap_or(&"white"); // Fallback color is white

    let icon_width = 130.0;
    let scale = options.scale.unwrap_or(1.0);
    let icon_right_margin = 30.0;
    let text_starting_position = 50.0;

    let icon_span_width = if options.icon.is_some() {
        icon_width + icon_right_margin // Icon width + some right margin
    } else {
        0.0 // No icon no problem
    };

    // Handle the starting position with an icon
    let status_text_begin = if options.icon.is_some() {
        icon_span_width + text_starting_position
    } else {
        text_starting_position
    };

    const SPACER: f32 = 100.0;

    // We're not worrying about height here because it's largely constant.
    let label_width = calc_width(&label)?;
    let status_width = calc_width(&status)?;
    let label_box_width = label_width + SPACER + icon_span_width; // The container for the label final width
    let status_box_width = status_width + SPACER; // The container for the status final width
    let width = label_box_width + status_box_width; // The TOTAL width of both

    let accessible_text = create_accessible_text(&label, &status);

    // Create boilerplate svg shell
    let mut document = Document::new()
        .set("width", scale * width as f64 / 10.0)
        .set("height", scale * 20.0)
        .set("viewBox", format!("0 0 {} 200", width))
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("role", "img") // The badge is functionally an image
        .set("aria-label", accessible_text.clone()); // We label it the status..

    if options.icon.is_some() {
        document = document.set("xmlns:xlink", "http://www.w3.org/1999/xlink");
    }

    // Add title
    document = document.add(Title::new("").add(TextNode::new(accessible_text)));

    // Add icon if present
    if let Some(icon) = options.icon {
        let image = Image::new()
            .set("x", 40)
            .set("y", 35)
            .set("width", icon_width)
            .set("xlink:href", icon);

        document = document.add(image);
    }

    let bg_group = Group::new()
        .add(
            Rectangle::new()
                .set("fill", format!("#{}", label_color))
                .set("width", label_box_width)
                .set("height", 200),
        )
        .add(
            Rectangle::new()
                .set("fill", format!("#{}", status_color))
                .set("x", label_box_width)
                .set("width", status_box_width)
                .set("height", 200),
        );

    document = document.add(bg_group);

    // Text group
    let mut text_group = Group::new()
        .set("aria-hidden", "true")
        .set("fill", "#fff")
        .set("text-anchor", "start")
        .set("font-family", "Verdana,DejaVu Sans,sans-serif")
        .set("font-size", "110");

    text_group = text_group
        .add(
            Text::new("")
                .set("x", label_box_width + 55.0)
                .set("y", 148)
                .set("textLength", status_box_width)
                .set("fill", "#000")
                .set("opacity", "0.1")
                .add(TextNode::new(status.clone())),
        )
        .add(
            Text::new("")
                .set("x", label_box_width + 45.0)
                .set("y", 138)
                .set("textLength", status_box_width)
                .add(TextNode::new(status)),
        );

    document = document.add(text_group);

    println!("{:#}", document);

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

    let st_text_width = calc_width(&options.status)?;
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
