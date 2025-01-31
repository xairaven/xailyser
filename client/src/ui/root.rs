use std::thread::JoinHandle;

#[derive(Default)]
pub struct UiRoot {
    pub net_thread: Option<JoinHandle<()>>,
}

impl UiRoot {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("Root. Dashboard");
    }
}
