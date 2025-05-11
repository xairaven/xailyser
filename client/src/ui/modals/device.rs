use crate::context::Context;
use crate::ui::modals::{Modal, ModalFields};
use dpi::protocols::ethernet::mac::MacAddress;
use egui::{Grid, TextEdit, Ui};

pub struct DeviceModal {
    alias: String,
    mac: MacAddress,
    modal: ModalFields,
}

impl Modal for DeviceModal {
    fn show_content(&mut self, ui: &mut Ui, ctx: &mut Context) {
        Grid::new("DeviceNameEdit")
            .num_columns(2)
            .striped(false)
            .spacing([20.0, 20.0])
            .show(ui, |ui| {
                ui.label(format!("{}:", t!("Modal.DeviceAlias.Label.Alias")));
                ui.add(
                    TextEdit::singleline(&mut self.alias).desired_width(f32::INFINITY),
                );
                ui.end_row();

                ui.label(format!("{}:", t!("Modal.DeviceAlias.Label.MAC")));
                ui.label(self.mac.to_string());
                ui.end_row();
            });

        ui.add_space(16.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                if ui.button(t!("Button.Save")).clicked() {
                    self.save(ctx)
                }
            });
            columns[1].vertical_centered_justified(|ui| {
                if ui.button(t!("Button.Close")).clicked() {
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

impl DeviceModal {
    pub fn with_id(id: MacAddress, ctx: &Context) -> Self {
        Self {
            alias: match ctx.net_storage.devices.aliases.get(&id) {
                Some(alias) => alias.clone(),
                None => Default::default(),
            },
            mac: id,
            modal: ModalFields::default()
                .with_title(format!("✏ {}", t!("Modal.DeviceAlias.Title")))
                .with_width(300.0),
        }
    }

    fn save(&mut self, ctx: &mut Context) {
        ctx.net_storage
            .devices
            .aliases
            .insert(self.mac.clone(), self.alias.trim().to_owned());

        self.close();
    }
}
