# MAMMUT-FFT

A Rust library and application for audio processing using Fast Fourier Transform (FFT). Inspired by the application mammut by &Oslash;vind Hammer/NoTAM, it does the same thing: loads a wave file and analyzes the whole file with one FFT instead of Short-time Fourier transform which is normally used. Not all operations in the original program are implemented as of now...

## Features

- Load and save WAV audio files
- Perform one real FFT analysis on the entire audio data
- Manipulate the audio in the frequency domain
- Interactive command-line interface
- Web frontend with Rust code compiled to WASM
- WAV files are saved in 32-bit float format 

## Project Structure

This project is organized as a Rust workspace with multiple crates:

- `lib`: Core FFT audio processing library
- `cli`: Command-line interface application
- `web`: WebAssembly frontend (planned)

## CLI Commands

The command-line interface supports the following commands:

```
  open <filename>              - Open an audio file and perform FFT
  save <filename>              - Save the audio file after inverse FFT
  pow <exponent>               - Raise the amplitude of each FFT bin to the specified power
  lowpass <cutoff_hz>          - Apply a lowpass filter at the specified cutoff frequency
  highpass <cutoff_hz>         - Apply a highpass filter at the specified cutoff frequency
  bandpass <low_hz> <high_hz>  - Apply a bandpass filter between the specified frequencies
  phase <shift_radians>        - Apply a phase shift to all frequencies
  phasemul <factor>            - Multiply all phases by a factor (creates interesting effects)
  swapbins <block_size> <repeat> - Randomly swap frequency bins
  swapchannels <repeat>        - Randomly swap bins between channels (stereo effects)
  mix <weight1> <weight2> ...  - Mix channels with specified weights
  split <filename> <num_parts> [group_size] - Split frequency spectrum into multiple files
  info                         - Display information about the loaded audio and FFT data
  help                         - Show this help message
  quit                         - Exit the program
  ```

## Building and Running

### Prerequisites

- Rust compiler (1.86.0 or newer)
- Cargo package manager

### Building

To build the entire project:

```bash
cargo build --release
```

To build just the CLI application:

```bash
cargo build --release -p mammut_fft_cli
```

### Running the CLI

```bash
cargo run --release -p mammut_fft_cli
```

Or after building:

```bash
./target/release/mammut_fft_cli
```

## WebAssembly Support

To build the web version run:

```bash
./web/build.sh
```

Then to run it:

```bash
./web/run.sh
```

## Dependencies

- realfft: Efficient real-to-complex FFT implementation
- hound: WAV file reader/writer
- num-complex: Complex number support
- rustyline: Interactive command-line input
- clap: Command-line argument parsing

## TODO

This app was coded as an experiment using AI for programming (Claude 3.7), with some manual corrections where I didn't manage to get it to do what I wanted. It may need some code clean up, but it looks quite good and it works...

Implement more operations, either from the original application or other ideas that may pop up.

The spectrum view does not show anything for large files

## Live Version

A working version can be found at https://anatemno.org/projects/mammut-fft/

## Usage (web version)

Open audio file with "Open Audio" button or drag-and-drop a file.
Try to use small values for the operations first, as they may turn the file into all noise if too much.
To hear the result in the player, you must first press the "Process Audio" button.
"Download Processed Audio" will let you download and save the file locally. It is saved as a 32-bit float wave file.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
