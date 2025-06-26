use crate::context::Context;
use crate::net::inspector::ProtocolsRegistered;
use crate::ui::styles;
use crate::ui::tabs::Tab;
use egui::{Grid, RichText, ScrollArea};
use strum::IntoEnumIterator;

#[derive(Default)]
pub struct StatsTab;

impl StatsTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        self.tab_heading(ui);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.columns(2, |column| {
                    column[0].vertical(|ui| {
                        self.protocols_view(ui, ctx);
                    });

                    column[1].vertical(|ui| {
                        self.main_statistics_view(ui, ctx);
                    });
                });
            });
    }

    fn main_statistics_view(&self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.heading(format!("{}.", t!("Tab.Stats.Main.Header")));
        Grid::new("StatsMain")
            .striped(false)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(format!("{}:", t!("Tab.Stats.Main.Captured")));
                ui.label(ctx.net_storage.inspector.ethernet.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Stats.Main.Records")));
                ui.label(ctx.net_storage.inspector.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Stats.Main.DeviceAliases")));
                ui.label(ctx.net_storage.devices.list.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Stats.Main.DevicesFound")));
                ui.label(ctx.net_storage.devices.aliases.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Stats.Main.ConnectionProfiles")));
                ui.label(ctx.profiles_storage.profiles.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Stats.Main.Ports")));
                ui.label(ctx.net_storage.lookup.port_service.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Stats.Main.Vendors")));
                ui.label(ctx.net_storage.lookup.vendors_amount.to_string());
                ui.end_row();
            });
    }

    fn protocols_view(&self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.heading(format!("{}:", t!("Tab.Stats.Protocols.Header")));
        Grid::new("Stats.Protocols.Grid")
            .striped(false)
            .num_columns(2)
            .show(ui, |ui| {
                for protocol in ProtocolsRegistered::iter() {
                    ui.label(format!("{protocol}:"));
                    ui.label(format!(
                        "{}",
                        ctx.net_storage.inspector.records_captured(&protocol)
                    ));
                    ui.end_row();
                }
            });
    }

    fn tab_heading(&self, ui: &mut egui::Ui) {
        ui.add_space(styles::space::TAB);
        ui.heading(
            RichText::new(Tab::Stats.to_string().as_str()).size(styles::heading::HUGE),
        );
    }
}
