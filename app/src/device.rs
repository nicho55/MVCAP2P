use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Platform {
    Android,
    Linux,
    Windows,
    Other,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputMode {
    Touch,
    MouseKeyboard,
}

#[derive(Resource, Clone, Debug)]
pub struct DeviceProfile {
    pub platform: Platform,
    pub input_mode: InputMode,
}

impl DeviceProfile {
    pub fn detect() -> Self {
        let platform = if cfg!(target_os = "android") {
            Platform::Android
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Other
        };
        let input_mode = match platform {
            Platform::Android => InputMode::Touch,
            _ => InputMode::MouseKeyboard,
        };
        Self {
            platform,
            input_mode,
        }
    }

    pub fn is_mobile(&self) -> bool {
        self.platform == Platform::Android
    }
}

impl Default for DeviceProfile {
    fn default() -> Self {
        Self::detect()
    }
}
