use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2, pos2};

use crate::floor_plan::{plan_route::PlanRoute, svg_loader::SvgLoader};




#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlanGeometry {
    pub corners: Vec<Vec2>,
    pub obstacles: Vec<Rect>,
}

impl Default for PlanGeometry {
    fn default() -> Self {
         let obstacles: Vec<Rect> = (0..5).map(|_| {
            let x0 = rand::random::<f32>() * 0.9;
            let y0 = rand::random::<f32>() * 0.9;
            let w = 0.05 + rand::random::<f32>() * 0.05;
            let h = 0.05 + rand::random::<f32>() * 0.05;
            Rect::from_min_max(pos2(x0, y0), pos2(x0 + w, y0 + h))
        }).collect();
        let corners = vec![
                Vec2::new(0.1, 0.1),
                Vec2::new(0.5, 0.1),
                Vec2::new(0.5, 0.2),
                Vec2::new(0.9, 0.1),
                Vec2::new(0.9, 0.9),
                Vec2::new(0.1, 0.9),
            ];
        Self {
            corners,
            obstacles,
        }
    }           
}



#[derive(serde::Deserialize, serde::Serialize)]
pub struct FloorPlan {    
    data_loader: SvgLoader,
    plan_geometry: PlanGeometry,
    points_to_pass: Vec<Pos2>,
    lines: Vec<Vec<Pos2>>,
    plan_route: PlanRoute,
    stroke: Stroke,
    fill: Color32,
    corner_radius: f32,
}

impl Default for FloorPlan {
    fn default() -> Self {        
        let points_to_pass: Vec<Pos2> = (0..10).map(|_| pos2(rand::random::<f32>(), rand::random::<f32>())).collect();
        let svg_loader = SvgLoader::default();
        Self {
            data_loader: svg_loader,
            plan_geometry: PlanGeometry::default(),
            points_to_pass,
            lines: Default::default(),
            stroke: Stroke::new(2.0, Color32::from_rgb(25, 200, 100)),
            fill: Color32::from_rgb(50, 100, 200),
            corner_radius: 12.0,
            plan_route: PlanRoute::default(),
        }
    }
}   

impl FloorPlan {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.corner_radius, 0.0..=80.0).text("Corner radius"));
        // ui.add(egui::Slider::new(&mut self.ps_grid_size, 10..=800).text("Grid size"));
        ui.separator();
        self.data_loader.ui_control(ui, &mut self.plan_geometry);

    }

}

impl FloorPlan {
    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (canvas, response) = ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::hover());

        
        // self.plan_geometry = plan_geometry;
        let corners_pos = self.plan_geometry.corners.iter().map(|c| pos2(c.x, c.y)).collect();
        let plan_route = PlanRoute::new(self.points_to_pass.clone(), corners_pos, self.plan_geometry.obstacles.clone());
        self.plan_route = plan_route; 
        // Interact first (needs &mut ui, painter not yet alive)
        self.draw_obstacles(ui, canvas);
        self.draw_stops(ui, canvas);
        self.draw_walls(ui, canvas);
        self.plan_route.draw(ui, canvas);

        response
    }




    fn draw_stops(&mut self, ui: &mut Ui, canvas: Rect) {
        let wh = Vec2::new(canvas.width(), canvas.height());
        let point_radius = 6.0;
        let mut drag_delta: Option<(usize, Vec2)> = None;
        let painter = ui.painter();
        for (point_idx, point) in self.points_to_pass.iter().enumerate() {
            let center = canvas.min + point.to_vec2() * wh;
            let rect_to_drag = Rect::from_center_size(center, Vec2::splat(point_radius * 2.0));
            let id = ui.id().with(point_idx).with("stop");
            let response = ui.interact(rect_to_drag, id, Sense::drag());
            if response.dragged() {
                 drag_delta = Some((point_idx, response.drag_delta()));               
            }            
        }
        // Apply drag (mutable borrow, separate from the iterator above)
        if let Some((point_idx, delta)) = drag_delta {
            let pos = delta / wh;
            self.points_to_pass[point_idx] += pos;            
        }

        // Draw all points
        for point in &self.points_to_pass {
            let center = canvas.min + point.to_vec2() * wh;
            painter.circle_filled(center, point_radius, self.fill);
        }            

    }

    fn draw_obstacles(&mut self, ui: &mut Ui, canvas: Rect) {
        let wh = Vec2::new(canvas.width(), canvas.height());
        // Collect drag deltas first (immutable borrow ends before mutation)\n
        let mut drag_delta: Option<(usize, Vec2)> = None;
        for (rect_idx, rect) in self.plan_geometry.obstacles.iter().enumerate() {        
            let min = canvas.min + rect.min.to_vec2() * wh;
            let max = canvas.min + rect.max.to_vec2() * wh;
            let r = Rect::from_min_max(min, max);
            let inset = 12.0;
            let body_rect = r.shrink(inset);
            let body_id = ui.id().with(rect_idx).with("body");
            let body_response = ui.interact(body_rect, body_id, Sense::drag());
            if body_response.dragged() {
                drag_delta = Some((rect_idx, body_response.drag_delta()));
            }
        }
        // Apply drag (mutable borrow, separate from the iterator above)
        if let Some((rect_idx, delta)) = drag_delta {
            let delta = delta / wh;
            self.plan_geometry.obstacles[rect_idx].min += delta;
            self.plan_geometry.obstacles[rect_idx].max += delta;
        }

        // Draw all rects
        let painter = ui.painter();
        for rect in &self.plan_geometry.obstacles {
            let min = canvas.min + rect.min.to_vec2() * wh;
            let max = canvas.min + rect.max.to_vec2() * wh;
            let r = Rect::from_min_max(min, max);
            painter.rect_filled(r, 0.0, self.fill);
            painter.rect_stroke(r, 0.0, self.stroke, egui::StrokeKind::Middle);
        }
    }


    pub fn draw_walls(&mut self, ui: &mut egui::Ui, canvas: Rect) {
        let wh = Vec2::new(canvas.width(), canvas.height());
        let points: Vec<egui::Pos2> = self
            .plan_geometry.corners
            .iter()
            .map(|corner| canvas.min + *corner * wh)
            .collect();
        // println!("Drawing room with corners: {:?}", points);
        ui.painter().add(egui::Shape::closed_line(
            points,
            Stroke::new(1.0, Color32::BLACK),
        ));

        let boxes = self.plan_geometry.obstacles.iter().map(|b| {
            Rect::from_min_max(
                canvas.min + b.min.to_vec2() * wh,
                canvas.min + b.max.to_vec2() * wh,
            )
        });
        // println!("Drawing room with boxes: {:?}", boxes);
        for b in boxes {
            ui.painter().rect_filled(
                b,
                0.0,
                Color32::DARK_RED
            );  
        }
    }

}
