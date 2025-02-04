use crate::context::Context;
use egui::Grid;
use xailyser_common::messages::ClientRequest;

#[derive(Default)]
pub struct SettingsTab {
    reboot_requested: bool, // To show confirmation
}

impl SettingsTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        Grid::new("Settings.Grid").num_columns(4).show(ui, |ui| {
            self.reboot_view(ui, ctx);
        });
    }

    fn reboot_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.label("Restart the server:");

        if !self.reboot_requested {
            if ui.button("Apply").clicked() {
                self.reboot_requested = true;
            }
        } else {
            if ui.button("CONFIRM").clicked() {
                self.reboot_requested = false;

                if let Err(err) = ctx.ui_tx.try_send(ClientRequest::Reboot) {
                    log::error!("Failed to send command (Reboot): {}", err);
                } else {
                    log::info!("UI -> WS: Sent reboot command.");
                }
            }

            if ui.button("Cancel").clicked() {
                self.reboot_requested = false;
            }
        }

        ui.label("*").on_hover_text("After confirmation, you may not receive a message about the reboot.\nMonitor the server status.");
        ui.end_row();
    }
}
