use rand::Rng;
use svg::node::element::{Circle, Group, Rectangle};
use voronator::VoronoiDiagram;
use voronator::delaunator::Point as VoronoiPoint;

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

pub fn create_speckle_group(
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

    bg = bg.add(
        Rectangle::new()
            .set("fill", fill_color.to_string())
            .set("x", x_offset)
            .set("width", section_width)
            .set("height", section_height),
    );

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
