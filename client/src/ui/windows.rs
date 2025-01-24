use crate::context::Context;
use std::sync::{Arc, Mutex};

pub trait Window: Send + Sync {
    fn show(&mut self, ui: &egui::Ui, context: Arc<Mutex<Context>>);
    fn is_closed(&self) -> bool;
}

pub mod main;
pub mod message;
