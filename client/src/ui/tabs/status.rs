use crate::context::Context;
use crate::net;
use crate::net::speed::SpeedUnitPerSecond;
use crate::ui::modals::message::MessageModal;
use egui::{DragValue, Grid, ScrollArea};
use strum::IntoEnumIterator;

pub struct StatusTab {
    display_window_seconds: u32,
    display_unit: SpeedUnitPerSecond,
}

impl StatusTab {
    pub fn new(ctx: &Context) -> Self {
        Self {
            display_window_seconds: ctx.client_settings.plot.display_window_seconds,
            display_unit: ctx.client_settings.plot.units.clone(),
        }
    }
}

impl StatusTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    self.plot_view(ui, ctx);
                    //self.plot_settings_view(ui, ctx);
                });
                self.pcap_save_view(ui, ctx);
            });
    }

    pub fn plot_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
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
            .height(plot_height)
            .show(ui, |plot_ui| {
                plot_ui.line(throughput_line);
                plot_ui.line(send_line);
                plot_ui.line(receive_line);
            });
    }

    pub fn plot_settings_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        Grid::new("Status.Tab.Settings.Plot.Grid")
            .striped(false)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(format!("{}:", t!("Tab.Status.Label.DisplayPeriod")));
                ui.add(
                    DragValue::new(&mut self.display_window_seconds)
                        .speed(1)
                        .range(1..=u32::MAX)
                        .suffix(format!(" {}", t!("Tab.Status.Suffix.DisplayInterval"))),
                );
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Status.Label.SpeedUnits")));
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    egui::ComboBox::from_id_salt("Settings.Status.SpeedUnits.ComboBox")
                        .selected_text(self.display_unit.to_string())
                        .show_ui(ui, |ui| {
                            for unit in SpeedUnitPerSecond::iter() {
                                let text = unit.to_string();
                                ui.selectable_value(&mut self.display_unit, unit, text);
                            }
                        });
                });
                ui.end_row();

                ui.horizontal_centered(|ui| {
                    if ui.button(t!("Button.Apply")).clicked() {
                        ctx.client_settings.plot.display_window_seconds =
                            self.display_window_seconds;
                        ctx.client_settings.plot.units = self.display_unit.clone();
                    }
                });

                ui.horizontal_centered(|ui| {
                    if ui.button(t!("Button.Save")).clicked() {
                        ctx.config.plot_display_window_seconds =
                            ctx.client_settings.plot.display_window_seconds;
                        ctx.config.plot_speed_units =
                            ctx.client_settings.plot.units.clone();

                        match ctx.config.save_to_file() {
                            Ok(_) => {
                                log::info!(
                                    "Plot Settings: Successfully saved client config."
                                );
                                MessageModal::info(&t!(
                                    "Message.Success.ClientConfigSaved"
                                ))
                                .try_send_by(&ctx.modals_tx);
                            },
                            Err(err) => {
                                log::error!(
                                    "Plot Settings: Failed to save client config: {}",
                                    err
                                );
                                MessageModal::error(&format!(
                                    "{} {}",
                                    t!("Error.FailedSaveClientConfigIntoFile"),
                                    err
                                ))
                                .try_send_by(&ctx.modals_tx);
                            },
                        };
                    }
                });
            });
    }

    pub fn pcap_save_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
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
