use crate::context::Context;
use crate::profiles::Profile;
use crate::ui::components::auth::AuthFields;
use crate::ui::modals::message::MessageModal;
use crate::ui::modals::{Modal, ModalFields};
use egui::{Grid, TextEdit, Ui};

pub struct ProfileModal {
    title: String,
    auth: AuthFields,

    modal: ModalFields,
    mode: ProfileOperationMode,
}

enum ProfileOperationMode {
    Add,
    Edit(usize),
}

impl Modal for ProfileModal {
    fn show_content(&mut self, ui: &mut Ui, ctx: &mut Context) {
        Grid::new("AuthenticationFields")
            .num_columns(2)
            .striped(false)
            .spacing([20.0, 20.0])
            .show(ui, |ui| {
                ui.label("Name:");
                ui.add(
                    TextEdit::singleline(&mut self.title).desired_width(f32::INFINITY),
                );
                ui.end_row();

                ui.label("IP:");
                ui.add(
                    TextEdit::singleline(&mut self.auth.ip).desired_width(f32::INFINITY),
                );
                ui.end_row();

                ui.label("Port:");
                ui.add(
                    TextEdit::singleline(&mut self.auth.port)
                        .desired_width(f32::INFINITY),
                );
                ui.end_row();

                ui.label("Password:");
                ui.add(
                    TextEdit::singleline(&mut self.auth.password)
                        .password(true)
                        .desired_width(f32::INFINITY),
                );
                ui.end_row();
            });

        ui.add_space(16.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                if ui.button("Save").clicked() {
                    self.save(ctx)
                }
            });
            columns[1].vertical_centered_justified(|ui| {
                if ui.button("Close").clicked() {
                    self.close()
                }
            });
        });
    }

    fn close(&mut self) {
        self.modal.is_open = false;
    }

    fn modal_fields(&self) -> &ModalFields {
        &self.modal
    }
}

impl ProfileModal {
    pub fn operation_add() -> Self {
        Self {
            title: String::new(),
            auth: Default::default(),

            modal: ModalFields::default()
                .with_title("➕ Add Profile".to_string())
                .with_width(300.0),

            mode: ProfileOperationMode::Add,
        }
    }

    pub fn operation_edit(index: usize, existing: &Profile) -> Self {
        Self {
            title: existing.title.clone(),
            auth: AuthFields {
                ip: existing.ip.to_string(),
                port: existing.port.to_string(),
                password: existing.password.clone(),
            },
            modal: ModalFields::default()
                .with_title("✏ Edit Profile".to_string())
                .with_width(300.0),
            mode: ProfileOperationMode::Edit(index),
        }
    }

    fn save(&mut self, ctx: &mut Context) {
        let fields = AuthFields {
            ip: self.auth.ip.clone(),
            port: self.auth.port.clone(),
            password: self.auth.password.clone(),
        };
        let profile = match fields.into_profile(&self.title) {
            Ok(value) => value,
            Err(err) => {
                let text = format!("Failed to save profile: {}", err);
                let _ = ctx.modals_tx.try_send(Box::new(MessageModal::error(&text)));
                return;
            },
        };

        match self.mode {
            ProfileOperationMode::Add => {
                ctx.profiles_storage.profiles.push(profile);
            },
            ProfileOperationMode::Edit(index) => {
                debug_assert!(ctx.profiles_storage.profiles.len() > index);
                if ctx.profiles_storage.profiles.len() <= index {
                    let _ = ctx.modals_tx.try_send(Box::new(MessageModal::error(
                        "Failed to edit profile",
                    )));
                    return;
                }
                ctx.profiles_storage.profiles[index] = profile;
            },
        }

        self.close();
    }
}
