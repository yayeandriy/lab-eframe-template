use egui::{Color32, Rect};

use crate::floor_plan::floor_plan::FloorPlan;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MainScene {
    scene_rect: Rect,
    plan: FloorPlan,
    scene_bg_color: Color32,
}

impl Default for MainScene {
    fn default() -> Self {
        Self {
            scene_rect: Rect::ZERO, // `egui::Scene` will initialize this to something valid
            plan: FloorPlan::default(),
            scene_bg_color: Color32::LIGHT_GRAY,
        }
    }
}



impl MainScene {
    fn ui_control(&mut self, ui: &mut egui::Ui) {
       
        ui.horizontal(|ui| {
            ui.label("Bg color");
            ui.color_edit_button_srgba(&mut self.scene_bg_color);
        });
    }
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.ui_control(ui);
            ui.separator();
            self.plan.ui_control(ui);
            ui.separator();
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        
       self.toolbar(ui);


        ui.separator();

        egui::Frame::canvas(ui.style())
            .inner_margin(0.0)
            .fill(self.scene_bg_color)
            .show(ui, |ui| {
                self.plan.ui_content(ui);
            });
        
    }
}