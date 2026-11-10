pub const TIME_FORMAT: &str = "%m/%d %H:%M:%S";

pub mod colors {
    pub const SILENT: egui::Color32 = egui::Color32::GRAY;

    pub const ENABLED: egui::Color32 = egui::Color32::GREEN;
    pub const DISABLED: egui::Color32 = egui::Color32::RED;

    pub const FIELD_NOT_APPLIED: egui::Color32 = egui::Color32::RED;

    pub const OUTDATED: egui::Color32 = egui::Color32::RED;
    pub const OUTDATED_DARK: egui::Color32 = egui::Color32::DARK_RED;
    pub const UPDATED: egui::Color32 = egui::Color32::GREEN;
    pub const UPDATED_DARK: egui::Color32 = egui::Color32::DARK_GREEN;
}

pub mod heading {
    pub const NORMAL: f32 = 16.0;
    pub const HUGE: f32 = 25.0;

    pub fn grid(text: &str) -> egui::RichText {
        egui::RichText::new(text).underline()
    }

    pub fn normal(title: &str) -> egui::RichText {
        egui::RichText::new(format!("{title}:"))
            .size(NORMAL)
            .strong()
    }

    pub fn huge(title: &str) -> egui::RichText {
        egui::RichText::new(title).size(HUGE).strong()
    }
}

pub mod space {
    pub const SMALL: f32 = 10.0;
    pub const TAB: f32 = 13.0;
}

pub mod spacing {
    pub const GRID: f32 = 20.0;

    #[allow(dead_code)]
    pub fn with_temp<R>(
        ui: &mut egui::Ui, temp_spacing: egui::Vec2, f: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let default_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = temp_spacing;

        let result = f(ui);

        ui.spacing_mut().item_spacing = default_spacing;

        result
    }

    #[allow(dead_code)]
    pub fn with_temp_x<R>(
        ui: &mut egui::Ui, temp_spacing: f32, f: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let default_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = egui::vec2(temp_spacing, default_spacing.y);

        let result = f(ui);

        ui.spacing_mut().item_spacing = default_spacing;

        result
    }

    #[allow(dead_code)]
    pub fn with_temp_y<R>(
        ui: &mut egui::Ui, temp_spacing: f32, f: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let default_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = egui::vec2(default_spacing.x, temp_spacing);

        let result = f(ui);

        ui.spacing_mut().item_spacing = default_spacing;

        result
    }
}

pub mod text {
    use crate::ui::styles::colors;
    use egui::RichText;

    pub const SMALL: f32 = 10.0;

    pub fn is_enabled(is_enabled: bool) -> RichText {
        if is_enabled {
            RichText::new(t!("Button.State.Enabled")).color(colors::ENABLED)
        } else {
            RichText::new(t!("Button.State.Disabled")).color(colors::DISABLED)
        }
    }

    pub fn action(is_enabled: bool) -> RichText {
        if is_enabled {
            RichText::new(t!("Button.Action.Disable"))
        } else {
            RichText::new(t!("Button.Action.Enable"))
        }
    }

    pub fn field_not_applied(
        ui: &mut egui::Ui, mut label: RichText, is_not_applied: bool,
    ) -> egui::Response {
        if is_not_applied {
            label = label.color(colors::FIELD_NOT_APPLIED);
            ui.add(egui::Label::new(label))
                .on_hover_text(t!("Styles.Hover.FieldNotApplied"))
        } else {
            ui.add(egui::Label::new(label))
        }
    }
}

pub mod themes {
    use core::fmt;
    use egui_aesthetix::Aesthetix;
    use std::rc::Rc;
    use std::str::FromStr;
    use strum_macros::EnumIter;

    #[derive(Default, Copy, Clone, EnumIter, PartialEq, Eq, Ord, PartialOrd, Debug)]
    pub enum Preference {
        StandardDark,
        StandardLight,
        CarlDark,
        NordDark,
        NordLight,
        TokyoNight,

        #[default]
        TokyoNightStorm,
    }

    impl Preference {
        pub fn title(&self) -> &'static str {
            match self {
                Self::StandardDark => "Standard Dark",
                Self::StandardLight => "Standard Light",
                Self::CarlDark => "Carl Dark",
                Self::NordDark => "Nord Dark",
                Self::NordLight => "Nord Light",
                Self::TokyoNight => "Tokyo Night",
                Self::TokyoNightStorm => "Tokyo Night Storm",
            }
        }

        pub fn into_aesthetix_theme(self) -> Rc<dyn Aesthetix> {
            match self {
                Self::StandardDark => Rc::new(egui_aesthetix::themes::StandardDark),
                Self::StandardLight => Rc::new(egui_aesthetix::themes::StandardLight),
                Self::CarlDark => Rc::new(egui_aesthetix::themes::CarlDark),
                Self::NordDark => Rc::new(egui_aesthetix::themes::NordDark),
                Self::NordLight => Rc::new(egui_aesthetix::themes::NordLight),
                Self::TokyoNight => Rc::new(egui_aesthetix::themes::TokyoNight),
                Self::TokyoNightStorm => Rc::new(egui_aesthetix::themes::TokyoNightStorm),
            }
        }
    }

    impl fmt::Display for Preference {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let string = match self {
                Self::StandardDark => "standard_dark",
                Self::StandardLight => "standard_light",
                Self::CarlDark => "carl_dark",
                Self::NordDark => "nord_dark",
                Self::NordLight => "nord_light",
                Self::TokyoNight => "tokyo_night",
                Self::TokyoNightStorm => "tokyo_night_storm",
            };
            write!(f, "{string}")
        }
    }

    impl FromStr for Preference {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "standard_dark" => Ok(Self::StandardDark),
                "standard_light" => Ok(Self::StandardLight),
                "carl_dark" => Ok(Self::CarlDark),
                "nord_dark" => Ok(Self::NordDark),
                "nord_light" => Ok(Self::NordLight),
                "tokyo_night" => Ok(Self::TokyoNight),
                "tokyo_night_storm" => Ok(Self::TokyoNightStorm),
                _ => Err(()),
            }
        }
    }
}

pub fn invisible(ui: &mut egui::Ui) {
    ui.label("");
}
