use crate::context::Context;

#[derive(Default)]
pub struct StatusTab;

impl StatusTab {
    pub fn show(&mut self, ui: &mut egui::Ui, _ctx: &mut Context) {
        ui.label("Something");
    }
}
