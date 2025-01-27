use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use rustfft::{FftPlanner, num_complex::Complex};
use noise::{NoiseFn, Perlin};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const SAMPLE_RATE: u32 = 44100;
const FFT_SIZE: usize = 2048;

struct AudioEngine {
    sample_rate: u32,
    fft_size: usize,
    fft_data: Arc<Mutex<Vec<f32>>>,
    perlin: Perlin,
    start_time: Instant,
}

impl AudioEngine {
    fn new(sample_rate: u32, fft_size: usize) -> Self {
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
            fft_data[i] = (1.0 + magnitude).log10();
        }
    }

    pub fn draw(&self, frame: &mut [u8], width: u32, height: u32) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        
        for y in 0..height {
            for x in 0..width {
                let noise_value = self.get_visualization_data(
                    x as f64 / width as f64,
                    y as f64 / height as f64,
                    elapsed
                );
                
                let color_value = ((noise_value + 1.0) * 127.5) as u8;
                let pixel_index = (y * width + x) as usize * 4;
                
                frame[pixel_index] = color_value;     // R
                frame[pixel_index + 1] = 0;           // G
                frame[pixel_index + 2] = 255 - color_value; // B
                frame[pixel_index + 3] = 255;         // A
            }
        }
    }

    fn get_visualization_data(&self, x: f64, y: f64, time: f64) -> f64 {
        let fft_data = self.fft_data.lock().unwrap();
        let fft_index = (x * (fft_data.len() as f64)) as usize;
        let magnitude = if fft_index < fft_data.len() {
            fft_data[fft_index] as f64
        } else {
            0.0
        };

        let noise = self.perlin.get([
            x * 2.0,
            y * 2.0,
            time * 0.5 + magnitude * 2.0
        ]);
        
        (noise + magnitude * 2.0) / 3.0
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Audio Frequency Visualizer Engine")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;

    let engine = Arc::new(Mutex::new(AudioEngine::new(SAMPLE_RATE, FFT_SIZE)));

    let host = cpal::default_host();
    let device = host.default_input_device()
        .expect("no input device available");

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Default,
    };

    let engine_clone = Arc::clone(&engine);

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let engine = engine_clone.lock().unwrap();
            engine.process_audio(data);
        },
        move |err| {
            eprintln!("Error in audio stream: {:?}", err);
        },
        None
    ).unwrap();

    stream.play().unwrap();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            let engine = engine.lock().unwrap();
            engine.draw(pixels.frame_mut(), WIDTH, HEIGHT);
            if let Err(err) = pixels.render() {
                log::error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.close_requested() || input.destroyed() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log::error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            window.request_redraw();
        }
    });
}