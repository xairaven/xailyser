use crate::context::Context;
use crate::net;
use crate::ui::components::throughput_settings::ThroughputSettings;
use crate::ui::modals::message::MessageModal;
use crate::ui::styles;
use crate::ui::tabs::Tab;
use egui::{RichText, ScrollArea};

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
