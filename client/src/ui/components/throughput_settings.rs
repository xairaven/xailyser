use crate::context::Context;
use crate::net::speed::SpeedUnitPerSecond;
use crate::ui::modals::message::MessageModal;
use crate::ui::styles;
use crate::ui::styles::spacing;
use egui::{DragValue, Grid, RichText};
use strum::IntoEnumIterator;

pub struct ThroughputSettings {
    is_opened: bool,
    display_window_seconds: u32,
    display_unit: SpeedUnitPerSecond,
}

impl ThroughputSettings {
    pub fn new(ctx: &Context) -> Self {
        Self {
            is_opened: false,
            display_window_seconds: ctx.client_settings.plot.display_window_seconds,
            display_unit: ctx.client_settings.plot.units.clone(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        self.heading(ui);

        const GRID_COLUMNS: usize = 4;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(styles::space::SMALL);
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        spacing::with_temp_y(ui, spacing::GRID, |ui| {
                            Grid::new("Status.ThroughputSettings.Grid")
                                .striped(false)
                                .num_columns(GRID_COLUMNS)
                                .show(ui, |ui| {
                                    self.save_client_config_view(ui, ctx);
                                    ui.end_row();

                                    self.display_period_view(ui, ctx);
                                    ui.end_row();

                                    self.display_unit_view(ui, ctx);
                                    ui.end_row();
                                });
                        });
                    },
                );
            });
    }

    pub fn heading(&mut self, ui: &mut egui::Ui) {
        ui.add_space(styles::space::TAB);
        ui.columns(2, |columns| {
            const LEFT_COLUMN: usize = 0;
            const RIGHT_COLUMN: usize = 1;
            columns[LEFT_COLUMN].vertical(|ui| {
                ui.heading(
                    RichText::new(format!("📊 {}", t!("Tab.ThroughputSettings.Header")))
                        .size(styles::heading::HUGE),
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
    }

    fn save_client_config_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(styles::heading::normal(&t!(
            "Tab.ThroughputSettings.Label.SaveConfig"
        ))))
        .on_hover_text(t!("Tab.ThroughputSettings.Hover.SettingSavesConfig"));

        if ui.button(t!("Button.Save")).clicked() {
            ctx.config.plot_display_window_seconds =
                ctx.client_settings.plot.display_window_seconds;
            ctx.config.plot_speed_units = ctx.client_settings.plot.units.clone();

            match ctx.config.save_to_file() {
                Ok(_) => {
                    log::info!("Plot Settings: Successfully saved client config.");
                    MessageModal::info(&t!("Message.Success.ClientConfigSaved"))
                        .try_send_by(&ctx.modals_tx);
                },
                Err(err) => {
                    log::error!("Plot Settings: Failed to save client config: {}", err);
                    MessageModal::error(&format!(
                        "{} {}",
                        t!("Error.FailedSaveClientConfigIntoFile"),
                        err
                    ))
                    .try_send_by(&ctx.modals_tx);
                },
            };
        }
    }

    pub fn display_period_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label =
            styles::heading::normal(&t!("Tab.ThroughputSettings.Label.DisplayPeriod"));
        let not_applied = self.display_window_seconds
            != ctx.client_settings.plot.display_window_seconds;
        styles::text::field_not_applied(ui, label, not_applied);

        ui.add(
            DragValue::new(&mut self.display_window_seconds)
                .speed(1)
                .range(1..=u32::MAX)
                .suffix(format!(
                    " {}",
                    t!("Tab.ThroughputSettings.Suffix.DisplayInterval")
                )),
        );

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!(
                "Plot Settings: Display window changed to {}",
                self.display_window_seconds
            );
            ctx.client_settings.plot.display_window_seconds = self.display_window_seconds;
        }
        if ui.button("🔙").clicked() {
            self.display_window_seconds = ctx.client_settings.plot.display_window_seconds;
        }
    }

    pub fn display_unit_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label =
            styles::heading::normal(&t!("Tab.ThroughputSettings.Label.SpeedUnits"));
        let not_applied = self.display_unit != ctx.client_settings.plot.units;
        styles::text::field_not_applied(ui, label, not_applied);

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

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!(
                "Plot Settings: Speed units changed to {}",
                self.display_unit
            );
            ctx.client_settings.plot.units = self.display_unit.clone();
        }
        if ui.button("🔙").clicked() {
            self.display_unit = ctx.client_settings.plot.units.clone();
        }
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
}
