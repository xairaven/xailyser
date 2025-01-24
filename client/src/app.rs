use crate::context::Context;
use crate::ui::Resolution;
use egui::ThemePreference;

#[derive(Default)]
pub struct App {
    pub resolution: Resolution,

    pub context: Context,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, theme: ThemePreference) -> Self {
        cc.egui_ctx
            .options_mut(|options| options.theme_preference = theme);
        Default::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello World!");
        });
    }
}
