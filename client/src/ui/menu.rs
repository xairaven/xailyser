use crate::context::Context;
use crate::ui::styles;
use egui::RichText;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Default, Display, EnumIter)]
pub enum Tab {
    #[default]
    #[strum(to_string = "🏠 Status")]
    Status,

    #[strum(to_string = "⚙ Settings")]
    Settings,

    #[strum(to_string = "ℹ About")]
    About,

    #[strum(to_string = "🗙 Exit")]
    Exit,
}

#[derive(Default)]
pub struct Menu {
    pub tab_current: Tab,
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
                        [
                            crate::ui::root::MENU_PANEL_MIN_WIDTH - 10.0,
                            styles::BUTTON_HEIGHT,
                        ],
                        egui::Button::new(
                            RichText::new(format!("{}", tab))
                                .size(styles::COMPONENT_FONT_SIZE),
                        ),
                    )
                    .clicked()
                {
                    self.tab_current = tab;
                }
                ui.add_space(2.0);
            }
        });
    }
}
