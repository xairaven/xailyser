#[derive(Default)]
pub struct SettingsComponent;

impl SettingsComponent {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("Settings");
    }
}
