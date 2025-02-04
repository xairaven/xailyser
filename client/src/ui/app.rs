use crate::context::Context;
use crate::ui::components::auth::AuthComponent;
use crate::ui::components::root::RootComponent;
use crate::ui::themes::ThemePreference;
use crate::ui::windows::Window;
use std::thread::JoinHandle;
use xailyser_common::messages::ServerResponse;

pub struct App {
    context: Context,

    auth_component: AuthComponent,
    root_component: RootComponent,

    net_thread: Option<JoinHandle<()>>,

    sub_windows: Vec<Box<dyn Window>>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, theme: ThemePreference) -> Self {
        let context = Context::new(theme);

        cc.egui_ctx
            .set_style(context.active_theme.into_aesthetix_theme().custom_style());

        Self {
            context,

            net_thread: None,

            auth_component: Default::default(),
            root_component: Default::default(),

            sub_windows: vec![],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // If not authenticated, showing `auth` window.
            if !self.auth_component.authenticated() {
                self.auth_component.show(ui, &mut self.context);
            }

            // If authenticated after showing auth component, then showing UI root.
            if self.auth_component.authenticated() {
                // Taking net-thread join handle from the auth component
                if self.auth_component.net_thread.is_some() {
                    self.net_thread = self.auth_component.net_thread.take();
                }

                // Showing the root component.
                self.root_component.show(ui, &mut self.context);
            }

            // Getting sub-windows from the channels (in context).
            if let Ok(sub_window) = self.context.windows_rx.try_recv() {
                self.sub_windows.push(sub_window);
            }

            // Showing sub-windows.
            self.show_opened_sub_windows(ui);
        });

        self.process_server_responses();
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

    fn process_server_responses(&self) {
        match self.context.ws_rx.try_recv() {
            Ok(ServerResponse::InterfacesList(_)) => {
                todo!()
            },
            Ok(ServerResponse::SetInterfaceResult(_)) => {
                todo!()
            },
            Ok(ServerResponse::ChangePasswordResult(_)) => {
                todo!()
            },
            Ok(ServerResponse::Error(_)) => {
                todo!()
            },
            _ => {},
        }

        // self.windows_tx.try_send()
    }
}
