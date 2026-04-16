use egui::{
    Color32, Grid, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, Widget as _,
    epaint::{self, CubicBezierShape, PathShape, QuadraticBezierShape},
    pos2,
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PaintBezier {
    /// Bézier curve degree, it can be 3, 4.
    degree: usize,

    /// The control points. The [`Self::degree`] first of them are used.
    control_points: [Pos2; 4],

    /// Stroke for Bézier curve.
    stroke: Stroke,

    /// Fill for Bézier curve.
    fill: Color32,

    /// Stroke for auxiliary lines.
    aux_stroke: Stroke,

    bounding_box_stroke: Stroke,
}

impl Default for PaintBezier {
    fn default() -> Self {
        Self {
            degree: 4,
            control_points: [
                pos2(50.0, 50.0),
                pos2(60.0, 250.0),
                pos2(200.0, 200.0),
                pos2(250.0, 50.0),
            ],
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            fill: Color32::from_rgb(50, 100, 150).linear_multiply(0.25),
            aux_stroke: Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
            bounding_box_stroke: Stroke::new(0.0, Color32::LIGHT_GREEN.linear_multiply(0.25)),
        }
    }
}

impl PaintBezier {
    #[allow(dead_code)]
    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Colors", |ui| {
            Grid::new("colors")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Fill color");
                    ui.color_edit_button_srgba(&mut self.fill);
                    ui.end_row();

                    ui.label("Curve Stroke");
                    ui.add(&mut self.stroke);
                    ui.end_row();

                    ui.label("Auxiliary Stroke");
                    ui.add(&mut self.aux_stroke);
                    ui.end_row();

                    ui.label("Bounding Box Stroke");
                    ui.add(&mut self.bounding_box_stroke);
                    ui.end_row();
                });
        });

        ui.collapsing("Global tessellation options", |ui| {
            let mut tessellation_options = ui.ctx().tessellation_options(|to| *to);
            tessellation_options.ui(ui);
            ui.tessellation_options_mut(|to| *to = tessellation_options);
        });

        ui.radio_value(&mut self.degree, 3, "Quadratic Bézier");
        ui.radio_value(&mut self.degree, 4, "Cubic Bézier");
        ui.label("Move the points by dragging them.");
        ui.small("Only convex curves can be accurately filled.");
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), ui.available_height()), Sense::hover());
        // let painter = painter.with_clip_rect(egui::Rect::EVERYTHING);
        
        // let to_screen = emath::RectTransform::from_to(
        //     Rect::from_min_size(Pos2::ZERO, response.rect.size()),
        //     response.rect,
        // );

        let control_point_radius = 8.0;

        let control_point_shapes: Vec<Shape> = self
            .control_points
            .iter_mut()
            .enumerate()
            .take(self.degree)
            .map(|(i, point)| {
                let size = Vec2::splat(2.0 * control_point_radius);

                let point_rect = Rect::from_center_size(*point, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                *point += point_response.drag_delta();

                let stroke = ui.style().interact(&point_response).fg_stroke;

                Shape::circle_stroke(*point, control_point_radius, stroke)
            })
            .collect();

        let points: Vec<Pos2> = self
            .control_points
            .iter()
            .take(self.degree)
            .cloned()
            .collect();

        match self.degree {
            3 => {
                let points = points.clone().try_into().unwrap();
                let shape =
                    QuadraticBezierShape::from_points_stroke(points, true, self.fill, self.stroke);
                painter.add(epaint::RectShape::stroke(
                    shape.visual_bounding_rect(),
                    0.0,
                    self.bounding_box_stroke,
                    StrokeKind::Outside,
                ));
                painter.add(shape);
            }
            4 => {
                let points = points.clone().try_into().unwrap();
                let shape =
                    CubicBezierShape::from_points_stroke(points, true, self.fill, self.stroke);
                painter.add(epaint::RectShape::stroke(
                    shape.visual_bounding_rect(),
                    0.0,
                    self.bounding_box_stroke,
                    StrokeKind::Outside,
                ));
                painter.add(shape);
            }
            _ => {
                unreachable!();
            }
        }

        painter.add(PathShape::line(points, self.aux_stroke));
        painter.extend(control_point_shapes);

        response
    }
}

impl PaintBezier {
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