use crate::context::Context;
use crate::ui::styles;
use egui::{Grid, RichText};
use xailyser_common::messages::ClientRequest;

#[derive(Default)]
pub struct SettingsComponent {
    reboot_requested: bool, // To show confirmation
}

impl SettingsComponent {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.vertical_centered_justified(|ui| {
            ui.label(
                RichText::new("Settings")
                    .size(styles::COMPONENT_HEADING_FONT_SIZE)
                    .strong(),
            );
        });

        ui.add_space(styles::DEFAULT_SPACE);

        Grid::new("Settings.Grid")
            .num_columns(4)
            .spacing(styles::GRID_SPACING)
            .show(ui, |ui| {
                self.reboot_view(ui, ctx);
            });
    }

    fn reboot_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.label(
            RichText::new("Restart the server:")
                .color(egui::Color32::WHITE)
                .size(styles::COMPONENT_FONT_SIZE),
        );

        if !self.reboot_requested {
            if ui
                .add_sized(
                    [styles::BUTTON_WIDTH, styles::BUTTON_HEIGHT],
                    egui::Button::new(
                        RichText::new("Apply").size(styles::COMPONENT_FONT_SIZE),
                    ),
                )
                .clicked()
            {
                self.reboot_requested = true;
            }
        } else {
            if ui
                .add_sized(
                    [styles::BUTTON_WIDTH, styles::BUTTON_HEIGHT],
                    egui::Button::new(
                        RichText::new("CONFIRM").size(styles::COMPONENT_FONT_SIZE),
                    ),
                )
                .clicked()
            {
                self.reboot_requested = false;

                if let Err(err) = ctx.ui_tx.try_send(ClientRequest::Reboot) {
                    log::error!("Failed to send command (Reboot): {}", err);
                } else {
                    log::info!("UI -> WS: Sent reboot command.");
                }
            }

            if ui
                .add_sized(
                    [styles::BUTTON_WIDTH, styles::BUTTON_HEIGHT],
                    egui::Button::new(
                        RichText::new("Cancel").size(styles::COMPONENT_FONT_SIZE),
                    ),
                )
                .clicked()
            {
                self.reboot_requested = false;
            }
        }

        ui.label(
            RichText::new("*")
                .size(styles::COMPONENT_FONT_SIZE),
        ).on_hover_text("After confirmation, you may not receive a message about the reboot.\nMonitor the server status.");
        ui.end_row();
    }
}
