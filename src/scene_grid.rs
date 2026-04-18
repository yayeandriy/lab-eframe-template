use egui::{Color32, Sense, Shape, Ui, Vec2, pos2};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SceeneGrid {
    unit_size: Vec2,
    dist: Vec2, //distance between nodes in unit size
    node_size: f32,
    grid_color: Color32,
    pub n_nodes: usize
}

impl Default for SceeneGrid {
    fn default() -> Self {
        Self {
            unit_size: Vec2::new(10.0, 10.0),
            dist: Vec2::new(10.0, 10.0),
            node_size: 1.0,
            grid_color: Color32::GRAY.linear_multiply(0.25),
            n_nodes: 0,
        }
    }
}


impl SceeneGrid {
    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let desired_size = Vec2::new(ui.available_width(), ui.available_height());
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
        // Bypass the allocated-rect clip: paint on the scene's full clip rect.
        let painter = ui.painter_at(ui.clip_rect());

        let grid_color = self.grid_color;
        // Use desired_size (captured before allocation) — available_* returns 0 after allocate.
        let number_nodes_x = (desired_size.x / self.dist.x).ceil() as usize;
        let number_nodes_y = (desired_size.y / self.dist.y).ceil() as usize;

        self.n_nodes = (number_nodes_x + 1) * (number_nodes_y + 1);
        for i in 0..=number_nodes_x {
            for j in 0..=number_nodes_y {
                let x = rect.min.x + i as f32 * self.dist.x;
                let y = rect.min.y + j as f32 * self.dist.y;
                painter.add(Shape::circle_filled(pos2(x, y), self.node_size, grid_color));
            }
        }


        response
    }
    pub fn ui(&mut self, ui: &mut Ui) {
        // self.ui_control(ui);

        // Frame::canvas(ui.style())
        //     .fill(Color32::TRANSPARENT) 
        //     .stroke(Stroke::NONE)
        //     .show(ui, |ui| {
        //         self.ui_content(ui);
        //     });
        self.ui_content(ui);
        }
}
