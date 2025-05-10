use crate::context::Context;
use crate::ui::styles;
use crate::ui::tabs::Tab;
use dpi::protocols::ProtocolId;
use dpi::protocols::http::HttpDto;
use egui::{Grid, RichText, ScrollArea};
use strum::IntoEnumIterator;

pub struct InspectorTab {
    protocol_chosen: ProtocolId,
    page: usize,
}

impl Default for InspectorTab {
    fn default() -> Self {
        Self {
            protocol_chosen: ProtocolId::Arp,
            page: 1,
        }
    }
}

impl InspectorTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        self.tab_heading(ui, ctx);

        match self.protocol_chosen {
            ProtocolId::Arp => self.arp_view(ui, ctx),
            ProtocolId::DHCPv4 => self.dhcpv4_view(ui, ctx),
            ProtocolId::DHCPv6 => self.dhcpv6_view(ui, ctx),
            ProtocolId::DNS => self.dns_view(ui, ctx),
            ProtocolId::Ethernet => self.ethernet_view(ui, ctx),
            ProtocolId::HTTP => self.http_view(ui, ctx),
            ProtocolId::ICMPv4 => {},
            ProtocolId::ICMPv6 => {},
            ProtocolId::IPv4 => {},
            ProtocolId::IPv6 => {},
            ProtocolId::TCP => {},
            ProtocolId::UDP => {},
        };
    }

    fn protocol_view<T, F>(
        &mut self, ui: &mut egui::Ui, storage: &mut Vec<T>, grid_id: &str,
        num_columns: usize, headings: &[&str], mut render_row: F,
    ) where
        F: FnMut(&mut egui::Ui, usize, &T),
    {
        if self.clear_pages_buttons(ui, storage) {
            return;
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
                        for (id, packet) in
                            Self::page_slice(storage, self.page).iter().enumerate()
                        {
                            render_row(
                                ui,
                                (self.page - 1)
                                    .saturating_mul(Self::PAGE_SIZE)
                                    .saturating_add(id + 1),
                                packet,
                            );
                            ui.end_row();
                        }
                    });
            });
    }

    pub fn arp_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let storage = &mut ctx.net_storage.inspector.arp;
        self.protocol_view(
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
            |ui, id, packet| {
                ui.label(id.to_string());
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
        self.protocol_view(
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
            |ui, id, packet| {
                ui.label(id.to_string());
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
        self.protocol_view(
            ui,
            storage,
            "Inspector.DHCPv6.Packets",
            2,
            &[
                "Tab.Inspector.Label.Number",
                "Tab.Inspector.Protocol.DHCPv6.MessageType",
            ],
            |ui, id, packet| {
                ui.label(id.to_string());
                ui.label(packet.message_type.to_string());
            },
        );
    }

    pub fn dns_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let storage = &mut ctx.net_storage.inspector.dns;
        if self.clear_pages_buttons(ui, storage) {
            return;
        }

        // Table
        ScrollArea::both()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                // Data rows
                for (index, packet) in
                    Self::page_slice(storage, self.page).iter().enumerate()
                {
                    let record_number = (self.page - 1)
                        .saturating_mul(Self::PAGE_SIZE)
                        .saturating_add(index + 1);

                    ui.collapsing(format!("DNS Packet #{}", record_number), |ui| {
                        Grid::new(format!("DNS-Headers-{}", record_number))
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
                            Grid::new(format!("DNS-Headers-Question-{}", record_number))
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
                            record_number,
                            "Answer",
                            "Tab.Inspector.Protocol.DNS.Answer",
                            &packet.answer_section,
                        );
                        Self::dns_record_view(
                            ui,
                            record_number,
                            "Authority",
                            "Tab.Inspector.Protocol.DNS.Authority",
                            &packet.authority_section,
                        );
                        Self::dns_record_view(
                            ui,
                            record_number,
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
        self.protocol_view(
            ui,
            storage,
            "Inspector.Ethernet.Packets",
            7,
            &[
                "Tab.Inspector.Label.Number",
                "Tab.Inspector.Protocol.Ethernet.MacSender",
                "Tab.Inspector.Protocol.Ethernet.MacTarget",
                "Tab.Inspector.Protocol.IpSender",
                "Tab.Inspector.Protocol.IpTarget",
            ],
            |ui, id, package| {
                let packet = &package.0;
                let locator = &package.1;
                let (source_ip, target_ip) = locator.ip_to_string();

                ui.label(id.to_string());
                ui.label(packet.destination_mac.to_string());
                ui.label(packet.source_mac.to_string());
                ui.label(source_ip);
                ui.label(target_ip);
            },
        );
    }

    pub fn http_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let storage = &mut ctx.net_storage.inspector.http;
        if self.clear_pages_buttons(ui, storage) {
            return;
        }

        // Table
        ScrollArea::both()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                // Data rows
                for (index, (packet, locator)) in
                    Self::page_slice(storage, self.page).iter().enumerate()
                {
                    let record_number = (self.page - 1)
                        .saturating_mul(Self::PAGE_SIZE)
                        .saturating_add(index + 1);

                    ui.collapsing(format!("HTTP Packet #{}", record_number), |ui| {
                        Grid::new(format!("HTTP-Packet-{}", record_number))
                            .striped(false)
                            .num_columns(4)
                            .show(ui, |ui| {
                                match packet {
                                    HttpDto::Request(_) => {
                                        ui.label(styles::heading::grid(&t!(
                                            "Tab.Inspector.Protocol.HTTP.Request.Method"
                                        )));
                                        ui.label(styles::heading::grid(&t!(
                                            "Tab.Inspector.Protocol.HTTP.Request.Target"
                                        )));
                                    },
                                    HttpDto::Response(_) => {
                                        ui.label(styles::heading::grid(&t!(
                                            "Tab.Inspector.Protocol.HTTP.Response.StatusCode"
                                        )));
                                        ui.label(styles::heading::grid(&t!(
                                            "Tab.Inspector.Protocol.HTTP.Response.Reason"
                                        )));
                                    },
                                }
                                ui.label(styles::heading::grid(&t!(
                                            "Tab.Inspector.Protocol.IpSender"
                                        )));
                                ui.label(styles::heading::grid(&t!(
                                            "Tab.Inspector.Protocol.IpTarget"
                                        )));
                                ui.label(styles::heading::grid(&t!(
                                            "Tab.Inspector.Protocol.MacSender"
                                        )));
                                ui.label(styles::heading::grid(&t!(
                                            "Tab.Inspector.Protocol.MacTarget"
                                        )));
                                ui.end_row();

                                let (source_ip, target_ip) = locator.ip_to_string();
                                match packet {
                                    HttpDto::Request(request) => {
                                        ui.label(request.method.to_string());
                                        ui.label(request.target.to_string());
                                    },
                                    HttpDto::Response(response) => {
                                        ui.label(response.status_code.to_string());
                                        ui.label(response.reason.to_string());
                                    },
                                }
                                ui.label(source_ip);
                                ui.label(target_ip);
                                ui.label(locator.mac.0.to_string());
                                ui.label(locator.mac.1.to_string());
                                ui.end_row();
                            });

                        let headers = match packet {
                            HttpDto::Request(value) => &value.headers,
                            HttpDto::Response(value) => &value.headers,
                        };
                        if !headers.is_empty() {
                            ui.label(styles::heading::grid(&t!(
                                            "Tab.Inspector.Protocol.HTTP.Headers"
                                        )));
                            Grid::new(format!("HTTP-Headers-{}", record_number))
                                .striped(false)
                                .num_columns(2)
                                .show(ui, |ui| {
                                    for header in headers {
                                        ui.label(&header.0);
                                        ui.label(&header.1);
                                        ui.end_row();
                                    }
                                });
                        }
                    });
                }
            });
    }

    fn tab_heading(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
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
                        "\t{}: {}, {}: {}",
                        t!("Tab.Inspector.Label.Captured"),
                        ctx.net_storage.inspector.ethernet.len(),
                        t!("Tab.Inspector.Label.Records"),
                        ctx.net_storage.inspector.len(),
                    ))
                    .italics(),
                );
            });

            columns[RIGHT_COLUMN].with_layout(
                egui::Layout::right_to_left(egui::Align::Min),
                |ui| {
                    if ui.button(t!("Button.Clear")).clicked() {
                        ctx.net_storage.inspector.clear();
                        self.page = 1;
                    }
                },
            );
        });
    }

    fn clear_pages_buttons<T>(
        &mut self, ui: &mut egui::Ui, storage: &mut Vec<T>,
    ) -> bool {
        let mut to_restart = false;

        Grid::new("").num_columns(8).striped(false).show(ui, |ui| {
            if storage.is_empty() {
                ui.label(format!("{}:", t!("Tab.Inspector.Label.Protocol")));
            }

            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ComboBox::from_id_salt("Combobox.Inspector.Protocols")
                    .selected_text(self.protocol_chosen.to_string())
                    .show_ui(ui, |ui| {
                        for protocol in ProtocolId::iter() {
                            if ui
                                .selectable_value(
                                    &mut self.protocol_chosen,
                                    protocol,
                                    protocol.to_string(),
                                )
                                .clicked()
                            {
                                self.page = 1;
                                to_restart = true;
                            };
                        }
                    });
            });

            // Clear button or empty label
            if !storage.is_empty() {
                let total_pages = self.total_pages(storage.len());

                const LEFT_FAR: isize = -5;
                const LEFT: isize = -1;
                const RIGHT: isize = 1;
                const RIGHT_FAR: isize = 5;
                if ui
                    .add_enabled(
                        Self::can_go_to_page(self.page, LEFT_FAR, total_pages),
                        egui::Button::new("⏪"),
                    )
                    .clicked()
                {
                    self.page = (self.page as isize + LEFT_FAR) as usize;
                };
                if ui
                    .add_enabled(
                        Self::can_go_to_page(self.page, LEFT, total_pages),
                        egui::Button::new("◀"),
                    )
                    .clicked()
                {
                    self.page = (self.page as isize + LEFT) as usize;
                };
                ui.label(format!("Page {} of {} total", self.page, total_pages));
                if ui
                    .add_enabled(
                        Self::can_go_to_page(self.page, RIGHT, total_pages),
                        egui::Button::new("▶"),
                    )
                    .clicked()
                {
                    self.page = (self.page as isize + RIGHT) as usize;
                };
                if ui
                    .add_enabled(
                        Self::can_go_to_page(self.page, RIGHT_FAR, total_pages),
                        egui::Button::new("⏩"),
                    )
                    .clicked()
                {
                    self.page = (self.page as isize + RIGHT_FAR) as usize;
                };
                if ui.button(t!("Button.Clear")).clicked() {
                    self.page = 1;
                    storage.clear();
                    to_restart = true;
                }
            } else {
                ui.label(RichText::new(t!("Tab.Inspector.Label.Empty")).italics());
            }
            ui.end_row();
        });

        to_restart
    }

    const PAGE_SIZE: usize = 100;
    fn page_slice<T>(items: &[T], page: usize) -> &[T] {
        let start = (page - 1).saturating_mul(Self::PAGE_SIZE);
        let end = (start + Self::PAGE_SIZE).min(items.len());
        &items[start..end]
    }

    fn can_go_to_page(current_page: usize, delta: isize, total_pages: usize) -> bool {
        let target = current_page as isize + delta;
        (1..=(total_pages as isize)).contains(&target)
    }

    fn total_pages(&self, total_items: usize) -> usize {
        let pages = total_items.div_ceil(Self::PAGE_SIZE);
        usize::max(1, pages)
    }
}
