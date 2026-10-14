use eframe::{NativeOptions, egui};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "custom",
        NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

struct MyApp;
impl Default for MyApp {
    fn default() -> Self {
        Self
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame) {
        ui.heading("custom show");
        let (resp, painter) = ui.allocate_painter(
            eframe::egui::Vec2 { x: 300.0, y: 150.0 },
            egui::Sense::hover(),
        );
        let r = resp.rect;
        let shape = egui::Shape::rect_filled(
            egui::Rect::from_min_size(
                r.min + egui::vec2(10.0, 10.0),
                egui::Vec2 { x: 30.0, y: 40. },
            ),
            5.0,
            egui::Color32::RED,
        );
        painter.add(shape);
        if resp.hovered() {
            ui.label("mouse in the area");
        }
    }
}
