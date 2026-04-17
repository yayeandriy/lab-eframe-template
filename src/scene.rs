use egui::{Color32, Rect, Scene};

use crate::paint_bezier::PaintBezier;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SceneDemo {
    scene_rect: Rect,
    bezier: PaintBezier
}

impl Default for SceneDemo {
    fn default() -> Self {
        Self {
            scene_rect: Rect::ZERO, // `egui::Scene` will initialize this to something valid
            bezier: PaintBezier::default(),
        }
    }
}



impl SceneDemo {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        
        ui.label(format!("Scene rect: {:#?}", &mut self.scene_rect));
        ui.label(format!("Curve size: {:#?}", &mut self.bezier.curve_size));

        ui.separator();

        egui::Frame::canvas(ui.style())
            .inner_margin(0.0)
            .fill(Color32::YELLOW)
            .show(ui, |ui| {
                let scene = Scene::new()
                    .max_inner_size([1350.0, 1000.0])
                    .zoom_range(0.1..=2.0);

                let _reset_view = false;
                let _inner_rect = Rect::NAN;
                let _response = scene
                    .show(ui, &mut self.scene_rect, |ui| {
                            self.bezier.ui(ui);
                        // reset_view = ui.button("Reset view").clicked();

                        // ui.add_space(16.0);

                        // ui.put(
                        //     Rect::from_min_size(Pos2::new(0.0, -64.0), Vec2::new(200.0, 16.0)),
                        //     egui::Label::new("You can take a widget nowhere").selectable(false),
                        // );

                        // inner_rect = ui.min_rect();
                        
                    })
                    .response;

                // if reset_view || response.double_clicked() {
                //     self.scene_rect = inner_rect;
                // }
            });
        
    }
}