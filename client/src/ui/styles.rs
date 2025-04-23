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

    pub fn normal(title: &str) -> egui::RichText {
        egui::RichText::new(format!("{}:", title))
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
                Preference::StandardDark => "Standard Dark",
                Preference::StandardLight => "Standard Light",
                Preference::CarlDark => "Carl Dark",
                Preference::NordDark => "Nord Dark",
                Preference::NordLight => "Nord Light",
                Preference::TokyoNight => "Tokyo Night",
                Preference::TokyoNightStorm => "Tokyo Night Storm",
            }
        }

        pub fn into_aesthetix_theme(self) -> Rc<dyn Aesthetix> {
            match self {
                Preference::StandardDark => Rc::new(egui_aesthetix::themes::StandardDark),
                Preference::StandardLight => {
                    Rc::new(egui_aesthetix::themes::StandardLight)
                },
                Preference::CarlDark => Rc::new(egui_aesthetix::themes::CarlDark),
                Preference::NordDark => Rc::new(egui_aesthetix::themes::NordDark),
                Preference::NordLight => Rc::new(egui_aesthetix::themes::NordLight),
                Preference::TokyoNight => Rc::new(egui_aesthetix::themes::TokyoNight),
                Preference::TokyoNightStorm => {
                    Rc::new(egui_aesthetix::themes::TokyoNightStorm)
                },
            }
        }
    }

    impl fmt::Display for Preference {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let string = match self {
                Preference::StandardDark => "standard_dark",
                Preference::StandardLight => "standard_light",
                Preference::CarlDark => "carl_dark",
                Preference::NordDark => "nord_dark",
                Preference::NordLight => "nord_light",
                Preference::TokyoNight => "tokyo_night",
                Preference::TokyoNightStorm => "tokyo_night_storm",
            };
            write!(f, "{}", string)
        }
    }

    impl FromStr for Preference {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "standard_dark" => Ok(Preference::StandardDark),
                "standard_light" => Ok(Preference::StandardLight),
                "carl_dark" => Ok(Preference::CarlDark),
                "nord_dark" => Ok(Preference::NordDark),
                "nord_light" => Ok(Preference::NordLight),
                "tokyo_night" => Ok(Preference::TokyoNight),
                "tokyo_night_storm" => Ok(Preference::TokyoNightStorm),
                _ => Err(()),
            }
        }
    }
}
