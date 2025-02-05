use core::fmt;
use egui_aesthetix::Aesthetix;
use std::rc::Rc;
use std::str::FromStr;
use strum_macros::EnumIter;

#[derive(Default, Copy, Clone, EnumIter, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum ThemePreference {
    StandardDark,
    StandardLight,
    CarlDark,
    NordDark,
    NordLight,
    TokyoNight,

    #[default]
    TokyoNightStorm,
}

impl ThemePreference {
    pub fn title(&self) -> &'static str {
        match self {
            ThemePreference::StandardDark => "Standard Dark",
            ThemePreference::StandardLight => "Standard Light",
            ThemePreference::CarlDark => "Carl Dark",
            ThemePreference::NordDark => "Nord Dark",
            ThemePreference::NordLight => "Nord Light",
            ThemePreference::TokyoNight => "Tokyo Night",
            ThemePreference::TokyoNightStorm => "Tokyo Night Storm",
        }
    }

    pub fn into_aesthetix_theme(self) -> Rc<dyn Aesthetix> {
        match self {
            ThemePreference::StandardDark => {
                Rc::new(egui_aesthetix::themes::StandardDark)
            },
            ThemePreference::StandardLight => {
                Rc::new(egui_aesthetix::themes::StandardLight)
            },
            ThemePreference::CarlDark => Rc::new(egui_aesthetix::themes::CarlDark),
            ThemePreference::NordDark => Rc::new(egui_aesthetix::themes::NordDark),
            ThemePreference::NordLight => Rc::new(egui_aesthetix::themes::NordLight),
            ThemePreference::TokyoNight => Rc::new(egui_aesthetix::themes::TokyoNight),
            ThemePreference::TokyoNightStorm => {
                Rc::new(egui_aesthetix::themes::TokyoNightStorm)
            },
        }
    }
}

impl fmt::Display for ThemePreference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            ThemePreference::StandardDark => "standard_dark",
            ThemePreference::StandardLight => "standard_light",
            ThemePreference::CarlDark => "carl_dark",
            ThemePreference::NordDark => "nord_dark",
            ThemePreference::NordLight => "nord_light",
            ThemePreference::TokyoNight => "tokyo_night",
            ThemePreference::TokyoNightStorm => "tokyo_night_storm",
        };
        write!(f, "{}", string)
    }
}

impl FromStr for ThemePreference {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standard_dark" => Ok(ThemePreference::StandardDark),
            "standard_light" => Ok(ThemePreference::StandardLight),
            "carl_dark" => Ok(ThemePreference::CarlDark),
            "nord_dark" => Ok(ThemePreference::NordDark),
            "nord_light" => Ok(ThemePreference::NordLight),
            "tokyo_night" => Ok(ThemePreference::TokyoNight),
            "tokyo_night_storm" => Ok(ThemePreference::TokyoNightStorm),
            _ => Err(()),
        }
    }
}
