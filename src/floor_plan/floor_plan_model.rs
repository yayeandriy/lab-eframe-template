use egui::{Rect, Vec2};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FloorPlanModel {
    pub room_corners: Vec<Vec2>,
    /// Last SVG load error, not persisted.
    #[serde(skip)]
    pub load_error: Option<String>,
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
            load_error: None,
        }
    }
}

impl FloorPlanModel {
    /// Open a native file dialog, load the chosen SVG, and replace `room_corners`.
    /// On wasm this is a no-op (file access not yet supported here).
    pub fn load_from_svg_dialog(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::floor_plan::svg_loader::load_svg_corners;

            let picked = rfd::FileDialog::new()
                .add_filter("SVG", &["svg"])
                .set_directory(
                    std::env::current_dir()
                        .unwrap_or_default()
                        .join("assets"),
                )
                .pick_file();

            if let Some(path) = picked {
                match load_svg_corners(&path) {
                    Ok(corners) => {
                        self.room_corners = corners;
                        self.load_error = None;
                    }
                    Err(e) => {
                        self.load_error = Some(e);
                    }
                }
            }
        }
    }

    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
        if ui.button("Load floor plan SVG…").clicked() {
            self.load_from_svg_dialog();
        }
        if let Some(err) = &self.load_error {
            ui.colored_label(egui::Color32::RED, err);
        }
    }

    pub fn draw_room(&mut self, ui: &mut egui::Ui, canvas: Rect) {
        let wh = Vec2::new(canvas.width(), canvas.height());
        let points: Vec<egui::Pos2> = self
            .room_corners
            .iter()
            .map(|corner| canvas.min + *corner * wh)
            .collect();
        ui.painter().add(egui::Shape::closed_line(
            points,
            egui::Stroke::new(2.0, egui::Color32::BLACK),
        ));
    }
}