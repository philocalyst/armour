use css_style::unit::{em, px};
use harfrust::{Direction, FontRef, UnicodeBuffer, script};
use ttf_parser::{Face, GlyphId};

// use css_style;
use crate::colors::COLORS;
use std::error::Error;
use svg::Document;
use svg::node::Text as TextNode;
use svg::node::element::{
    Circle, ClipPath, Definitions, Filter, FilterEffectComposite, FilterEffectDisplacementMap,
    FilterEffectDistantLight, FilterEffectFlood, FilterEffectGaussianBlur, FilterEffectMerge,
    FilterEffectMergeNode, FilterEffectMorphology, FilterEffectOffset,
    FilterEffectSpecularLighting, FilterEffectTurbulence, Group, Image, Path as SvgPath, Rectangle,
    Title,
};

use rand::Rng;
use voronator::VoronoiDiagram;
use voronator::delaunator::Point as VoronoiPoint;

use kurbo::{BezPath, PathEl, Point};

use ttf_parser::OutlineBuilder as TtfOutlineBuilder;

const FONT_SIZE: f32 = 20.0;

#[derive(Clone, Default)]
pub struct Status {
    name: String,
    color: String,
}

#[derive(Clone, Default)]
pub struct Label {
    name: String,
    color: String,
}

#[derive(Clone, Default)]
pub struct BadgerOptions {
    pub status: Status,       // The "right side" of the k/v THIS IS NEEDED!!
    pub label: Option<Label>, // The "left side" of the k/v, describing the status
    pub icon: Option<String>, // A name of a supported icon
    pub scale: Option<f64>,   // The scale of the entire badge
}

// Struct to implement ttf_parser's OutlineBuilder, building a kurbo path
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
        // Scale and flip Y for SVG (glyph Y is positive up, SVG positive down)
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

// Function to convert kurbo BezPath to SVG 'd' string
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

// Core function: Convert text to SVG paths using shaping and outlining
fn text_to_svg_paths(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    fill_color: &str,
) -> Result<(Group, f32), Box<dyn Error>> {
    let font_data = include_bytes!(
        "/Users/philocalyst/Library/Fonts/HomeManager/truetype/Charis-BoldItalic.ttf"
    );

    // Parse font with ttf-parser
    let face = Face::parse(font_data, 0)?;

    let font = FontRef::new(font_data)?;

    // Create HarfRüst shaper
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.set_script(script::LATIN);

    let shaper_data = harfrust::ShaperData::new(&font); // Pass raw bytes
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

        // Use ttf-parser's outline_glyph
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

fn create_accessible_text(label: &str, status: &str) -> String {
    format!("{label}: {status}")
}

fn create_text_outline() -> Result<Filter, Box<dyn Error>> {
    let filter_id = "outlineBehindFilter".to_string();

    let morphology = FilterEffectMorphology::new()
        .set("in", "SourceAlpha".to_string())
        .set("operator", "dilate")
        .set("radius", 0.31)
        .set("result", "dilated".to_string());

    let offset = FilterEffectOffset::new()
        .set("in", "dilated".to_string())
        .set("dx", -0.21)
        .set("dy", 0.41)
        .set("result", "offsetOutline".to_string());

    // feFlood: Create the outlne color
    let flood = FilterEffectFlood::new()
        .set("flood-color", "#FF0000")
        .set("result", "outlineColor".to_string());

    // feComposite: Combine the flood color with the dilated shape
    let composite = FilterEffectComposite::new()
        .set("in", "outlineColor".to_string())
        .set("in2", "offsetOutline".to_string())
        .set("operator", "in")
        .set("result", "outline".to_string());

    // feMerge: Merge the outline (behind) and the source graphic (on top)
    let merge = FilterEffectMerge::new()
        .add(FilterEffectMergeNode::new().set("in", "outline")) // Outline first
        .add(FilterEffectMergeNode::new().set("in", "SourceGraphic")); // Original graphic second

    // Create the <filter> element and add its children
    let filter = Filter::new()
        .set("id", filter_id.clone())
        .add(morphology)
        .add(offset)
        .add(flood)
        .add(composite)
        .add(merge);
    Ok(filter)
}

fn polygon_centroid(pts: &[VoronoiPoint]) -> (f64, f64) {
    let n = pts.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let (mut cx, mut cy, mut area) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (x0, y0) = (pts[i].x, pts[i].y);
        let (x1, y1) = (pts[(i + 1) % n].x, pts[(i + 1) % n].y);
        let a = x0 * y1 - x1 * y0;
        cx += (x0 + x1) * a;
        cy += (y0 + y1) * a;
        area += a;
    }
    let area = area * 3.0;
    if area.abs() < 1e-10 {
        let sx: f64 = pts.iter().map(|p| p.x).sum();
        let sy: f64 = pts.iter().map(|p| p.y).sum();
        return (sx / n as f64, sy / n as f64);
    }
    (cx / area, cy / area)
}

fn dist_to_segment(p: (f64, f64), v: (f64, f64), w: (f64, f64)) -> f64 {
    let l2 = (w.0 - v.0).powi(2) + (w.1 - v.1).powi(2);
    if l2 < 1e-10 {
        return ((p.0 - v.0).powi(2) + (p.1 - v.1).powi(2)).sqrt();
    }
    let t = ((p.0 - v.0) * (w.0 - v.0) + (p.1 - v.1) * (w.1 - v.1)) / l2;
    let t = t.clamp(0.0, 1.0);
    let proj = (v.0 + t * (w.0 - v.0), v.1 + t * (w.1 - v.1));
    ((p.0 - proj.0).powi(2) + (p.1 - proj.1).powi(2)).sqrt()
}

fn inner_circle_radius(centroid: (f64, f64), polygon: &[VoronoiPoint]) -> f64 {
    let n = polygon.len();
    if n < 2 {
        return 0.0;
    }
    let mut min_dist = f64::MAX;
    for i in 0..n {
        let v = (polygon[i].x, polygon[i].y);
        let w = (polygon[(i + 1) % n].x, polygon[(i + 1) % n].y);
        let d = dist_to_segment(centroid, v, w);
        if d < min_dist {
            min_dist = d;
        }
    }
    min_dist
}

struct VoronoiCell {
    centroid: (f64, f64),
    inner_circle_radius: f64,
}

fn create_voronoi_tessellation(
    width: f64,
    height: f64,
    num_points: usize,
    relax_iterations: usize,
    rng: &mut impl Rng,
) -> Vec<VoronoiCell> {
    let mut points: Vec<(f64, f64)> = (0..num_points)
        .map(|_| (rng.random_range(0.0..width), rng.random_range(0.0..height)))
        .collect();

    // Lloyd relaxation
    for _ in 0..relax_iterations {
        let diagram = match VoronoiDiagram::<VoronoiPoint>::from_tuple(
            &(0.0, 0.0),
            &(width, height),
            &points,
        ) {
            Some(d) => d,
            None => break,
        };

        for (i, cell) in diagram.cells().iter().enumerate() {
            if i < points.len() {
                let pts = cell.points();
                if !pts.is_empty() {
                    let c = polygon_centroid(pts);
                    points[i] = (c.0.clamp(0.0, width), c.1.clamp(0.0, height));
                }
            }
        }
    }

    // Final tessellation
    let diagram =
        match VoronoiDiagram::<VoronoiPoint>::from_tuple(&(0.0, 0.0), &(width, height), &points) {
            Some(d) => d,
            None => return Vec::new(),
        };

    diagram
        .cells()
        .iter()
        .filter_map(|cell| {
            let pts = cell.points();
            if pts.is_empty() {
                return None;
            }
            let centroid = polygon_centroid(pts);
            let icr = inner_circle_radius(centroid, pts);
            if icr < 0.1 {
                return None;
            }
            Some(VoronoiCell {
                centroid,
                inner_circle_radius: icr,
            })
        })
        .collect()
}

fn create_speckle_group(
    x_offset: f32,
    section_width: f32,
    section_height: f32,
    fill_color: &str,
    clip_id: &str,
    filter_id: &str,
    rng: &mut impl Rng,
) -> Group {
    let w = section_width as f64;
    let h = section_height as f64;
    let area = w * h;
    let num_points = ((area / 25.0) as usize).max(8);

    let cells = create_voronoi_tessellation(w, h, num_points, 2, rng);

    let mut bg = Group::new();

    // Base color rectangle
    bg = bg.add(
        Rectangle::new()
            .set("fill", fill_color.to_string())
            .set("x", x_offset)
            .set("width", section_width)
            .set("height", section_height),
    );

    // Circle speckle group with filter and clip — white circles over
    // the base color create lighter speckles (like the JS reference where
    // bgColor and circle fill are different shades of the same hue)
    let mut circle_group = Group::new()
        .set("fill", "#ffffff")
        .set("filter", format!("url(#{filter_id})"))
        .set("clip-path", format!("url(#{clip_id})"));

    for cell in &cells {
        let r_min = cell.inner_circle_radius / 2.0;
        let r_max = cell.inner_circle_radius;
        let r = rng.random_range(r_min..=r_max);
        let opacity = rng.random_range(0.05..=0.35);

        circle_group = circle_group.add(
            Circle::new()
                .set("cx", cell.centroid.0 + x_offset as f64)
                .set("cy", cell.centroid.1)
                .set("r", r)
                .set("opacity", format!("{opacity:.2}")),
        );
    }

    bg = bg.add(circle_group);
    bg
}

fn create_speckle_filter(id: &str, seed: u32, badge_height: f32) -> Filter {
    // Scale filter parameters proportionally — the JS reference uses
    // scale=32 and stdDeviation="0 3" for an 800px canvas
    let ratio = badge_height / 800.0;
    let displacement_scale = 32.0 * ratio;
    let blur_std = 3.0 * ratio;

    let fe_turbulence = FilterEffectTurbulence::new()
        .set("type", "turbulence")
        .set("baseFrequency", "0.022 0.218")
        .set("numOctaves", "2")
        .set("seed", seed)
        .set("stitchTiles", "stitch")
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("result", "turbulence");

    let fe_blur = FilterEffectGaussianBlur::new()
        .set("stdDeviation", format!("0 {blur_std:.2}"))
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("in", "turbulence")
        .set("edgeMode", "duplicate")
        .set("result", "blur");

    let fe_displacement = FilterEffectDisplacementMap::new()
        .set("in", "SourceGraphic")
        .set("in2", "blur")
        .set("scale", format!("{displacement_scale:.2}"))
        .set("xChannelSelector", "R")
        .set("yChannelSelector", "B")
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("result", "displacementMap");

    Filter::new()
        .set("id", id)
        .set("x", "-20%")
        .set("y", "-20%")
        .set("width", "140%")
        .set("height", "140%")
        .set("filterUnits", "objectBoundingBox")
        .set("primitiveUnits", "userSpaceOnUse")
        .set("color-interpolation-filters", "sRGB")
        .add(fe_turbulence)
        .add(fe_blur)
        .add(fe_displacement)
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
        .unwrap_or(&"#60AB92"); // Fallback color is blue (corrected from your code)

    let label_background_color = options
        .label_color // Fixed: was status_color
        .and_then(|c| color_presets.get(c.as_str()))
        .unwrap_or(&"#150E5C"); // Fallback color is slate gray

    let icon_width = 30.0; // How large an icon is (the height will be capped though)
    let _scale = options.scale.unwrap_or(1.0);
    let icon_right_margin = 10.0;
    let height = FONT_SIZE * 1.2;

    let icon_span_width = if options.icon.is_some() {
        icon_width + icon_right_margin // Icon width + some right margin
    } else {
        0.0 // No icon no problem
    };

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

    // Voronoi speckled backgrounds
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

    let text_outline = create_text_outline()?;
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

    // Styling
    let style = css_style::style()
        .and_size(|conf| {
            conf.height(em(height_normalized))
                .width(em(total_width_normalized))
        })
        .and_border(|conf| conf.radius(px(10)));

    let style = format!(r#"svg {{{style}}}"#);
    document = document.add(svg::node::element::Style::new(style));

    // For testing/output (unchanged)
    let output = format!("{document:#}");
    let output = output.replace("\n", "");
    use std::fs;
    fs::write("./test.svg", output)?;

    Ok(document)
}

fn create_nnnoise_filter(id: &str) -> Filter {
    let fe_turbulence = FilterEffectTurbulence::new()
        .set("type", "turbulence")
        .set("baseFrequency", "0.102")
        .set("numOctaves", "4")
        .set("seed", "15")
        .set("stitchTiles", "stitch")
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("result", "turbulence");

    let fe_distant_light = FilterEffectDistantLight::new()
        .set("azimuth", "3")
        .set("elevation", "129");

    let fe_specular_lighting = FilterEffectSpecularLighting::new()
        .set("surfaceScale", "12")
        .set("specularConstant", "0.9")
        .set("specularExponent", "20")
        .set("lighting-color", "#7957A8")
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("in", "turbulence")
        .set("result", "specularLighting")
        .add(fe_distant_light);

    Filter::new()
        .set("id", id)
        .set("x", "-20%")
        .set("y", "-20%")
        .set("width", "140%")
        .set("height", "140%")
        .set("filterUnits", "objectBoundingBox")
        .set("primitiveUnits", "userSpaceOnUse")
        .set("color-interpolation-filters", "linearRGB")
        .add(fe_turbulence)
        .add(fe_specular_lighting)
}

pub fn bare(options: BadgerOptions) -> Result<Document, Box<dyn Error>> {
    let color_presets = &COLORS;
    let color = options
        .label_color
        .as_ref()
        .and_then(|c| color_presets.get(c.as_str()))
        .unwrap();

    let scale = options.scale.unwrap_or(1.0);

    // let st_text_width = calc_width(&options.status, 0.0)?;
    let st_rect_width = 1.0 + 115.0;

    let sanitized_status = &options.status;

    // Create boilerplate svg shell
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
