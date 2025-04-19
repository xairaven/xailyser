use crate::context::Context;
use crate::profiles::Profile;
use crate::ui::components::auth::AuthFields;
use crate::ui::modals::connection_profiles::ProfileModal;
use crate::ui::modals::message::MessageModal;
use egui::{CentralPanel, Grid, RichText, ScrollArea, TopBottomPanel};

#[derive(Default)]
pub struct ConnectionProfilesComponent {
    is_opened: bool,

    // Internal field for removing profiles
    to_remove: Option<usize>,
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
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        if ctx.profiles_storage.profiles.is_empty() {
                            ui.label("Empty");
                            return;
                        }

                        for (index, profile) in
                            ctx.profiles_storage.profiles.iter().enumerate()
                        {
                            self.show_card(ui, ctx, auth_fields, index, profile);
                        }
                    });
                });
            });

        // Removing elements
        if let Some(index) = self.to_remove.take() {
            debug_assert!(profiles_amount > index);
            if profiles_amount <= index {
                let _ = ctx
                    .modals_tx
                    .try_send(Box::new(MessageModal::error("Failed to remove profile")));
                return;
            }
            ctx.profiles_storage.profiles.remove(index);
        }
    }

    fn show_card(
        &mut self, ui: &mut egui::Ui, ctx: &Context, auth_fields: &mut AuthFields,
        index: usize, profile: &Profile,
    ) {
        let theme = ctx.client_settings.theme.into_aesthetix_theme();
        egui::Frame::group(&egui::Style::default())
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(theme.margin_style())
            .corner_radius(5.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.columns(3, |columns| {
                        const LEFT_COLUMN: usize = 0;
                        const RIGHT_COLUMN: usize = 2;
                        columns[LEFT_COLUMN].with_layout(
                            egui::Layout::left_to_right(egui::Align::Min),
                            |ui| {
                                ui.heading(&profile.title);
                            },
                        );

                        columns[RIGHT_COLUMN].with_layout(
                            egui::Layout::right_to_left(egui::Align::Min),
                            |ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("✏ Edit").clicked() {
                                        let modal =
                                            ProfileModal::operation_edit(index, profile);
                                        let _ = ctx.modals_tx.try_send(Box::new(modal));
                                    }

                                    if ui.button("🗑 Delete").clicked() {
                                        self.to_remove = Some(index);
                                    }

                                    if ui.button("▶ Use").clicked() {
                                        *auth_fields = AuthFields {
                                            ip: profile.ip.to_string(),
                                            port: profile.port.to_string(),
                                            password: profile.password.to_string(),
                                        };
                                        self.close();
                                    }
                                });
                            },
                        );
                    });

                    Grid::new(format!("ConnectionProfile{}", index))
                        .striped(false)
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Address: ").strong());
                            ui.label(format!("{}", profile.ip));
                            ui.end_row();

                            ui.label(RichText::new("Port: ").strong());
                            ui.label(format!("{}", profile.port));
                            ui.end_row();
                        });
                });
            });

        ui.add_space(4.0);
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
