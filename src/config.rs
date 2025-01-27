use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

#[derive(Serialize, Deserialize, Clone)]
pub struct VisualizerConfig {
    pub noise_scale: f64,
    pub noise_speed: f64,
    pub fft_scale: f32,
    pub color_scheme: ColorScheme,
    pub playback: PlaybackConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaybackConfig {
    pub volume: f32,
    pub auto_play: bool,
}

#[derive(Serialize, Deserialize, Clone)]
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
            playback: PlaybackConfig {
                volume: 1.0,
                auto_play: false,
            },
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

    pub fn save(&self) -> io::Result<()> {
        let config_str = toml::to_string(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write("config.toml", config_str)
    }
} 