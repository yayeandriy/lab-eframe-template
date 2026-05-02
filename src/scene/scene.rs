use egui::{Color32, Rect};

use crate::{floor_plan::floor_plan::FloorPlan, scene::{drawing::Drawing, paint_bezier::PaintBezier, scene_grid::SceeneGrid}};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MainScene {
    scene_rect: Rect,
    bezier: PaintBezier,
    drawing: Drawing,
    plan: FloorPlan,
    grid: SceeneGrid,
    scene_bg_color: Color32,
}

impl Default for MainScene {
    fn default() -> Self {
        Self {
            scene_rect: Rect::ZERO, // `egui::Scene` will initialize this to something valid
            bezier: PaintBezier::default(),
            drawing: Drawing::default(),
            plan: FloorPlan::default(),
            grid: SceeneGrid::default(),
            scene_bg_color: Color32::LIGHT_GRAY,
        }
    }
}



impl MainScene {
    fn ui_control(&mut self, ui: &mut egui::Ui) {
        // ui.label(format!("Scene rect: {:#?}", &mut self.scene_rect));
        // ui.label(format!("Curve size: {:#?}", &mut self.bezier.curve_size));
        // ui.label(format!("Number of nodes: {}", self.grid.n_nodes));
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
            self.grid.ui_control(ui);
            self.drawing.ui_control(ui);
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