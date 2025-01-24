use crate::app::App;
use crate::context::Context;
use crate::ui::windows::message::MessageWindow;
use crate::ui::windows::Window;
use crate::websocket;
use std::sync::{Arc, Mutex};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.label("Hello world!");

    if ui.button("CONNECT").clicked() {
        match websocket::core::connect(Arc::clone(&app.context)) {
            Ok(handle) => app.net_thread = Some(handle),
            Err(err) => {
                let mut message = format!("{}.", err);
                if let Some(additional) = err.additional_info() {
                    message.push_str(&format!(" {}", additional));
                }

                if let Ok(guard) = app.context.try_lock() {
                    let window = MessageWindow::error(&message);
                    let _ = guard.windows_tx.send(Box::new(window));
                }
            },
        }
    }

    // Sub window system
    if let Ok(guard) = app.context.try_lock() {
        if let Ok(sub_window) = guard.windows_rx.try_recv() {
            app.sub_windows.push(sub_window);
        }
    }

    show_opened_sub_windows(ui, Arc::clone(&app.context), &mut app.sub_windows);
}

fn show_opened_sub_windows(
    ui: &egui::Ui, context: Arc<Mutex<Context>>, windows: &mut Vec<Box<dyn Window>>,
) {
    let mut closed_windows: Vec<usize> = vec![];

    for (index, window) in windows.iter_mut().enumerate() {
        window.show(ui, Arc::clone(&context));

        if window.is_closed() {
            closed_windows.push(index);
        }
    }

    closed_windows.iter().for_each(|index| {
        windows.remove(*index);
    });
}
