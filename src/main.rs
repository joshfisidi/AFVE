use afve::engine::AudioEngine;
use symphonia::core::audio::SampleBuffer;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;
use symphonia::default::get_codecs;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const SAMPLE_RATE: u32 = 44100;
const FFT_SIZE: usize = 2048;

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
    let engine_clone = Arc::clone(&engine);

    // Audio setup
    let host = cpal::default_host();
    let device = host.default_output_device()
        .expect("no output device available");
    
    let config = device.default_output_config()
        .expect("no default output config available");

    // Audio playback thread
    std::thread::spawn(move || {
        let path = Path::new("audio.mp3");
        let file = File::open(path).expect("Failed to open audio file");
        let media_source = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("mp3");

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let probed = get_probe()
            .format(&hint, media_source, &format_opts, &metadata_opts)
            .expect("Unsupported format");

        let mut format = probed.format;
        let track = format.default_track().expect("No default track");
        
        let decoder_opts = Default::default();
        let mut decoder = get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .expect("Unsupported codec");

        let stream = device.build_output_stream(
            &config.config(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Get next packet and decode
                if let Ok(packet) = format.next_packet() {
                    if let Ok(decoded) = decoder.decode(&packet) {
                        let spec = *decoded.spec();
                        let mut sample_buf = SampleBuffer::<f32>::new(
                            decoded.capacity() as u64,
                            spec
                        );
                        sample_buf.copy_interleaved_ref(decoded);
                        
                        // Copy decoded audio to output buffer
                        let samples = sample_buf.samples();
                        let len = data.len().min(samples.len());
                        data[..len].copy_from_slice(&samples[..len]);
                        
                        // Process audio for visualization
                        let engine = engine_clone.lock().unwrap();
                        engine.process_audio(samples);
                    }
                }
            },
            |err| eprintln!("Audio playback error: {}", err),
            None
        ).expect("Failed to build output stream");

        stream.play().expect("Failed to play audio stream");
    });

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

            // Add space key handler for play/pause
            if input.key_pressed(winit::event::VirtualKeyCode::Space) {
                let mut engine = engine.lock().unwrap();
                engine.toggle_playback();
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