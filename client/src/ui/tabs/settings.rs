use crate::context::Context;
use crate::ui::themes::ThemePreference;
use egui::Grid;
use strum::IntoEnumIterator;
use xailyser_common::messages::ClientRequest;

#[derive(Default)]
pub struct SettingsTab {
    reboot_requested: bool, // To show confirmation
}

impl SettingsTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(20.0);
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    Grid::new("Settings.Grid")
                        .striped(false)
                        .num_columns(4)
                        .show(ui, |ui| {
                            self.theme_view(ui, ctx);
                            ui.end_row();

                            self.reboot_view(ui, ctx);
                            ui.end_row();
                        });
                },
            );
        });
    }

    fn theme_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(
            egui::RichText::new("Theme:").size(16.0).strong(),
        ));

        egui::ComboBox::from_id_salt("Settings.Theme.ComboBox")
            .width(200.0)
            .selected_text(ctx.active_theme.title())
            .show_ui(ui, |ui| {
                for theme in ThemePreference::iter() {
                    let res: egui::Response =
                        ui.selectable_value(&mut ctx.active_theme, theme, theme.title());
                    if res.changed() {
                        log::info!("Theme changed to {}", theme.title());
                        ui.ctx()
                            .set_style(theme.into_aesthetix_theme().custom_style());
                    }
                }
            });
    }

    fn reboot_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(
            egui::RichText::new("Restart the server:")
                .size(16.0)
                .strong(),
        ));

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
