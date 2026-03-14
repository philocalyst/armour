use css_style::unit::{em, px};
use svg::Document;
use svg::node::Text as TextNode;
use svg::node::element::{ClipPath, Definitions, Image, Rectangle, Title};
use rand::Rng;
use tracing::{debug, instrument};

use crate::colors::COLORS;
use crate::error::{ArmourError, ArmourResult};

use super::filters::{create_nnnoise_filter, create_speckle_filter, create_text_outline};
use super::text::{FONT_SIZE, text_to_svg_paths};
use super::voronoi::create_speckle_group;

#[derive(Clone, Default)]
pub struct BadgerOptions {
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub label: Option<String>,
    pub status: String,
    pub icon: Option<String>,
    pub scale: Option<f64>,
}

fn create_accessible_text(label: &str, status: &str) -> String {
    format!("{label}: {status}")
}

#[instrument(skip_all, fields(status = %options.status))]
pub fn badgen(options: BadgerOptions) -> ArmourResult<Document> {
    if options.status.is_empty() {
        return Err(ArmourError::Svg("<status> must be non-empty string".into()));
    }

    let label = options.label;

    if label.is_none() {
        return bare(BadgerOptions {
            status: options.status,
            primary_color: options.primary_color,
            scale: options.scale,
            ..Default::default()
        });
    }

    // Safe: we just checked `is_none()` above and returned early
    let label = label.ok_or_else(|| ArmourError::Svg("label unexpectedly None".into()))?;
    let status = options.status;

    let color_presets = &COLORS;

    let status_background_color = options
        .secondary_color
        .and_then(|c| color_presets.get(c.as_str()))
        .unwrap_or(&"#60AB92");

    let label_background_color = options
        .primary_color
        .and_then(|c| color_presets.get(c.as_str()))
        .unwrap_or(&"#150E5C");

    let icon_width = 30.0;
    let _scale = options.scale.unwrap_or(1.0);
    let icon_right_margin = 10.0;
    let height = FONT_SIZE * 1.2;

    let icon_span_width = if options.icon.is_some() {
        icon_width + icon_right_margin
    } else {
        0.0
    };

    let accessible_text = create_accessible_text(&label, &status);
    debug!(accessible_text, "building labeled badge");

    let mut document = Document::new()
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("role", "img")
        .set("aria-label", accessible_text.clone());

    if options.icon.is_some() {
        document = document.set("xmlns:xlink", "http://www.w3.org/1999/xlink");
    }

    document = document.add(Title::new(accessible_text));

    if let Some(icon) = options.icon {
        let image = Image::new()
            .set("x", 0)
            .set("y", 40)
            .set("width", icon_width)
            .set("xlink:href", icon);

        document = document.add(image);
    }

    let spacer: f32 = FONT_SIZE * 0.2;
    let label_start = icon_span_width + spacer;
    let baseline = height * 0.80;

    let (label_paths, label_end) =
        text_to_svg_paths(&label, label_start, baseline, FONT_SIZE, "#FFB4BB")?;

    let status_start = label_end + (spacer * 2.0);
    let (status_paths, status_end) =
        text_to_svg_paths(&status, status_start, baseline, FONT_SIZE, "#F5ECEB")?;

    let label_width = label_end + ((status_start - label_end) / 2.0);
    let status_width = status_end - status_start + (spacer / 2.0);
    let total_width: f32 = label_width + status_width + spacer;

    let mut rng = rand::rng();
    let seed: u32 = rng.random();

    let label_bg = create_speckle_group(
        0.0,
        label_width,
        height,
        label_background_color,
        "clipLabel",
        "ssspot-filter",
        &mut rng,
    );
    let status_bg = create_speckle_group(
        label_width,
        status_width + spacer,
        height,
        status_background_color,
        "clipStatus",
        "ssspot-filter",
        &mut rng,
    );

    let total_width_normalized = total_width / 16.0;
    let height_normalized = height / 16.0;

    let text_outline = create_text_outline();
    let noise = create_nnnoise_filter("nnoise");
    let speckle = create_speckle_filter("ssspot-filter", seed, height);

    let clip_label = ClipPath::new().set("id", "clipLabel").add(
        Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", label_width)
            .set("height", height),
    );
    let clip_status = ClipPath::new().set("id", "clipStatus").add(
        Rectangle::new()
            .set("x", label_width)
            .set("y", 0)
            .set("width", status_width + spacer)
            .set("height", height),
    );

    let defs = Definitions::new()
        .add(text_outline)
        .add(noise)
        .add(speckle)
        .add(clip_label)
        .add(clip_status);

    document = document.set("filter", format!("url(#{})", "nnoise"));

    document = document.add(defs);
    document = document.add(label_bg).add(status_bg);
    document = document.set("viewBox", format!("0 0 {total_width} {height}"));
    document = document.add(label_paths).add(status_paths);

    let style = css_style::style()
        .and_size(|conf| {
            conf.height(em(height_normalized))
                .width(em(total_width_normalized))
        })
        .and_border(|conf| conf.radius(px(10)));

    let style = format!(r#"svg {{{style}}}"#);
    document = document.add(svg::node::element::Style::new(style));

    let output = format!("{document:#}");
    let output = output.replace("\n", "");
    std::fs::write("./test.svg", output)?;

    Ok(document)
}

#[instrument(skip_all, fields(status = %options.status))]
pub fn bare(options: BadgerOptions) -> ArmourResult<Document> {
    let color_presets = &COLORS;
    let color = options
        .primary_color
        .as_ref()
        .and_then(|c| color_presets.get(c.as_str()))
        .ok_or_else(|| ArmourError::Config("no valid primary color for bare badge".into()))?;

    let scale = options.scale.unwrap_or(1.0);
    let st_rect_width = 1.0 + 115.0;
    let sanitized_status = &options.status;

    debug!("building bare badge");

    let mut document = Document::new()
        .set("width", scale * st_rect_width / 10.0)
        .set("height", scale * 20.0)
        .set("viewBox", format!("0 0 {st_rect_width} 200"))
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("role", "img")
        .set("aria-label", sanitized_status.clone());

    document = document.add(Title::new("").add(TextNode::new(sanitized_status.clone())));

    Ok(document)
}
