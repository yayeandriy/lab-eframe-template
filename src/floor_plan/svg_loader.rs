use egui::{Rect, Vec2};
use std::path::Path;

pub fn load_svg_geometry(path: &Path) -> Result<(Vec<Vec2>, Vec<Rect>), String> {
    let corners = load_svg_corners(path)?;
    let svg_size = calc_svg_size(&corners);
    let normalized_corners = normalize(corners);
    let boxes = load_svg_boxes(path, svg_size).unwrap_or_default();
    Ok((normalized_corners, boxes))
}

fn calc_svg_size(corners: &[Vec2]) -> Vec2 {
    let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_x = corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let max_y = corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
    Vec2::new(max_x - min_x, max_y - min_y)
}

/// Load the first `<path>` from an SVG file and return its vertices normalized to 0–1.
pub fn load_svg_corners(path: &Path) -> Result<Vec<Vec2>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;

    let doc = roxmltree::Document::parse(&content)
        .map_err(|e| format!("SVG parse error: {e}"))?;

    // Find the first <path> element with a 'd' attribute
    let d = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "path" && n.has_attribute("d"))
        .and_then(|n| n.attribute("d"))
        .ok_or_else(|| "No <path d=\"...\"> element found in SVG".to_string())?;

    let points = parse_path_d(d)?;

    if points.len() < 3 {
        return Err(format!(
            "SVG path has only {} points, need at least 3",
            points.len()
        ));
    }

    Ok(points)
}

pub fn load_svg_boxes(path: &Path, svg_size: Vec2) -> Result<Vec<Rect>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;

    let doc = roxmltree::Document::parse(&content)
        .map_err(|e| format!("SVG parse error: {e}"))?;

    // Find all <rect> elements and convert them to corner points
    let mut boxes = Vec::new();
    for node in doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "rect") {
        let x = node.attribute("x").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        let y = node.attribute("y").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        let width = node.attribute("width").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        let height = node.attribute("height").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

        if width > 1e-6 && height > 1e-6 {
            println!("SVG BOX SOIRCE: {:?}", (x, y, width, height));
            let min = 
                vec![
                    Vec2::new(x as f32, y as f32) / svg_size,
                    Vec2::new(width as f32, height as f32) / svg_size,
                ] ;
            
            println!("SVG BOX NORMALIZED: {:?}", min);
            boxes.push(Rect::from_min_size(
                min[0].to_pos2(),
                min[1],
            ));
        }
    }

    if boxes.is_empty() {
        return Err("No <rect> elements found in SVG".to_string());
    }

    Ok(boxes)
}


/// Parse an SVG path `d` string into a flat list of 2-D points.
/// For curve commands only the end-point is used (good enough for room outlines).
fn parse_path_d(d: &str) -> Result<Vec<Vec2>, String> {
    use svgtypes::{PathParser, PathSegment};

    let mut points: Vec<Vec2> = Vec::new();
    let mut cx = 0.0f64;
    let mut cy = 0.0f64;
    let mut start_x = 0.0f64;
    let mut start_y = 0.0f64;

    for segment in PathParser::from(d) {
        let seg = segment.map_err(|e| format!("Path parse error: {e}"))?;
        match seg {
            PathSegment::MoveTo { abs, x, y } => {
                let (nx, ny) = resolve(abs, cx, cy, x, y);
                cx = nx;
                cy = ny;
                start_x = cx;
                start_y = cy;
                points.push(v(cx, cy));
            }
            PathSegment::LineTo { abs, x, y } => {
                let (nx, ny) = resolve(abs, cx, cy, x, y);
                cx = nx;
                cy = ny;
                points.push(v(cx, cy));
            }
            PathSegment::HorizontalLineTo { abs, x } => {
                cx = if abs { x } else { cx + x };
                points.push(v(cx, cy));
            }
            PathSegment::VerticalLineTo { abs, y } => {
                cy = if abs { y } else { cy + y };
                points.push(v(cx, cy));
            }
            // For curves use only the endpoint — fine for polygonal room outlines
            PathSegment::CurveTo { abs, x, y, .. } => {
                let (nx, ny) = resolve(abs, cx, cy, x, y);
                cx = nx;
                cy = ny;
                points.push(v(cx, cy));
            }
            PathSegment::SmoothCurveTo { abs, x, y, .. } => {
                let (nx, ny) = resolve(abs, cx, cy, x, y);
                cx = nx;
                cy = ny;
                points.push(v(cx, cy));
            }
            PathSegment::Quadratic { abs, x, y, .. } => {
                let (nx, ny) = resolve(abs, cx, cy, x, y);
                cx = nx;
                cy = ny;
                points.push(v(cx, cy));
            }
            PathSegment::SmoothQuadratic { abs, x, y } => {
                let (nx, ny) = resolve(abs, cx, cy, x, y);
                cx = nx;
                cy = ny;
                points.push(v(cx, cy));
            }
            PathSegment::EllipticalArc { abs, x, y, .. } => {
                let (nx, ny) = resolve(abs, cx, cy, x, y);
                cx = nx;
                cy = ny;
                points.push(v(cx, cy));
            }
            PathSegment::ClosePath { .. } => {
                cx = start_x;
                cy = start_y;
                // Don't add a duplicate closing point
            }
        }
    }

    // Remove the last point if it duplicates the first (closed path)
    if points.len() > 1 {
        let first = points[0];
        if let Some(last) = points.last() {
            if (last.x - first.x).abs() < 1e-4 && (last.y - first.y).abs() < 1e-4 {
                points.pop();
            }
        }
    }

    Ok(points)
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn resolve(abs: bool, cx: f64, cy: f64, x: f64, y: f64) -> (f64, f64) {
    if abs { (x, y) } else { (cx + x, cy + y) }
}

fn v(x: f64, y: f64) -> Vec2 {
    Vec2::new(x as f32, y as f32)
}

fn normalize(pts: Vec<Vec2>) -> Vec<Vec2> {
    let min_x = pts.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let min_y = pts.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_x = pts.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let max_y = pts.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
    let rx = (max_x - min_x).max(1e-6);
    let ry = (max_y - min_y).max(1e-6);
    pts.into_iter()
        .map(|p| Vec2::new((p.x - min_x) / rx, (p.y - min_y) / ry))
        .collect()
}
