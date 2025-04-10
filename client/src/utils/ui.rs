#[allow(dead_code)]
pub fn with_temp_spacing<R>(
    ui: &mut egui::Ui, temp_spacing: egui::Vec2, f: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let default_spacing = ui.spacing().item_spacing;
    ui.spacing_mut().item_spacing = temp_spacing;

    let result = f(ui);

    ui.spacing_mut().item_spacing = default_spacing;

    result
}

#[allow(dead_code)]
pub fn with_temp_spacing_x<R>(
    ui: &mut egui::Ui, temp_spacing: f32, f: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let default_spacing = ui.spacing().item_spacing;
    ui.spacing_mut().item_spacing = egui::vec2(temp_spacing, default_spacing.y);

    let result = f(ui);

    ui.spacing_mut().item_spacing = default_spacing;

    result
}

#[allow(dead_code)]
pub fn with_temp_spacing_y<R>(
    ui: &mut egui::Ui, temp_spacing: f32, f: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let default_spacing = ui.spacing().item_spacing;
    ui.spacing_mut().item_spacing = egui::vec2(default_spacing.x, temp_spacing);

    let result = f(ui);

    ui.spacing_mut().item_spacing = default_spacing;

    result
}
