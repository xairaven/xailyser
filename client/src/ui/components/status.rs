#[derive(Default)]
pub struct StatusComponent;

impl StatusComponent {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("Status");
    }
}
