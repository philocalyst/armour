use rand::Rng;
use svg::node::element::{Group, Polygon, Rectangle};
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

struct VoronoiCell {
    vertices: Vec<(f64, f64)>,
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
            if pts.len() < 3 {
                return None;
            }
            Some(VoronoiCell {
                vertices: pts.iter().map(|p| (p.x, p.y)).collect(),
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

    let mut cell_group = Group::new()
        .set("filter", format!("url(#{filter_id})"))
        .set("clip-path", format!("url(#{clip_id})"));

    for cell in &cells {
        let opacity = rng.random_range(0.05..=0.35);

        let points_str: String = cell
            .vertices
            .iter()
            .map(|(x, y)| format!("{},{}", x + x_offset as f64, y))
            .collect::<Vec<_>>()
            .join(" ");

        cell_group = cell_group.add(
            Polygon::new()
                .set("points", points_str)
                .set("fill", "#ffffff")
                .set("stroke", "#ffffff")
                .set("stroke-width", 0.3)
                .set("opacity", format!("{opacity:.2}")),
        );
    }

    bg = bg.add(cell_group);
    bg
}
