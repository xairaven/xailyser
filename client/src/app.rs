use crate::ui;
use crate::ui::windows::Window;
use egui::ThemePreference;
use std::thread::JoinHandle;

#[derive(Default)]
pub struct App {
    pub net_thread: Option<JoinHandle<()>>,

    pub sub_windows: Vec<Box<dyn Window>>,
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
            ui::windows::main::show(self, ui);
        });
    }
}
