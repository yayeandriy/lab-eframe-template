use egui::{Color32, Pos2, Rect, Stroke, Vec2, epaint::{CubicBezierShape, PathStroke, QuadraticBezierShape}, pos2};
use serde::{Deserialize, Serialize};
use grid_pathfinding::{PathingGrid, waypoints_to_path};
use grid_util::{Point, grid::ValueGrid};

#[derive(Deserialize, Serialize)]
pub struct PlanRoute {
    pub points_to_pass: Vec<Pos2>,
    pub corners: Vec<Pos2>,
    pub obstacles: Vec<Rect>,
    ps_grid_size: usize,
}

impl Default for PlanRoute {
    fn default() -> Self {
        Self {
            points_to_pass: Vec::new(),
            corners: Vec::new(),
            obstacles: Vec::new(),
            ps_grid_size: 100, // 100x100 grid for pathfinding
        }
    }
}

impl PlanRoute{
    pub fn new(points_to_pass: Vec<Pos2>, corners: Vec<Pos2>, obstacles: Vec<Rect>) -> Self {
        Self {
            points_to_pass,
            corners,
            obstacles,
            ps_grid_size: 100,
        }
    }
    
    pub fn find_route(&mut self) -> Vec<Pos2> {
        let mut pathing_grid: PathingGrid = PathingGrid::new(self.ps_grid_size, self.ps_grid_size, false);
        
        for rect in &self.obstacles {
            let min_x = (rect.min.x * self.ps_grid_size as f32) as usize;
            let min_y = (rect.min.y * self.ps_grid_size as f32) as usize;
            let max_x = (rect.max.x * self.ps_grid_size as f32).ceil() as usize;
            let max_y = (rect.max.y * self.ps_grid_size as f32).ceil() as usize;
            for x in min_x..max_x {
                for y in min_y..max_y {
                    pathing_grid.set(x as i32, y as i32, true);
                }
            }
        }
        //Restrict area out of the floor plan room bounds, which could complex shape and not necessarily rectangular. This is a bit of a hack, but it allows us to use the same pathfinding code without modification.
        let corners = &self.corners;
        if corners.len() >= 3 {
            // Scale corners to grid space
            let polygon: Vec<Pos2> = corners
                .iter()
                .map(|c| pos2(c.x * self.ps_grid_size as f32, c.y * self.ps_grid_size as f32))
                .collect();
            for x in 0..self.ps_grid_size {
                for y in 0..self.ps_grid_size {
                    let point = pos2(x as f32 + 0.5, y as f32 + 0.5);
                    if !point_in_polygon(point, &polygon) {
                        pathing_grid.set(x as i32, y as i32, true);
                    }
                }
            }
        }


        for point in &self.points_to_pass {
            let x = (point.x * self.ps_grid_size as f32) as usize;
            let y = (point.y * self.ps_grid_size as f32) as usize;
            pathing_grid.set(x as i32, y as i32, false);
        }
      
        pathing_grid.generate_components();
        let grid_points: Vec<Point> = self.points_to_pass
            .iter()
            .map(|p| {
        Point::new(
                    (p.x * self.ps_grid_size as f32) as i32,
                    (p.y * self.ps_grid_size as f32) as i32,
                )
            })
            .collect();

        let mut full_path = Vec::new();

        for segment in grid_points.windows(2) {
            let start = segment[0];
            let end = segment[1];

            let Some(path) = pathing_grid.get_path_single_goal(start, end, false) else {
                return vec![]; // or skip / handle partial
            };

            let mut expanded = waypoints_to_path(path);

            // avoid duplicating points between segments
            if !full_path.is_empty() {
                expanded.remove(0);
            }

            full_path.extend(expanded);
        }

        full_path
            .into_iter()
            .map(|p| {
                pos2(
                    (p.x as f32 + 0.5) / self.ps_grid_size as f32,
                    (p.y as f32 + 0.5) / self.ps_grid_size as f32,
                )
            })
            .collect()
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, canvas: Rect) {
        let w = canvas.width();
        let h = canvas.height();
        let painter = ui.painter();
        // Draw path as catmull-rom spline through the waypoints
        let path: Vec<Pos2> = self.find_route().into_iter()
            .map(|p| canvas.min + Vec2::new(p.x * w, p.y * h))
            .collect();
        if path.len() >= 2 {
            // Straight-line polyline
            painter.add(egui::Shape::line(
                path.clone(),
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(200, 200, 50, 120)),
            ));
            // Rounded-corner polyline (forced corners at points_to_pass)
            let forced: Vec<Pos2> = self.points_to_pass.iter()
                .map(|p| canvas.min + Vec2::new(p.x * w, p.y * h))
                .collect();
            for shape in rounded_polyline(&path, &forced, 10.0) {
                painter.add(shape);
            }
            // Smooth Catmull-Rom spline on top
            let spline = catmull_rom_to_beziers(&path);
            for bez in spline {
                painter.add(bez);
            }
        }
    }
}



/// Ray-casting point-in-polygon test (works for convex and concave polygons).
fn point_in_polygon(p: Pos2, polygon: &[Pos2]) -> bool {
    let n = polygon.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let pi = polygon[i];
        let pj = polygon[j];
        if ((pi.y > p.y) != (pj.y > p.y))
            && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Remove collinear intermediate points, keeping only start, end, and direction-change points.
fn decimate_path(pts: &[Pos2]) -> Vec<Pos2> {
    decimate_path_keeping(pts, &[], 0.0)
}


/// Like `decimate_path` but also always keeps points within `snap` distance of any
/// `forced` position (e.g. user waypoints that must appear as explicit corners).
fn decimate_path_keeping(pts: &[Pos2], forced: &[Pos2], snap: f32) -> Vec<Pos2> {
    if pts.len() <= 2 {
        return pts.to_vec();
    }
    let is_forced = |p: Pos2| forced.iter().any(|f| (*f - p).length() <= snap.max(1.0));
    let mut result = vec![pts[0]];
    for i in 1..pts.len() - 1 {
        let prev = pts[i - 1];
        let curr = pts[i];
        let next = pts[i + 1];
        let d1 = (curr - prev).normalized();
        let d2 = (next - curr).normalized();
        if (d1.dot(d2) - 1.0).abs() > 0.015 || is_forced(curr) {
            result.push(curr);
        }
    }
    result.push(*pts.last().unwrap());
    result
}



/// Convert a polyline into Catmull-Rom cubic bezier segments for smooth spline rendering.
fn catmull_rom_to_beziers(pts: &[Pos2]) -> Vec<CubicBezierShape> {
    // First decimate: keep only direction-change points to remove staircase artifacts
    let decimated = decimate_path(pts);
    let stroke = PathStroke::new(2.0, Color32::from_rgb(200, 50, 50));
    let n = decimated.len();
    let mut out = Vec::with_capacity(n.saturating_sub(1));
    for i in 0..n.saturating_sub(1) {
        let p0 = if i == 0 { decimated[0] } else { decimated[i - 1] };
        let p1 = decimated[i];
        let p2 = decimated[i + 1];
        let p3 = if i + 2 < n { decimated[i + 2] } else { decimated[n - 1] };

        let seg_len = (p2 - p1).length();

        // Catmull-Rom control point offsets
        let off1 = (p2 - p0) / 6.0;
        let off2 = (p3 - p1) / 6.0;

        // Clamp to half segment length to prevent overshooting / loops
        let max_len = seg_len * 0.45;
        let clamp = |v: Vec2| -> Vec2 {
            let l = v.length();
            if l > max_len { v * (max_len / l) } else { v }
        };

        let cp1 = p1 + clamp(off1);
        let cp2 = p2 - clamp(off2);

        out.push(CubicBezierShape {
            points: [p1, cp1, cp2, p2],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: stroke.clone(),
        });
    }
    out
}


/// Draw a polyline with straight segments and rounded corners (quadratic bezier arcs at turns).
/// `radius` is in screen pixels. `forced_corners` are screen-space positions that are always
/// kept as turn points regardless of local direction change.
fn rounded_polyline(pts: &[Pos2], forced_corners: &[Pos2], radius: f32) -> Vec<egui::Shape> {
    let pts = decimate_path_keeping(pts, forced_corners, radius);
    let stroke = Stroke::new(1.5, Color32::from_rgb(50, 200, 180));
    let n = pts.len();
    if n < 2 {
        return vec![];
    }
    let mut shapes = Vec::new();
    // cursor tracks where the previous segment ended (after its start-side trim)
    let mut seg_start = pts[0];
    for i in 1..n - 1 {
        let prev = pts[i - 1];
        let curr = pts[i];
        let next = pts[i + 1];
        let d_in  = (curr - prev).normalized();
        let d_out = (next - curr).normalized();
        // Clamp radius to half of either adjacent segment length
        let r = radius
            .min((curr - prev).length() * 0.5)
            .min((next - curr).length() * 0.5);
        let arc_start = curr - d_in  * r;
        let arc_end   = curr + d_out * r;
        // Straight segment up to the arc start
        shapes.push(egui::Shape::line_segment([seg_start, arc_start], stroke));
        // Quadratic bezier arc (control point at the corner)
        shapes.push(egui::Shape::QuadraticBezier(QuadraticBezierShape {
            points: [arc_start, curr, arc_end],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: PathStroke::new(stroke.width, stroke.color),
        }));
        seg_start = arc_end;
    }
    // Final straight segment to the last point
    shapes.push(egui::Shape::line_segment([seg_start, *pts.last().unwrap()], stroke));
    shapes
}
