pub trait Window: Send + Sync {
    fn show(&mut self, ui: &egui::Ui);
    fn is_closed(&self) -> bool;
}

pub mod main;
pub mod message;
