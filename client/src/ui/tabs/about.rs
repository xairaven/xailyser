use crate::context::Context;

// Constant so we can dynamically center the contents slightly above the center of the frame
const UPPER_CENTER: f32 = 5.0;

pub struct AboutTab {
    version: semver::Version,
}

impl Default for AboutTab {
    fn default() -> Self {
        Self {
            version: semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(
                semver::Version {
                    major: 0,
                    minor: 0,
                    patch: 1,
                    pre: Default::default(),
                    build: Default::default(),
                },
            ),
        }
    }
}

impl AboutTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let theme = ctx.active_theme.into_aesthetix_theme();

        let computed_upper_center = ui.ctx().screen_rect().height() / UPPER_CENTER;
        ui.add_space(computed_upper_center);
        ui.vertical_centered_justified(|ui| {
            ui.add(egui::Label::new(
                egui::RichText::new(format!("XAILYSER v{}", self.version)).size(30.0)
                    .color(theme.fg_success_text_color_visuals()),
            ));
            ui.label("Client-server analyser of network traffic on a home network");

            ui.add_space(20.0);

            ui.label("Developer: Alex Kovalov");

            ui.add_space(20.0);

            ui.hyperlink_to(
                "Check out the code on GitHub!",
                "https://github.com/xairaven/xailyser",
            );
            ui.hyperlink_to(
                "*Latest release*",
                "https://github.com/xairaven/xailyser/releases",
            );
        });
    }
}
