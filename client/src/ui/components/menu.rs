use crate::context::Context;
use crate::ui::components::root;
use crate::ui::styles;
use crate::ui::tabs::Tab;
use egui::RichText;
use strum::IntoEnumIterator;

#[derive(Default)]
pub struct Menu {
    pub active_tab: Tab,
}

impl Menu {
    pub fn show(&mut self, ui: &mut egui::Ui, _ctx: &mut Context) {
        ui.vertical_centered_justified(|ui| {
            ui.heading(
                RichText::new("Menu")
                    .size(styles::COMPONENT_HEADING_FONT_SIZE)
                    .strong(),
            );
        });

        ui.add_space(10.0);

        ui.vertical_centered_justified(|ui| {
            for tab in Tab::iter() {
                if ui
                    .add_sized(
                        [root::MENU_PANEL_MIN_WIDTH - 10.0, styles::BUTTON_HEIGHT],
                        egui::Button::new(
                            RichText::new(format!("{}", tab))
                                .size(styles::COMPONENT_FONT_SIZE),
                        ),
                    )
                    .clicked()
                {
                    self.active_tab = tab;
                }
                ui.add_space(2.0);
            }
        });
    }
}
