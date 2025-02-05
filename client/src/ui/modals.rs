pub trait Modal: Send + Sync {
    fn show(&mut self, ui: &egui::Ui);
    fn is_closed(&self) -> bool;
}

pub mod message;
