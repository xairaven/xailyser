use crate::ui::auth::AuthRoot;
use crate::ui::root::UiRoot;
use crate::ui::windows::Window;
use crossbeam::channel::{unbounded, Receiver};
use egui::ThemePreference;

pub struct App {
    auth_root: AuthRoot,
    root: UiRoot,

    sub_windows: Vec<Box<dyn Window>>,

    windows_rx: Receiver<Box<dyn Window>>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, theme: ThemePreference) -> Self {
        cc.egui_ctx
            .options_mut(|options| options.theme_preference = theme);

        let (windows_tx, windows_rx) = unbounded::<Box<dyn Window>>();

        Self {
            windows_rx,

            auth_root: AuthRoot::new(windows_tx.clone()),
            root: UiRoot::new(windows_tx.clone()),

            sub_windows: Vec::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // If not authenticated, showing `auth` window.
            if !self.auth_root.authenticated() {
                self.auth_root.show(ui);
            }

            // If authenticated after showing auth component, then showing UI root.
            if self.auth_root.authenticated() {
                // Passing net thread to the root component
                if self.auth_root.net_thread.is_some() {
                    self.root.net_thread = self.auth_root.net_thread.take();
                }

                // Showing the root window.
                self.root.show(ui);
            }

            // Getting sub-windows from the channels (in context).
            if let Ok(sub_window) = self.windows_rx.try_recv() {
                self.sub_windows.push(sub_window);
            }

            // Showing sub-windows.
            self.show_opened_sub_windows(ui);
        });
    }
}

impl App {
    fn show_opened_sub_windows(&mut self, ui: &egui::Ui) {
        let mut closed_windows: Vec<usize> = vec![];

        for (index, window) in self.sub_windows.iter_mut().enumerate() {
            window.show(ui);

            if window.is_closed() {
                closed_windows.push(index);
            }
        }

        closed_windows.iter().for_each(|index| {
            self.sub_windows.remove(*index);
        });
    }
}
