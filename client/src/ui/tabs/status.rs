use crate::context::Context;
use crate::net;
use crate::net::device::LocalDevice;
use crate::ui::components::throughput_settings::ThroughputSettings;
use crate::ui::modals::device::DeviceModal;
use crate::ui::modals::message::MessageModal;
use crate::ui::styles;
use crate::ui::tabs::Tab;
use egui::{Grid, RichText, ScrollArea};

pub struct StatusTab {
    throughput_settings: ThroughputSettings,
}

impl StatusTab {
    pub fn new(ctx: &Context) -> Self {
        Self {
            throughput_settings: ThroughputSettings::new(ctx),
        }
    }
}

impl StatusTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        if self.throughput_settings.is_opened() {
            self.throughput_settings.show(ui, ctx);
            return;
        }

        self.tab_heading(ui);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    self.plot_view(ui, ctx);
                });
                self.current_peak_stats_view(ui, ctx);
                self.pcap_save_view(ui, ctx);
                self.devices_view(ui, ctx);
            });
    }

    pub fn tab_heading(&mut self, ui: &mut egui::Ui) {
        ui.add_space(styles::space::TAB);
        ui.columns(2, |columns| {
            const LEFT_COLUMN: usize = 0;
            const RIGHT_COLUMN: usize = 1;
            columns[LEFT_COLUMN].vertical(|ui| {
                ui.heading(
                    RichText::new(Tab::Status.to_string().as_str())
                        .size(styles::heading::HUGE),
                );
            });

            columns[RIGHT_COLUMN].with_layout(
                egui::Layout::right_to_left(egui::Align::Min),
                |ui| {
                    if ui
                        .button("⚙")
                        .on_hover_text(t!("Tab.Status.Hover.PlotSettings"))
                        .clicked()
                    {
                        self.throughput_settings.open();
                    }
                },
            );
        });
    }

    fn plot_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        use egui_plot::Legend;
        use egui_plot::Line;
        use egui_plot::Plot;
        use egui_plot::PlotPoints;

        ctx.net_storage.speed.update_info(&ctx.client_settings);
        let throughput_line = Line::new(
            t!("Tab.Status.Legend.Throughput"),
            PlotPoints::from_iter(ctx.net_storage.speed.throughput_iter()),
        );
        let send_line = Line::new(
            t!("Tab.Status.Legend.Send"),
            PlotPoints::from_iter(ctx.net_storage.speed.send_iter()),
        );
        let receive_line = Line::new(
            t!("Tab.Status.Legend.Receive"),
            PlotPoints::from_iter(ctx.net_storage.speed.receive_iter()),
        );

        let plot_height = ui.available_height() / 1.8;
        Plot::new("SpeedFlow")
            .legend(Legend::default().follow_insertion_order(false))
            .allow_boxed_zoom(false)
            .allow_double_click_reset(false)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .default_x_bounds(
                0.0,
                ctx.client_settings.plot.display_window_seconds as f64 + 1.0,
            )
            .x_axis_label(format!(
                "{}, {}",
                t!("Tab.Status.Plot.Axis.X.Label"),
                t!("Tab.Status.Plot.Axis.X.Label.Suffix")
            ))
            .y_axis_label(format!(
                "{}, {}",
                t!("Tab.Status.Plot.Axis.Y.Label"),
                ctx.client_settings.plot.units
            ))
            .height(plot_height)
            .show(ui, |plot_ui| {
                plot_ui.line(throughput_line);
                plot_ui.line(send_line);
                plot_ui.line(receive_line);
            });
    }

    fn current_peak_stats_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.label(format!(
                "⬆ {}: {:.2}",
                t!("Tab.Status.NetworkData.Label.Sent"),
                ctx.net_storage.speed.peak_sent()
            ));
            ui.label(format!(
                "⬇ {}: {:.2}",
                t!("Tab.Status.NetworkData.Label.Received"),
                ctx.net_storage.speed.peak_received()
            ));
            ui.label(format!(
                "🔀 {}: {:.2}",
                t!("Tab.Status.NetworkData.Label.Throughput"),
                ctx.net_storage.speed.peak_throughput()
            ));
            ui.label(format!(
                "{} ({}):",
                t!("Tab.Status.NetworkData.Label.Peak"),
                ctx.client_settings.plot.units
            ));
        });
    }

    fn pcap_save_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        if !ctx.net_storage.raw.is_empty() {
            Grid::new("UnparsedFramesControls")
                .num_columns(3)
                .striped(false)
                .show(ui, |ui| {
                    ui.label(format!(
                        "Unparsed Frames: {}",
                        ctx.net_storage.raw.amount()
                    ));
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
                            .add_filter(
                                net::PCAP_FILTER_NAME,
                                net::PCAP_FILTER_EXTENSIONS,
                            )
                            .save_file()
                        {
                            if let Err(err) =
                                ctx.net_storage.raw.save_pcap(path, link_type)
                            {
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

    fn devices_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.columns(2, |columns| {
            columns[0].horizontal(|ui| {
                ui.heading(format!("{}:", t!("Tab.Status.Devices.Heading")));
                if ctx.net_storage.devices.list.is_empty() {
                    ui.label(t!("Tab.Status.Devices.Empty"));
                }
            });

            columns[1].with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui
                    .button(t!("Tab.Status.Devices.Button.SaveAliases"))
                    .clicked()
                {
                    let modal = if let Err(err) =
                        ctx.net_storage.devices.save_aliases_to_file()
                    {
                        let mut text = format!(
                            "{}\n{}: {}.",
                            t!("Tab.Status.Devices.Modal.ErrorSave"),
                            t!("Error.AdditionalInfo"),
                            err
                        );
                        if let Some(additional_info) = err.additional_info() {
                            text.push_str(&format!("\n{}", additional_info));
                        }
                        MessageModal::error(&text)
                    } else {
                        MessageModal::info(&t!("Tab.Status.Devices.Modal.Success"))
                    };
                    let _ = ctx.modals_tx.try_send(Box::new(modal));
                }
            });
        });

        if ctx.net_storage.devices.list.is_empty() {
            return;
        }

        ui.vertical_centered_justified(|ui| {
            for (index, device) in ctx.net_storage.devices.list.iter().enumerate() {
                self.device_view(ui, ctx, device, index + 1);
            }
        });
    }

    fn device_view(
        &mut self, ui: &mut egui::Ui, ctx: &Context, device: &LocalDevice, index: usize,
    ) {
        let theme = ctx.client_settings.theme.into_aesthetix_theme();
        egui::Frame::group(&egui::Style::default())
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(theme.margin_style())
            .corner_radius(5.0)
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].vertical(|ui| {
                        if let Some(name) =
                            ctx.net_storage.devices.aliases.get(&device.mac)
                        {
                            ui.heading(name);
                        } else {
                            ui.heading(format!(
                                "{} #{}",
                                t!("Tab.Status.Devices.DeviceGeneric"),
                                index
                            ));
                        }
                    });

                    columns[1].with_layout(
                        egui::Layout::right_to_left(egui::Align::Min),
                        |ui| {
                            if ui
                                .button("✏")
                                .on_hover_text(t!("Tab.Status.Devices.Device.Edit"))
                                .clicked()
                            {
                                let _ = ctx.modals_tx.try_send(Box::new(
                                    DeviceModal::with_id(device.mac.clone(), ctx),
                                ));
                            }
                        },
                    );
                });

                ui.vertical(|ui| {
                    Grid::new(format!("DeviceCard{}", index))
                        .num_columns(2)
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label(
                                format!("{}:", t!("Tab.Status.Devices.Device.MAC"),),
                            );
                            ui.label(device.mac.to_string());
                            ui.end_row();

                            ui.label(format!(
                                "{}:",
                                t!("Tab.Status.Devices.Device.IPv4")
                            ));
                            ui.label(if device.ip.is_empty() {
                                "-".to_string()
                            } else {
                                device
                                    .ip
                                    .iter()
                                    .map(|ip| ip.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            });
                            ui.end_row();

                            ui.label(format!(
                                "{}:",
                                t!("Tab.Status.Devices.Device.IPv6")
                            ));
                            ui.label(if device.ipv6.is_empty() {
                                "-".to_string()
                            } else {
                                device
                                    .ipv6
                                    .iter()
                                    .map(|ip| ip.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            });
                            ui.end_row();

                            ui.label(format!(
                                "{}:",
                                t!("Tab.Status.Devices.Device.Vendor")
                            ));
                            ui.label(
                                device
                                    .vendor
                                    .as_ref()
                                    .map(|vendor| vendor.full.clone())
                                    .unwrap_or(
                                        t!("Tab.Status.Devices.Device.Vendor.Unknown")
                                            .to_string(),
                                    ),
                            );
                            ui.end_row();
                        });
                });
            });

        ui.add_space(4.0);
    }
}
