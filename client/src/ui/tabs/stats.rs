use crate::context::Context;
use crate::ui::styles;
use crate::ui::tabs::Tab;
use egui::{Grid, RichText};

#[derive(Default)]
pub struct StatsTab;

impl StatsTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        self.tab_heading(ui);

        self.main_statistics_view(ui, ctx);
    }

    fn main_statistics_view(&self, ui: &mut egui::Ui, ctx: &mut Context) {
        Grid::new("StatsMain")
            .striped(false)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(format!("{}:", t!("Tab.Status.Main.Captured")));
                ui.label(ctx.net_storage.inspector.ethernet.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Status.Main.Records")));
                ui.label(ctx.net_storage.inspector.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Status.Main.DevicesFound")));
                ui.label(ctx.net_storage.devices.list.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Status.Main.DeviceAliases")));
                ui.label(ctx.net_storage.devices.aliases.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Status.Main.ConnectionProfiles")));
                ui.label(ctx.profiles_storage.profiles.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Status.Main.Ports")));
                ui.label(ctx.net_storage.lookup.port_service.len().to_string());
                ui.end_row();

                ui.label(format!("{}:", t!("Tab.Status.Main.Vendors")));
                ui.label(ctx.net_storage.lookup.vendors_amount.to_string());
                ui.end_row();
            });
    }

    fn tab_heading(&self, ui: &mut egui::Ui) {
        ui.add_space(styles::space::TAB);
        ui.heading(
            RichText::new(Tab::Stats.to_string().as_str()).size(styles::heading::HUGE),
        );
    }
}
