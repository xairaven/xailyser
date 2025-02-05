use crate::commands::UiCommand;
use crate::context::Context;
use crate::ui::components::auth::AuthComponent;
use crate::ui::components::root::RootComponent;
use crate::ui::modals::Modal;
use crate::ui::themes::ThemePreference;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use xailyser_common::messages::ServerResponse;

pub struct App {
    context: Context,

    auth_component: AuthComponent,
    root_component: RootComponent,

    net_thread: Option<JoinHandle<()>>,

    modals: Vec<Box<dyn Modal>>,
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

            modals: vec![],
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

            // Getting modals from the channels (in context).
            if let Ok(modal) = self.context.modals_rx.try_recv() {
                self.modals.push(modal);
            }

            // Showing modals.
            self.show_opened_modals(ui);
        });

        self.process_server_responses();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(err) = self
            .context
            .ui_commands_tx
            .try_send(UiCommand::CloseConnection)
        {
            log::error!("Failed to send command (Close connection): {}", err);
        }
        self.context.shutdown_flag.store(true, Ordering::Release);

        if let Some(handle) = self.net_thread.take() {
            if handle.join().is_err() {
                log::error!("Failed to join net-thread handle.");
            }
        }
    }
}

impl App {
    fn show_opened_modals(&mut self, ui: &egui::Ui) {
        let mut closed_modals: Vec<usize> = vec![];

        for (index, modal) in self.modals.iter_mut().enumerate() {
            modal.show(ui);

            if modal.is_closed() {
                closed_modals.push(index);
            }
        }

        closed_modals.iter().for_each(|index| {
            self.modals.remove(*index);
        });
    }

    fn process_server_responses(&self) {
        match self.context.server_response_rx.try_recv() {
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
