use crate::context::Context;
use crate::net;
use crate::ui::modals::message::MessageModal;

#[derive(Default)]
pub struct StatusTab {}

impl StatusTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.label("PLACEHOLDER");

        if !ctx.net_storage.raw.is_empty() {
            ui.horizontal(|ui| {
                ui.label(format!("Unparsed Frames: {}", ctx.net_storage.raw.amount()));
                if ui.button("Save .pcap").clicked() {
                    let link_type = match ctx.settings_server.link_type {
                        Some(value) => value,
                        None => {
                            MessageModal::error(&t!("Error.FailedUnpackLinkType"))
                                .try_send_by(&ctx.modals_tx);
                            return;
                        },
                    };

                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter(net::PCAP_FILTER_NAME, net::PCAP_FILTER_EXTENSIONS)
                        .save_file()
                    {
                        if let Err(err) = ctx.net_storage.raw.save_pcap(path, link_type) {
                            MessageModal::error(&format!(
                                "{}: {}",
                                &t!("Error.Pcap"),
                                err
                            ))
                            .try_send_by(&ctx.modals_tx);
                        }
                    }
                }
                if ui.button("Reset").clicked() {
                    ctx.net_storage.raw.clear();
                }
            });
        }
    }
}
