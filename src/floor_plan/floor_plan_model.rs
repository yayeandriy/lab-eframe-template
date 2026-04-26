use egui::{Rect, Vec2};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FloorPlanModel {
    pub room_corners: Vec<Vec2>,
}

impl Default for FloorPlanModel {
    fn default() -> Self {
        Self {
            room_corners: vec![
                Vec2::new(0.1, 0.1),
                Vec2::new(0.5, 0.1),
                Vec2::new(0.5, 0.2),
                Vec2::new(0.9, 0.1),
                Vec2::new(0.9, 0.9),
                Vec2::new(0.1, 0.9),
            ],
        }
    }
}

impl FloorPlanModel {
    pub fn draw_room(&mut self, ui: &mut egui::Ui, canvas: Rect) {
        // Draw the floor plan
        let wh = Vec2::new(canvas.width(), canvas.height());
        let points: Vec<egui::Pos2> = self.room_corners.iter().map(|corner| canvas.min + *corner * wh).collect();
        ui.painter().add(egui::Shape::closed_line(points.clone(), egui::Stroke::new(2.0, egui::Color32::BLACK)));
    }
}