use egui::{Color32, Key, Rect, Scene};

use crate::{paint_bezier::PaintBezier, scene_grid::SceeneGrid};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MainScene {
    scene_rect: Rect,
    bezier: PaintBezier,
    grid: SceeneGrid,
    scene_bg_color: Color32,
}

impl Default for MainScene {
    fn default() -> Self {
        Self {
            scene_rect: Rect::ZERO, // `egui::Scene` will initialize this to something valid
            bezier: PaintBezier::default(),
            grid: SceeneGrid::default(),
            scene_bg_color: Color32::RED,
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
            self.grid.ui_control(ui);
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        
       self.toolbar(ui);


        ui.separator();

        egui::Frame::canvas(ui.style())
            .inner_margin(0.0)
            .fill(self.scene_bg_color)
            .show(ui, |ui| {
                let space_held = ui.input(|i| i.key_down(Key::Space));

                if space_held {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }

                let scene = Scene::new()
                    .max_inner_size([1350.0, 1000.0])
                    .zoom_range(0.1..=2.0);

                // Snapshot before Scene mutates scene_rect.
                let prev_rect = self.scene_rect;

                scene
                    .show(ui, &mut self.scene_rect, |ui| {
                        self.grid.ui(ui);
                        self.bezier.ui(ui);
                    });

                if !space_held {
                    // Pan leaves size unchanged; zoom changes size.
                    // Cancel any pure pan, but let zoom through.
                    let size_unchanged = (self.scene_rect.size() - prev_rect.size()).length() < 0.5;
                    if size_unchanged {
                        self.scene_rect = prev_rect;
                    } else {
                        // Zoom happened — keep new scale but cancel translation.
                        self.scene_rect = Rect::from_center_size(
                            prev_rect.center(),
                            self.scene_rect.size(),
                        );
                    }
                }
            });
        
    }
}