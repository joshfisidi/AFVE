# Audio Frequency Visualizer Engine (AFVE)

A real-time audio visualizer built in Rust that combines FFT-based frequency analysis with Perlin noise to create dynamic visualizations of audio playback.

## Features

- Real-time FFT audio analysis
- Dynamic visualization with Perlin noise
- Configurable visual parameters
- Volume control
- Play/pause functionality
- Support for MP3 and WAV audio formats

## Requirements

- Rust 1.70 or higher
- An audio file named `audio.mp3` in the project root directory
- A system with audio output capabilities

## Installation

1. Clone the repository:
```bash
git clone [your-repository-url]
cd afve
```

2. Build the project:
```bash
cargo build --release
```

## Usage

1. Place your audio file in the project root directory as `audio.mp3`
2. Run the visualizer:
```bash
cargo run --release
```

### Controls

- **Space**: Toggle play/pause
- **Close Window**: Exit the application

### Configuration

The visualizer can be customized through the `config.toml` file:

```toml
noise_scale = 0.01    # Controls the scale of the Perlin noise
noise_speed = 0.1     # Controls the speed of noise animation
fft_scale = 100.0     # Adjusts the intensity of the frequency visualization

[color_scheme]
r = 255              # Red component (0-255)
g = 100              # Green component (0-255)
b = 100              # Blue component (0-255)

[playback]
volume = 1.0         # Audio volume (0.0-1.0)
auto_play = false    # Start playback automatically
```

The configuration file is automatically loaded at startup and saved when settings change.

## Technical Details

- Uses `rustfft` for Fast Fourier Transform analysis
- `symphonia` for audio decoding
- `cpal` for audio playback
- `pixels` for efficient rendering
- `noise` for Perlin noise generation
- Window management with `winit`

## Building from Source

1. Ensure you have Rust and Cargo installed
2. Clone the repository
3. Install dependencies:
```bash
cargo build
```

## License

[Your chosen license]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 