use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2, epaint::PathShape, pos2};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Drawing {
    rects: Vec<[Pos2; 4]>,
    stroke: Stroke,
    fill: Color32,
    is_drawing: bool,
    start_pos: Pos2,
    default_size: Vec2,
    pub grid_dist: Vec2,
    #[serde(skip)]
    drag_origins: Vec<Option<[Pos2; 4]>>,
}

impl Default for Drawing {
    fn default() -> Self {
        Self {
            rects: Vec::new(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            fill: Color32::from_rgb(50, 100, 150).linear_multiply(0.25),
            is_drawing: false,
            start_pos: pos2(0.0, 0.0),
            default_size: Vec2::new(100.0, 100.0),
            grid_dist: Vec2::new(10.0, 10.0),
            drag_origins: Vec::new(),
        }
    }
}   

impl Drawing {
    #[allow(dead_code)]
    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
        //button to start drawing the rectangle
        if ui.button("Draw Rect").clicked() {
            let second_pos = self.start_pos + self.default_size;
            let new_rect = [pos2(self.start_pos.x, self.start_pos.y), pos2(second_pos.x, self.start_pos.y), pos2(second_pos.x, second_pos.y), pos2(self.start_pos.x, second_pos.y)];
            self.rects.push(new_rect);
            self.drag_origins.push(None);
        } 
            
        if ui.button("Cleanup").clicked() {
            self.rects.clear();
            self.drag_origins.clear();
        }

        
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let desired_size = Vec2::new(ui.available_width(), ui.available_height());
        let (_rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
        let painter = ui.painter_at(ui.clip_rect());
        // if self.is_drawing {
        //     ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        //     if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
        //         if ui.input(|i| i.pointer.any_released()) {
        //             self.is_drawing = false;
                    
        //         }
        //     }
        // }

        // Ensure drag_origins is in sync with rects length.
        self.drag_origins.resize(self.rects.len(), None);
        let stroke = self.stroke;
        let grid_dist = self.grid_dist;
        let drag_origins = &mut self.drag_origins;
        self.rects.iter_mut().enumerate().for_each(|(rect_idx, rect)| {
            Self::draw_rect(ui, &painter, rect, stroke, rect_idx, &mut drag_origins[rect_idx], grid_dist);
        });

        // painter.extend(control_point_shapes);

        response
    }
    pub fn ui(&mut self, ui: &mut Ui) {
         self.ui_content(ui);
        }

    
    fn snap(p: Pos2, dist: Vec2) -> Pos2 {
        pos2(
            (p.x / dist.x).round() * dist.x,
            (p.y / dist.y).round() * dist.y,
        )
    }

    fn draw_rect(ui: &mut Ui, painter: &egui::Painter, points: &mut [Pos2; 4], stroke: Stroke, rect_idx: usize, drag_origin: &mut Option<[Pos2; 4]>, grid_dist: Vec2) {
        // Edge handles: each edge connects corner i and corner (i+1)%4.
        // We make the midpoint of each edge draggable and move the two endpoints
        // only along the edge's normal (perpendicular) axis.
        for edge in 0..4usize {
            let a = points[edge];
            let b = points[(edge + 1) % 4];
            let mid = a + (b - a) * 0.5;
            let edge_vec = b - a;
            // Normal direction (perpendicular to the edge).
            let normal = Vec2::new(-edge_vec.y, edge_vec.x).normalized();

            let edge_id = ui.id().with(rect_idx).with("edge").with(edge);
            let hit_size = Vec2::new(edge_vec.length().max(8.0), 8.0);
            // Build an axis-aligned hit rect around the midpoint (good enough for AABB interaction).
            let edge_hit = Rect::from_center_size(mid, Vec2::splat(8.0).max(hit_size.abs()));
            let edge_response = ui.interact(edge_hit, edge_id, Sense::drag());

            if edge_response.dragged() {
                let delta = edge_response.drag_delta();
                let proj = delta.dot(normal);
                let move_vec = normal * proj;
                points[edge] += move_vec;
                points[(edge + 1) % 4] += move_vec;
            }
            if edge_response.dragged() || edge_response.drag_stopped() {
                points[edge] = Self::snap(points[edge], grid_dist);
                points[(edge + 1) % 4] = Self::snap(points[(edge + 1) % 4], grid_dist);
            }

            // Draw a small tick at the midpoint to show the handle.
            let handle_stroke = ui.style().interact(&edge_response).fg_stroke;
            painter.add(Shape::circle_filled(mid, 3.0, handle_stroke.color));
        }

        // Corner handles.
        for i in 0..4usize {
            let point = &mut points[i];
            let size = Vec2::splat(8.0);
            let point_rect = Rect::from_center_size(*point, size);
            let point_id = ui.id().with(rect_idx).with("corner").with(i);
            let point_response = ui.interact(point_rect, point_id, Sense::drag());
            *point += point_response.drag_delta();
            if point_response.dragged() || point_response.drag_stopped() {
                *point = Self::snap(*point, grid_dist);
            }
            let corner_stroke = ui.style().interact(&point_response).fg_stroke;
            painter.add(Shape::circle_stroke(*point, 4.0, corner_stroke));
        }

        // Draw the polygon body.
        let rect_shape = PathShape::closed_line(points.to_vec(), stroke);
        painter.add(Shape::Path(rect_shape));

        // Body drag — use an inset rect so edges/corners remain grabbable.
        let bbox = points.iter().fold(
            Rect::from_min_max(points[0], points[0]),
            |r, &p| r.union(Rect::from_min_max(p, p)),
        );
        let inset = 12.0;
        let body_rect = bbox.shrink(inset);
        let body_id = ui.id().with(rect_idx).with("body");
        let body_response = ui.interact(body_rect, body_id, Sense::drag());

        let any_handle_active = (0..4).any(|i| {
            ui.ctx().is_being_dragged(ui.id().with(rect_idx).with("corner").with(i))
                || ui.ctx().is_being_dragged(ui.id().with(rect_idx).with("edge").with(i))
        });

        let is_hovered = body_response.hovered() && !any_handle_active;
        let is_body_dragging = body_response.dragged() && !any_handle_active;

        // Record origin at drag start; clear when released.
        if is_body_dragging && drag_origin.is_none() {
            *drag_origin = Some(*points);
        } else if !is_body_dragging {
            *drag_origin = None;
        }

        // Show ghost at original position while dragging.
        if let Some(origin) = drag_origin {
            let ghost_stroke = Stroke::new(1.0, stroke.color.gamma_multiply(0.4));
            painter.add(Shape::Path(PathShape::closed_line(origin.to_vec(), ghost_stroke)));
        }

        // Hover highlight.
        if is_hovered || is_body_dragging {
            let highlight_stroke = Stroke::new(stroke.width + 1.5, stroke.color.gamma_multiply(1.6));
            painter.add(Shape::Path(PathShape::closed_line(points.to_vec(), highlight_stroke)));
        }

        if is_body_dragging {
            let delta = body_response.drag_delta();
            for p in points.iter_mut() {
                *p += delta;
            }
        }
        if (is_body_dragging || body_response.drag_stopped()) && !any_handle_active {
            for p in points.iter_mut() {
                *p = Self::snap(*p, grid_dist);
            }
        }
    }

}
