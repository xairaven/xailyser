#[derive(Default)]
pub struct Settings;

const DEFAULT_SPACE: f32 = 10.0;

impl Settings {
    pub fn show(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Settings");
            });

            ui.add_space(DEFAULT_SPACE);

            ui.label("Menu entry 1");
            ui.label("Menu entry 2");
            ui.label("Menu entry 3");
        });
    }
}
