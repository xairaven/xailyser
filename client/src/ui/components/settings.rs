use crate::context::Context;
use egui::{Grid, RichText, Vec2};
use xailyser_common::messages::ClientRequest;

const FONT_SIZE: f32 = 16.0;
const HEADING_FONT_SIZE: f32 = 26.0;

const SPACE: f32 = 10.0;
const SPACING: Vec2 = Vec2 { x: 10.0, y: 10.0 };

const BUTTON_WIDTH: f32 = 70.0;
const BUTTON_HEIGHT: f32 = 25.0;

#[derive(Default)]
pub struct SettingsComponent {
    reboot_requested: bool, // To show confirmation
}

impl SettingsComponent {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.vertical_centered_justified(|ui| {
            ui.label(RichText::new("Settings").size(HEADING_FONT_SIZE).strong());
        });

        ui.add_space(SPACE);

        Grid::new("Settings.Grid")
            .num_columns(3)
            .spacing(SPACING)
            .show(ui, |ui| {
                self.reboot_view(ui, ctx);
            });
    }

    fn reboot_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.label(
            RichText::new("Restart the server:")
                .color(egui::Color32::WHITE)
                .size(FONT_SIZE),
        );

        if !self.reboot_requested {
            if ui
                .add_sized(
                    [BUTTON_WIDTH, BUTTON_HEIGHT],
                    egui::Button::new(RichText::new("Apply").size(FONT_SIZE)),
                )
                .clicked()
            {
                self.reboot_requested = true;
            }
        } else {
            if ui
                .add_sized(
                    [BUTTON_WIDTH, BUTTON_HEIGHT],
                    egui::Button::new(RichText::new("CONFIRM").size(FONT_SIZE)),
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
                    [BUTTON_WIDTH, BUTTON_HEIGHT],
                    egui::Button::new(RichText::new("Cancel").size(FONT_SIZE)),
                )
                .clicked()
            {
                self.reboot_requested = false;
            }
        }
        ui.end_row();
    }
}
