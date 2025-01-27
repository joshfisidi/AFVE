use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use rustfft::{FftPlanner, num_complex::Complex};
use noise::{NoiseFn, Perlin};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::audio::Signal;
use crate::config::VisualizerConfig;

pub struct AudioEngine {
    sample_rate: u32,
    fft_size: usize,
    fft_data: Arc<Mutex<Vec<f32>>>,
    perlin: Perlin,
    start_time: Instant,
    config: VisualizerConfig,
}

impl AudioEngine {
    pub fn new(sample_rate: u32, fft_size: usize) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        AudioEngine {
            sample_rate,
            fft_size,
            fft_data: Arc::new(Mutex::new(vec![0.0; fft_size])),
            perlin: Perlin::new(seed),
            start_time: Instant::now(),
            config: VisualizerConfig::load(),
        }
    }

    pub fn process_audio(&self, data: &[f32]) {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.fft_size);

        let mut buffer: Vec<Complex<f32>> = data.iter()
            .take(self.fft_size)
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        // Apply Hann window
        for i in 0..buffer.len() {
            let multiplier = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 
                / buffer.len() as f32).cos());
            buffer[i] = buffer[i] * multiplier;
        }

        if buffer.len() < self.fft_size {
            buffer.resize(self.fft_size, Complex::new(0.0, 0.0));
        }

        fft.process(&mut buffer);

        let mut fft_data = self.fft_data.lock().unwrap();
        for (i, complex) in buffer.iter().take(self.fft_size / 2).enumerate() {
            let magnitude = complex.norm();
            fft_data[i] = magnitude;
        }
    }

    pub fn draw(&self, frame: &mut [u8], width: u32, height: u32) {
        let fft_data = self.fft_data.lock().unwrap();
        let elapsed = self.start_time.elapsed().as_secs_f64();

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % width as usize) as f64;
            let y = (i / width as usize) as f64;

            let noise_val = self.perlin.get([
                x * self.config.noise_scale + elapsed * self.config.noise_speed,
                y * self.config.noise_scale,
                elapsed * self.config.noise_speed * 2.0
            ]) as f32;

            let fft_index = (x / width as f64 * (self.fft_size / 2) as f64) as usize;
            let fft_value = if fft_index < fft_data.len() {
                fft_data[fft_index] / self.config.fft_scale
            } else {
                0.0
            };

            let combined = (noise_val as f32 + fft_value).max(0.0).min(1.0);
            let color_value = (combined * 255.0) as u8;

            pixel.copy_from_slice(&[
                (color_value as u32 * self.config.color_scheme.r as u32 / 255) as u8,
                (color_value as u32 * self.config.color_scheme.g as u32 / 255) as u8,
                (color_value as u32 * self.config.color_scheme.b as u32 / 255) as u8,
                255
            ]);
        }
    }
} 