use crate::context::Context;
use crate::ui::components::auth::AuthFields;
use crate::ui::modals::connection_profiles::ProfileModal;
use crate::ui::modals::message::MessageModal;
use egui::{CentralPanel, Grid, RichText, ScrollArea, TopBottomPanel};

#[derive(Default)]
pub struct ConnectionProfilesComponent {
    is_opened: bool,
}

impl ConnectionProfilesComponent {
    pub fn show(
        &mut self, ui: &mut egui::Ui, ctx: &mut Context, auth_fields: &mut AuthFields,
    ) {
        let theme = ctx.client_settings.theme.into_aesthetix_theme();
        let profiles_amount = ctx.profiles_storage.profiles.len();

        TopBottomPanel::top("TOP_PANEL_CONNECTION_PROFILES")
            .frame(
                egui::Frame::new()
                    .inner_margin(theme.margin_style())
                    .fill(theme.bg_primary_color_visuals()),
            )
            .show(ui.ctx(), |ui| {
                ui.columns(3, |columns| {
                    const LEFT_COLUMN: usize = 0;
                    const MAIN_COLUMN: usize = 1;
                    const RIGHT_COLUMN: usize = 2;

                    columns[LEFT_COLUMN].with_layout(
                        egui::Layout::left_to_right(egui::Align::Min),
                        |ui| {
                            if ui.button("➕").on_hover_text("Add new profile").clicked()
                            {
                                let _ = ctx
                                    .modals_tx
                                    .try_send(Box::new(ProfileModal::operation_add()));
                            }
                            if ui.button("💾").on_hover_text("Save profiles").clicked()
                            {
                                self.save_profiles(ctx);
                            }
                        },
                    );

                    columns[MAIN_COLUMN].vertical_centered(|ui| {
                        ui.heading(
                            RichText::new(format!(
                                "☎ Connection Profiles: {}",
                                profiles_amount
                            ))
                            .size(25.0),
                        );
                    });

                    columns[RIGHT_COLUMN].with_layout(
                        egui::Layout::right_to_left(egui::Align::Min),
                        |ui| {
                            if ui.button("✖").clicked() {
                                self.close();
                            }
                        },
                    );
                });
            });

        CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .inner_margin(theme.margin_style())
                    .fill(theme.bg_primary_color_visuals()),
            )
            .show(ui.ctx(), |ui| {
                ui.columns(3, |columns| {
                    const MAIN_COLUMN: usize = 1;
                    columns[MAIN_COLUMN].vertical(|ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.vertical_centered_justified(|ui| {
                                if ctx.profiles_storage.profiles.is_empty() {
                                    ui.label("Empty");
                                    return;
                                }

                                let mut to_remove: Option<usize> = None;
                                for (index, profile) in
                                    ctx.profiles_storage.profiles.iter().enumerate()
                                {
                                    ui.separator();

                                    Grid::new(format!("ProfileView{index}"))
                                        .num_columns(3)
                                        .show(ui, |ui| {
                                            ui.label("Title:");
                                            ui.label(profile.title.to_string());
                                            if ui
                                                .button("✏")
                                                .on_hover_text("Edit Profile")
                                                .clicked()
                                            {
                                                let modal = ProfileModal::operation_edit(
                                                    index, profile,
                                                );
                                                let _ = ctx
                                                    .modals_tx
                                                    .try_send(Box::new(modal));
                                            }
                                            ui.end_row();

                                            ui.label("IP:");
                                            ui.label(profile.ip.to_string());
                                            if ui
                                                .button("🗑")
                                                .on_hover_text("Remove Profile")
                                                .clicked()
                                            {
                                                to_remove = Some(index);
                                            }
                                            ui.end_row();

                                            ui.label("Port:");
                                            ui.label(profile.port.to_string());
                                            if ui
                                                .button("▶")
                                                .on_hover_text("Proceed with Profile")
                                                .clicked()
                                            {
                                                *auth_fields = AuthFields {
                                                    ip: profile.ip.to_string(),
                                                    port: profile.port.to_string(),
                                                    password: profile
                                                        .password
                                                        .to_string(),
                                                };
                                                self.close();
                                            }
                                            ui.end_row();

                                            ui.label("Password:");
                                            ui.label(&profile.password);
                                            ui.end_row();
                                        });
                                }
                                // Removing
                                if let Some(index) = to_remove {
                                    ctx.profiles_storage.profiles.remove(index);
                                }
                            });
                        });
                    });
                });
            });
    }

    pub fn is_opened(&self) -> bool {
        self.is_opened
    }

    pub fn open(&mut self) {
        self.is_opened = true;
    }

    pub fn close(&mut self) {
        self.is_opened = false;
    }

    fn save_profiles(&mut self, ctx: &mut Context) {
        let modal = if let Err(err) = ctx.profiles_storage.save_to_file() {
            let mut text = format!(
                "Failed to save connection profiles.\nAdditional Info: {}.",
                err
            );
            if let Some(additional_info) = err.additional_info() {
                text.push_str(&format!("\n{}", additional_info));
            }
            MessageModal::error(&text)
        } else {
            MessageModal::info("Successfully saved connection profiles!")
        };
        let _ = ctx.modals_tx.try_send(Box::new(modal));
    }
}
