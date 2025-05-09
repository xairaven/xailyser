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
            ProtocolId::Arp => self.arp_view(ui, ctx),
            ProtocolId::DHCPv4 => self.dhcpv4_view(ui, ctx),
            ProtocolId::DHCPv6 => self.dhcpv6_view(ui, ctx),
            ProtocolId::DNS => self.dns_view(ui, ctx),
            ProtocolId::Ethernet => self.ethernet_view(ui, ctx),
            ProtocolId::HTTP => {},
            ProtocolId::ICMPv4 => {},
            ProtocolId::ICMPv6 => {},
            ProtocolId::IPv4 => {},
            ProtocolId::IPv6 => {},
            ProtocolId::TCP => {},
            ProtocolId::UDP => {},
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

    fn protocol_view<T, F>(
        ui: &mut egui::Ui, storage: &mut Vec<T>, grid_id: &str, num_columns: usize,
        headings: &[&str], mut render_row: F,
    ) where
        F: FnMut(&mut egui::Ui, usize, &T),
    {
        // Clear button or empty label
        if !storage.is_empty() {
            ui.vertical_centered_justified(|ui| {
                if ui.button(t!("Button.Clear")).clicked() {
                    storage.clear();
                }
            });
        } else {
            ui.label(RichText::new(t!("Tab.Inspector.Label.Empty")).italics());
        }

        // Table
        ScrollArea::both()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                Grid::new(grid_id)
                    .striped(true)
                    .num_columns(num_columns)
                    .show(ui, |ui| {
                        // Headings row
                        if !storage.is_empty() {
                            for &h in headings {
                                ui.label(styles::heading::grid(&t!(h)));
                            }
                            ui.end_row();
                        }

                        // Data rows
                        for (idx, packet) in storage.iter().enumerate() {
                            render_row(ui, idx, packet);
                            ui.end_row();
                        }
                    });
            });
    }

    pub fn arp_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let storage = &mut ctx.net_storage.inspector.arp;
        InspectorTab::protocol_view(
            ui,
            storage,
            "Inspector.Arp.Packets",
            6,
            &[
                "Tab.Inspector.Label.Number",
                "Tab.Inspector.Protocol.Arp.Operation",
                "Tab.Inspector.Protocol.Arp.IpSender",
                "Tab.Inspector.Protocol.Arp.IpTarget",
                "Tab.Inspector.Protocol.Arp.MacSender",
                "Tab.Inspector.Protocol.Arp.MacTarget",
            ],
            |ui, idx, packet| {
                ui.label(idx.to_string());
                ui.label(packet.operation.to_string());
                ui.label(packet.sender_ip.to_string());
                ui.label(packet.target_ip.to_string());
                ui.label(packet.sender_mac.to_string());
                ui.label(packet.target_mac.to_string());
            },
        );
    }

    pub fn dhcpv4_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let storage = &mut ctx.net_storage.inspector.dhcpv4;
        InspectorTab::protocol_view(
            ui,
            storage,
            "Inspector.DHCPv4.Packets",
            7,
            &[
                "Tab.Inspector.Label.Number",
                "Tab.Inspector.Protocol.DHCPv4.MessageType",
                "Tab.Inspector.Protocol.DHCPv4.OldClientAddress",
                "Tab.Inspector.Protocol.DHCPv4.NewClientAddress",
                "Tab.Inspector.Protocol.DHCPv4.ServerAddress",
                "Tab.Inspector.Protocol.DHCPv4.RelayAgentAddress",
                "Tab.Inspector.Protocol.DHCPv4.ClientMAC",
            ],
            |ui, idx, packet| {
                ui.label(idx.to_string());
                ui.label(packet.message_type.to_string());
                ui.label(packet.old_client_address.to_string());
                ui.label(packet.new_client_address.to_string());
                ui.label(packet.server_address.to_string());
                ui.label(packet.relay_agent_address.to_string());
                ui.label(packet.hardware_address_client.to_string());
            },
        );
    }

    pub fn dhcpv6_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let storage = &mut ctx.net_storage.inspector.dhcpv6;
        InspectorTab::protocol_view(
            ui,
            storage,
            "Inspector.DHCPv6.Packets",
            2,
            &[
                "Tab.Inspector.Label.Number",
                "Tab.Inspector.Protocol.DHCPv6.MessageType",
            ],
            |ui, idx, packet| {
                ui.label(idx.to_string());
                ui.label(packet.message_type.to_string());
            },
        );
    }

    pub fn dns_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let storage = &mut ctx.net_storage.inspector.dns;
        // Clear button or empty label
        if !storage.is_empty() {
            ui.vertical_centered_justified(|ui| {
                if ui.button(t!("Button.Clear")).clicked() {
                    storage.clear();
                }
            });
        } else {
            ui.label(RichText::new(t!("Tab.Inspector.Label.Empty")).italics());
        }

        // Table
        ScrollArea::both()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                // Data rows
                for (index, packet) in storage.iter().enumerate() {
                    ui.collapsing(format!("DNS Packet #{}", index + 1), |ui| {
                        Grid::new(format!("DNS-Headers-{}", index + 1))
                            .striped(false)
                            .num_columns(4)
                            .show(ui, |ui| {
                                ui.label(styles::heading::grid(&t!(
                                    "Tab.Inspector.Protocol.DNS.MessageType"
                                )));
                                ui.label(styles::heading::grid(&t!(
                                    "Tab.Inspector.Protocol.DNS.OperationCode"
                                )));
                                ui.label(styles::heading::grid(&t!(
                                    "Tab.Inspector.Protocol.DNS.AuthoritativeAnswer"
                                )));
                                ui.label(styles::heading::grid(&t!(
                                    "Tab.Inspector.Protocol.DNS.ResponseCode"
                                )));
                                ui.end_row();

                                ui.label(packet.message_type.to_string());
                                ui.label(packet.operation_code.to_string());
                                match packet.authoritative_answer {
                                    true => ui.label("+"),
                                    false => ui.label("-"),
                                };
                                ui.label(packet.response_code.to_string());
                                ui.end_row();
                            });

                        let question_section_len = packet.question_section.len();
                        if !question_section_len > 0 {
                            ui.label(format!(
                                "{} ({}: {})",
                                t!("Tab.Inspector.Protocol.DNS.Question"),
                                t!("Tab.Inspector.Protocol.DNS.Records"),
                                question_section_len
                            ));
                            Grid::new(format!("DNS-Headers-Question-{}", index + 1))
                                .striped(false)
                                .num_columns(4)
                                .show(ui, |ui| {
                                    ui.label(styles::heading::grid(&t!(
                                        "Tab.Inspector.Label.Number"
                                    )));
                                    ui.label(styles::heading::grid(&t!(
                                        "Tab.Inspector.Protocol.DNS.Question.Name"
                                    )));
                                    ui.label(styles::heading::grid(&t!(
                                        "Tab.Inspector.Protocol.DNS.Question.EntryType"
                                    )));
                                    ui.label(styles::heading::grid(&t!(
                                        "Tab.Inspector.Protocol.DNS.Question.Class"
                                    )));
                                    ui.end_row();

                                    for (index, question) in
                                        packet.question_section.iter().enumerate()
                                    {
                                        ui.label((index + 1).to_string());
                                        ui.label(question.name.to_string());
                                        ui.label(question.entry_type.to_string());
                                        ui.label(question.class.to_string());
                                        ui.end_row();
                                    }
                                });
                        }

                        Self::dns_record_view(
                            ui,
                            index,
                            "Answer",
                            "Tab.Inspector.Protocol.DNS.Answer",
                            &packet.answer_section,
                        );
                        Self::dns_record_view(
                            ui,
                            index,
                            "Authority",
                            "Tab.Inspector.Protocol.DNS.Authority",
                            &packet.authority_section,
                        );
                        Self::dns_record_view(
                            ui,
                            index,
                            "Additional",
                            "Tab.Inspector.Protocol.DNS.Additional",
                            &packet.additional_section,
                        );
                    });
                }
            });
    }

    fn dns_record_view(
        ui: &mut egui::Ui, packet_id: usize, section_id: &str, name: &str,
        section: &[dpi::protocols::dns::ResourceRecord],
    ) {
        let len = section.len();
        if len > 0 {
            ui.label(format!(
                "{} ({}: {})",
                t!(name),
                t!("Tab.Inspector.Protocol.DNS.Records"),
                len
            ));
            Grid::new(format!("DNS-Records-{}-{}", section_id, packet_id))
                .striped(false)
                .num_columns(6)
                .show(ui, |ui| {
                    ui.label(styles::heading::grid(&t!("Tab.Inspector.Label.Number")));
                    ui.label(styles::heading::grid(&t!(
                        "Tab.Inspector.Protocol.DNS.Record.Name"
                    )));
                    ui.label(styles::heading::grid(&t!(
                        "Tab.Inspector.Protocol.DNS.Record.RecordType"
                    )));
                    ui.label(styles::heading::grid(&t!(
                        "Tab.Inspector.Protocol.DNS.Record.Class"
                    )));
                    ui.label(styles::heading::grid(&t!(
                        "Tab.Inspector.Protocol.DNS.Record.TimeToLive"
                    )));
                    ui.label(styles::heading::grid(&t!(
                        "Tab.Inspector.Protocol.DNS.Record.Data"
                    )));
                    ui.end_row();

                    for (index, record) in section.iter().enumerate() {
                        ui.label((index + 1).to_string());
                        ui.label(record.name.to_string());
                        ui.label(record.record_type.to_string());
                        ui.label(record.class.to_string());
                        ui.label(record.time_to_live.to_string());
                        ui.label(record.data.to_string());
                        ui.end_row();
                    }
                });
        }
    }

    pub fn ethernet_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let storage = &mut ctx.net_storage.inspector.ethernet;
        InspectorTab::protocol_view(
            ui,
            storage,
            "Inspector.Ethernet.Packets",
            3,
            &[
                "Tab.Inspector.Label.Number",
                "Tab.Inspector.Protocol.Ethernet.MacSender",
                "Tab.Inspector.Protocol.Ethernet.MacTarget",
            ],
            |ui, id, packet| {
                ui.label(id.to_string());
                ui.label(packet.destination_mac.to_string());
                ui.label(packet.source_mac.to_string());
            },
        );
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
