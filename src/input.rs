use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::fs;

#[derive(Deserialize, Clone, Copy)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl From<InputBinding> for Binding {
    fn from(binding: InputBinding) -> Self {
        match binding {
            InputBinding::Key(key) => Binding::from(key),
            InputBinding::Mouse(button) => Binding::from(button),
        }
    }
}

pub fn load_config<T: DeserializeOwned + Default>(path: &str) -> T {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| ron::from_str(&s).ok())
        .unwrap_or_default()
}
