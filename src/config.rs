use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct VisualizerConfig {
    pub noise_scale: f64,
    pub noise_speed: f64,
    pub fft_scale: f32,
    pub color_scheme: ColorScheme,
}

#[derive(Deserialize)]
pub struct ColorScheme {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for VisualizerConfig {
    fn default() -> Self {
        Self {
            noise_scale: 0.01,
            noise_speed: 0.1,
            fft_scale: 100.0,
            color_scheme: ColorScheme { r: 255, g: 255, b: 255 },
        }
    }
}

impl VisualizerConfig {
    pub fn load() -> Self {
        match fs::read_to_string("config.toml") {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
} 