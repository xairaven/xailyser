use crate::context::Context;

#[derive(Default)]
pub struct StatusComponent;

impl StatusComponent {
    pub fn show(&mut self, ui: &mut egui::Ui, _ctx: &mut Context) {
        ui.label("Status");
    }
}
