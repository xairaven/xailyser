use crate::context::Context;
use crate::ui::styles;
use crate::ui::tabs::Tab;
use dpi::protocols::ProtocolId;
use egui::{Grid, RichText, ScrollArea};
use strum::IntoEnumIterator;

pub struct InspectorTab {
    protocol_chosen: ProtocolId,
}

impl Default for InspectorTab {
    fn default() -> Self {
        Self {
            protocol_chosen: ProtocolId::Arp,
        }
    }
}

impl InspectorTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        self.tab_heading(ui, ctx);
        self.select_view(ui);

        match self.protocol_chosen {
            ProtocolId::Ethernet => {},
            ProtocolId::Arp => self.arp_view(ui, ctx),
            ProtocolId::IPv4 => {},
            ProtocolId::IPv6 => {},
            ProtocolId::ICMPv4 => {},
            ProtocolId::ICMPv6 => {},
            ProtocolId::TCP => {},
            ProtocolId::UDP => {},
            ProtocolId::DHCPv4 => {},
            ProtocolId::DHCPv6 => {},
            ProtocolId::DNS => {},
            ProtocolId::HTTP => {},
        };
    }

    pub fn select_view(&mut self, ui: &mut egui::Ui) {
        Grid::new("").num_columns(2).striped(false).show(ui, |ui| {
            ui.label(format!("{}:", t!("Tab.Inspector.Label.Protocol")));

            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ComboBox::from_id_salt("Combobox.Inspector.Protocols")
                    .selected_text(self.protocol_chosen.to_string())
                    .show_ui(ui, |ui| {
                        for protocol in ProtocolId::iter() {
                            ui.selectable_value(
                                &mut self.protocol_chosen,
                                protocol,
                                protocol.to_string(),
                            );
                        }
                    });
            });

            ui.end_row();
        });
    }

    pub fn arp_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let storage = &mut ctx.net_storage.inspector.arp;
        if !storage.is_empty() {
            ui.vertical_centered_justified(|ui| {
                if ui.button(t!("Button.Clear")).clicked() {
                    storage.clear();
                }
            });
        } else {
            ui.label(RichText::new(t!("Tab.Inspector.Label.Empty")).italics());
        }

        ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                Grid::new("Inspector.Arp.Packets")
                    .striped(true)
                    .num_columns(6)
                    .show(ui, |ui| {
                        if !storage.is_empty() {
                            ui.label(styles::heading::grid(&t!(
                                "Tab.Inspector.Label.Number"
                            )));
                            ui.label(styles::heading::grid(&t!(
                                "Tab.Inspector.Protocol.Arp.Operation"
                            )));
                            ui.label(styles::heading::grid(&t!(
                                "Tab.Inspector.Protocol.Arp.IpSender"
                            )));
                            ui.label(styles::heading::grid(&t!(
                                "Tab.Inspector.Protocol.Arp.IpTarget"
                            )));
                            ui.label(styles::heading::grid(&t!(
                                "Tab.Inspector.Protocol.Arp.MacSender"
                            )));
                            ui.label(styles::heading::grid(&t!(
                                "Tab.Inspector.Protocol.Arp.MacTarget"
                            )));
                        }
                        ui.end_row();

                        for (index, packet) in storage.iter().enumerate() {
                            ui.label(format!("{}", index));
                            ui.label(format!("{}", packet.operation));
                            ui.label(format!("{}", packet.sender_ip));
                            ui.label(format!("{}", packet.sender_mac));
                            ui.label(format!("{}", packet.target_ip));
                            ui.label(format!("{}", packet.target_mac));
                            ui.end_row();
                        }
                    })
            });
    }

    fn tab_heading(&self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add_space(styles::space::TAB);

        ui.columns(3, |columns| {
            const LEFT_COLUMN: usize = 0;
            const RIGHT_COLUMN: usize = 2;
            columns[LEFT_COLUMN].horizontal_wrapped(|ui| {
                ui.heading(
                    RichText::new(Tab::Inspector.to_string().as_str())
                        .size(styles::heading::HUGE),
                );
                ui.label(
                    RichText::new(format!(
                        "\t{}: {}",
                        t!("Tab.Inspector.Label.Captured"),
                        ctx.net_storage.inspector.len()
                    ))
                    .italics(),
                );
            });

            columns[RIGHT_COLUMN].with_layout(
                egui::Layout::right_to_left(egui::Align::Min),
                |ui| {
                    if ui.button(t!("Button.Clear")).clicked() {
                        ctx.net_storage.inspector.clear();
                    }
                },
            );
        });
    }
}
