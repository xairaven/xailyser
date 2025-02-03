use crate::context::Context;
use crate::ui;
use crate::ui::components::settings::SettingsComponent;
use crate::ui::components::status::StatusComponent;
use crate::ui::components::Tab;
use crate::ui::styles;
use egui::{CentralPanel, RichText, SidePanel};
use std::thread::JoinHandle;
use strum::IntoEnumIterator;

const MENU_PANEL_MIN_WIDTH: f32 = ui::MIN_WINDOW_WIDTH * 0.25;

#[derive(Default)]
pub struct UiRoot {
    pub net_thread: Option<JoinHandle<()>>,

    tab_current: Tab,

    status_component: StatusComponent,
    settings_component: SettingsComponent,
}

impl UiRoot {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        SidePanel::left("MENU_PANEL")
            .resizable(false)
            .min_width(MENU_PANEL_MIN_WIDTH)
            .show_separator_line(true)
            .show_inside(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.heading(
                        RichText::new("Menu")
                            .size(styles::COMPONENT_HEADING_FONT_SIZE)
                            .strong(),
                    );
                });

                ui.add_space(10.0);

                ui.vertical_centered_justified(|ui| {
                    for tab in Tab::iter() {
                        if ui
                            .add_sized(
                                [MENU_PANEL_MIN_WIDTH - 10.0, styles::BUTTON_HEIGHT],
                                egui::Button::new(
                                    RichText::new(format!("{}", tab))
                                        .size(styles::COMPONENT_FONT_SIZE),
                                ),
                            )
                            .clicked()
                        {
                            self.tab_current = tab;
                        }
                        ui.add_space(2.0);
                    }
                });
            });

        CentralPanel::default().show_inside(ui, |ui| match self.tab_current {
            Tab::Status => {
                self.status_component.show(ui, ctx);
            },
            Tab::Settings => {
                self.settings_component.show(ui, ctx);
            },
            Tab::About => {
                todo!()
            },
            Tab::Exit => {
                todo!()
            },
        });
    }
}
