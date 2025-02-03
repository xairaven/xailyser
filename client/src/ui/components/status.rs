use crate::context::Context;
use crate::ui::styles;
use egui::RichText;

#[derive(Default)]
pub struct StatusComponent;

impl StatusComponent {
    pub fn show(&mut self, ui: &mut egui::Ui, _ctx: &mut Context) {
        ui.vertical_centered_justified(|ui| {
            ui.label(
                RichText::new("Dashboard")
                    .size(styles::COMPONENT_HEADING_FONT_SIZE)
                    .strong(),
            );
        });
    }
}
