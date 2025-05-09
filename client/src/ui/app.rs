use crate::config::Config;
use crate::context::Context;
use crate::ui::components::auth::AuthComponent;
use crate::ui::components::root::RootComponent;
use crate::ui::modals::Modal;
use crate::ws;
use crate::ws::request::UiClientRequest;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;

pub struct App {
    context: Context,

    auth_component: AuthComponent,
    root_component: RootComponent,

    net_thread: Option<JoinHandle<()>>,

    modals: Vec<Box<dyn Modal>>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        let ctx = Context::new(config);
        cc.egui_ctx
            .set_style(ctx.config.theme.into_aesthetix_theme().custom_style());

        Self {
            net_thread: None,

            auth_component: AuthComponent::new(&ctx),
            root_component: RootComponent::new(&ctx),

            modals: vec![],

            context: ctx,
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
                // Taking net-thread join handle from the auth component (Auth just happened)
                if self.auth_component.net_thread.is_some() {
                    self.net_thread = self.auth_component.net_thread.take();
                    self.root_component
                        .update_client_settings_info(&self.context);
                }
                // First connection time
                if self.context.heartbeat.last_sync.is_none() {
                    self.context.heartbeat.update();
                }

                // Heartbeat
                self.context.heartbeat.check(
                    &self.context.client_settings,
                    &self.context.ui_client_requests_tx,
                );

                // Showing the root component.
                self.root_component.show(ui, &mut self.context);

                // Logout from root component, if requested.
                if self.root_component.logout_requested() {
                    self.root_component.logout(&self.context);
                    self.auth_component.logout(&self.context);
                    self.context.logout();
                }
            }

            // Getting modals from the channels (in context).
            if let Ok(modal) = self.context.modals_rx.try_recv() {
                self.modals.push(modal);
            }

            // Showing modals.
            self.show_opened_modals(ui);
        });

        // Processing all responses
        while let Ok(response) = self.context.data_response_rx.try_recv() {
            ws::response::data(&mut self.context, response);
        }
        while let Ok(response) = self.context.server_response_rx.try_recv() {
            ws::response::process(&mut self.context, response);
        }
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(err) = self
            .context
            .ui_client_requests_tx
            .try_send(UiClientRequest::CloseConnection)
        {
            log::error!("Failed to send command (Close connection): {}", err);
        }
        log::info!("Shutdown started...");
        self.context.shutdown_flag.store(true, Ordering::Release);

        if let Some(handle) = self.net_thread.take() {
            if handle.join().is_err() {
                log::error!("Failed to join net-thread handle.");
            }
        }
        log::info!("Shutdown complete");
    }
}

impl App {
    fn show_opened_modals(&mut self, ui: &egui::Ui) {
        let mut closed_modals: Vec<usize> = vec![];

        for (index, modal) in self.modals.iter_mut().enumerate() {
            modal.show(ui, &mut self.context);

            if modal.is_closed() {
                closed_modals.push(index);
            }
        }

        closed_modals.iter().for_each(|index| {
            self.modals.remove(*index);
        });
    }
}
