use crate::regkey::RegistryKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    Dark,
    Light,
}

impl ThemeVariant {
    pub fn invert(self) -> Self {
        match self {
            ThemeVariant::Dark => ThemeVariant::Light,
            ThemeVariant::Light => ThemeVariant::Dark,
        }
    }
}

pub fn set_theme(variant: ThemeVariant) {
    let value = match variant {
        ThemeVariant::Dark => 0,
        ThemeVariant::Light => 1,
    };

    let key = RegistryKey::open_or_create(
        &RegistryKey::HKCU,
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
    );

    key.set_dword("AppsUseLightTheme", value);
    key.set_dword("SystemUsesLightTheme", value);
}
