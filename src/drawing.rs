use std::ops::Not;

use egui::{
    Color32, CornerRadius, Grid, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, Widget as _, epaint::{self, CubicBezierShape, PathShape, QuadraticBezierShape, RectShape}, pos2
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Drawing {
    rects: Vec<Rect>,
    stroke: Stroke,
    fill: Color32,
    is_drawing: bool,
    start_pos: Pos2,
    default_size: Vec2,
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
        }
    }
}   

impl Drawing {
    #[allow(dead_code)]
    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
        //button to start drawing the rectangle
        if ui.button("Draw Rect").clicked() {
            // self.is_drawing = self.is_drawing.not();
            let second_pos = self.start_pos + self.default_size;
            let new_rect = Rect::from_two_pos(self.start_pos, second_pos);
            self.rects.push(new_rect);
        } 
            
        if ui.button("Cleanup").clicked() {
            self.rects.clear();
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

        self.rects.iter().for_each(|rect| {
            let new_rect_shape = RectShape {
                        rect: *rect,
                        corner_radius: CornerRadius::ZERO,
                        fill: self.fill,
                        stroke: self.stroke,
                        stroke_kind: StrokeKind::Outside,
                        round_to_pixels: None,
                        blur_width: 0.0,
                        brush: Default::default(),
                        angle: 0.0,
                    };
                painter.add(Shape::Rect(new_rect_shape));           
        });

        // painter.extend(control_point_shapes);

        response
    }
    pub fn ui(&mut self, ui: &mut Ui) {
         self.ui_content(ui);
        }
}
