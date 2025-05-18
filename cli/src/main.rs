//! FFT Audio Processor CLI
//!
//! Command-line interface for the FFT Audio Processor library.
//! Provides an interactive shell for audio processing operations.

use std::process;

use mammut_fft_lib::{utils, AudioProcessor};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Print the help message showing available commands.
fn print_help() {
    println!("Available commands:");
    println!("  open <filename>                - Open an audio file and perform FFT");
    println!("  save <filename>                - Save the audio file after inverse FFT");
    println!("  pow <exponent>                 - Raise the amplitude of each FFT bin to the specified power");
    println!(
        "  lowpass <cutoff_hz>            - Apply a lowpass filter at the specified cutoff frequency"
    );
    println!("  highpass <cutoff_hz>           - Apply a highpass filter at the specified cutoff frequency");
    println!("  bandpass <low_hz> <high_hz>    - Apply a bandpass filter between the specified frequencies");
    println!("  chord <freq1> <amp1> <freq2> <amp2> <freq3> <amp3> <freq4> <amp4> <freq5> <amp5> <width> <harmonics> - Apply a chord filter to isolate specific frequencies and harmonics");
    //println!("  phase <shift_radians>          - Apply a phase shift to all frequencies");
    println!("  phasemul <factor>              - Multiply all phases by a factor (creates interesting effects)");
    println!("  swapbins <block_size> <repeat> - Randomly swap frequency bins");
    println!(
        "  swapchannels <repeat>          - Randomly swap bins between channels (stereo effects)"
    );
    println!("  spectrumshift <shift_hz>       - Shift the frequency spectrum up or down by the specified amount in Hz");
    println!("  stretch <exponent>             - Apply non-linear frequency stretching with the specified exponent");
    println!(
        "  wobble <frequency> <amplitude> - Apply a wobbling effect to the frequency spectrum"
    );
    println!(
        "  threshold <level> [above]      - Apply a threshold filter to remove frequency components"
    );
    println!(
        "  derivateamp <multiplier>       - Apply amplitude derivative effect to the spectrum"
    );
    println!(
        "  keeppeaks                      - Keep only the local peaks in the frequency spectrum"
    );
    //println!("  mix <weight1> <weight2> ...     - Mix channels with specified weights");
    println!("  split <filename> <num_parts> [group_size] - Split frequency spectrum into multiple files");
    println!(
        "  info                           - Display information about the loaded audio and FFT data"
    );
    println!("  help                           - Show this help message");
    println!("  quit                           - Exit the program");
}

/// Process a user command.
fn process_command(command: &str, processor: &mut AudioProcessor) {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "open" => {
            if parts.len() != 2 {
                println!("Usage: open <filename>");
                return;
            }

            println!("Opening file: {}", parts[1]);
            match utils::load_from_wav(processor, parts[1]) {
                Ok(_) => {
                    let info = processor.get_info();
                    println!(
                        "Loaded file with {} channels at {} Hz",
                        info.channels, info.sample_rate
                    );

                    if let Some(time_info) = &info.time_data {
                        println!(
                            "Duration: {}",
                            utils::format_time(time_info.duration_seconds)
                        );
                    }
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        "save" => {
            if parts.len() != 2 {
                println!("Usage: save <filename>");
                return;
            }

            match processor.perform_ifft() {
                Ok(_) => println!("Performed inverse FFT for {}", parts[1]),
                Err(e) => {
                    println!("Error performing inverse FFT for {}: {}", parts[1], e);
                }
            }

            println!("Saving to file: {}", parts[1]);
            match utils::save_to_wav(processor, parts[1]) {
                Ok(_) => println!("File saved successfully: {}", parts[1]),
                Err(e) => println!("Error: {}", e),
            }
        }
        "pow" => {
            if parts.len() != 2 {
                println!("Usage: pow <exponent>");
                return;
            }

            // Parse the exponent parameter
            let exponent = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: exponent must be a valid floating-point number");
                    return;
                }
            };

            println!("Applying power function with exponent: {}", exponent);
            match processor.apply_pow(exponent) {
                Ok(_) => println!("Power function applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "lowpass" => {
            if parts.len() != 2 {
                println!("Usage: lowpass <cutoff_hz>");
                return;
            }

            // Parse the cutoff frequency
            let cutoff_hz = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: cutoff frequency must be a valid floating-point number");
                    return;
                }
            };

            println!(
                "Applying lowpass filter at cutoff: {}",
                utils::format_frequency(cutoff_hz)
            );
            match processor.apply_lowpass(cutoff_hz) {
                Ok(_) => println!("Lowpass filter applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "highpass" => {
            if parts.len() != 2 {
                println!("Usage: highpass <cutoff_hz>");
                return;
            }

            // Parse the cutoff frequency
            let cutoff_hz = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: cutoff frequency must be a valid floating-point number");
                    return;
                }
            };

            println!(
                "Applying highpass filter at cutoff: {}",
                utils::format_frequency(cutoff_hz)
            );
            match processor.apply_highpass(cutoff_hz) {
                Ok(_) => println!("Highpass filter applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "bandpass" => {
            if parts.len() != 3 {
                println!("Usage: bandpass <low_hz> <high_hz>");
                return;
            }

            // Parse the cutoff frequencies
            let low_hz = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: low cutoff frequency must be a valid floating-point number");
                    return;
                }
            };

            let high_hz = match parts[2].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: high cutoff frequency must be a valid floating-point number");
                    return;
                }
            };

            println!(
                "Applying bandpass filter between {} and {}",
                utils::format_frequency(low_hz),
                utils::format_frequency(high_hz)
            );

            match processor.apply_bandpass(low_hz, high_hz) {
                Ok(_) => println!("Bandpass filter applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "chord" => {
            if parts.len() != 13 {
                println!("Usage: chord <freq1> <amp1> <freq2> <amp2> <freq3> <amp3> <freq4> <amp4> <freq5> <amp5> <width> <harmonics>");
                println!("  freq1-5: Frequencies to keep (Hz, 0 to skip)");
                println!("  amp1-5: Relative amplitude for each frequency (0-1, 0 to skip)");
                println!("  width: Width around each frequency to keep (0-50 cents)");
                println!("  harmonics: Harmonic strength factor (0-1)");
                println!("    0: Only fundamental frequencies");
                println!("    1: Harmonics follow sawtooth wave amplitude decay (1/n)");
                return;
            }

            // Parse the frequency parameters
            let freq1 = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: frequency 1 must be a valid number");
                    return;
                }
            };

            let amp1 = match parts[2].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=1.0).contains(&value) {
                        println!("Error: amplitude 1 must be between 0 and 1");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: amplitude 1 must be a valid number");
                    return;
                }
            };

            let freq2 = match parts[3].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: frequency 2 must be a valid number");
                    return;
                }
            };

            let amp2 = match parts[4].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=1.0).contains(&value) {
                        println!("Error: amplitude 2 must be between 0 and 1");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: amplitude 2 must be a valid number");
                    return;
                }
            };

            let freq3 = match parts[5].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: frequency 3 must be a valid number");
                    return;
                }
            };

            let amp3 = match parts[6].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=1.0).contains(&value) {
                        println!("Error: amplitude 3 must be between 0 and 1");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: amplitude 3 must be a valid number");
                    return;
                }
            };

            let freq4 = match parts[7].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: frequency 4 must be a valid number");
                    return;
                }
            };

            let amp4 = match parts[8].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=1.0).contains(&value) {
                        println!("Error: amplitude 4 must be between 0 and 1");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: amplitude 4 must be a valid number");
                    return;
                }
            };

            let freq5 = match parts[9].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: frequency 5 must be a valid number");
                    return;
                }
            };

            let amp5 = match parts[10].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=1.0).contains(&value) {
                        println!("Error: amplitude 5 must be between 0 and 1");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: amplitude 5 must be a valid number");
                    return;
                }
            };

            let width = match parts[11].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=50.0).contains(&value) {
                        println!("Error: width must be between 0 and 50 cents");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: width must be a valid number");
                    return;
                }
            };

            let harmonics = match parts[12].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=1.0).contains(&value) {
                        println!("Error: harmonics must be between 0 and 1");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: harmonics must be a valid number");
                    return;
                }
            };

            println!(
                "Applying chord filter with frequencies: [{}, {}, {}, {}, {}]",
                freq1, freq2, freq3, freq4, freq5
            );
            println!(
                "Amplitudes: [{}, {}, {}, {}, {}]",
                amp1, amp2, amp3, amp4, amp5
            );
            println!("Width: {} cents, Harmonics strength: {}", width, harmonics);

            match processor.apply_chord_filter(
                [freq1, freq2, freq3, freq4, freq5],
                [amp1, amp2, amp3, amp4, amp5],
                width,
                harmonics,
            ) {
                Ok(_) => println!("Chord filter applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "phase" => {
            if parts.len() != 2 {
                println!("Usage: phase <shift_radians>");
                return;
            }

            // Parse the phase shift
            let phase_shift = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: phase shift must be a valid floating-point number");
                    return;
                }
            };

            println!("Applying phase shift of {} radians", phase_shift);
            match processor.apply_phase_shift(phase_shift) {
                Ok(_) => println!("Phase shift applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "phasemul" => {
            if parts.len() != 2 {
                println!("Usage: phasemul <factor>");
                return;
            }

            // Parse the factor parameter
            let factor = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: factor must be a valid floating-point number");
                    return;
                }
            };

            println!("Applying phase multiplication with factor: {}", factor);
            match processor.apply_phase_multiply(factor) {
                Ok(_) => println!("Phase multiplication applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "swapbins" => {
            if parts.len() != 3 {
                println!("Usage: swapbins <block_size> <repeat>");
                println!("  block_size: Max distance between swapped bins (percentage 0-100)");
                println!("  repeat: Number of swaps to perform per channel (percentage of number of bins)");
                return;
            }

            // Parse the block_size parameter (percentage)
            let block_size = match parts[1].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=100.0).contains(&value) {
                        println!("Error: block_size must be between 0 and 100");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: block_size must be a valid floating-point number");
                    return;
                }
            };

            // Parse the repeat parameter
            let repeat = match parts[2].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=100.0).contains(&value) {
                        println!("Error: repeat must be between 0 and 100");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: repeat must be a valid floating-point number");
                    return;
                }
            };

            println!(
                "Swapping frequency bins with block_size: {}%, repeat: {}%",
                block_size, repeat
            );
            match processor.swap_bins(block_size, repeat) {
                Ok(_) => println!("Frequency bins swapped successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "swapchannels" => {
            if parts.len() != 2 {
                println!("Usage: swapchannels <repeat>");
                println!("  repeat: Number of swaps to perform per channel (percentage of number of bins)");
                return;
            }

            // Parse the repeat parameter
            let repeat = match parts[1].parse::<f64>() {
                Ok(value) => {
                    if !(0.0..=100.0).contains(&value) {
                        println!("Error: repeat must be between 0 and 100");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: repeat must be a valid floating-point number");
                    return;
                }
            };

            println!("Swapping bins between channels, repeat: {}%", repeat);
            match processor.swap_channels(repeat) {
                Ok(_) => println!("Channel bins swapped successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "spectrumshift" => {
            if parts.len() != 2 {
                println!("Usage: spectrumshift <shift_hz>");
                return;
            }

            // Parse the shift in Hz
            let shift_hz = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: shift amount must be a valid floating-point number");
                    return;
                }
            };

            println!("Applying spectrum shift of {} Hz...", shift_hz);
            match processor.apply_spectrum_shift(shift_hz) {
                Ok(_) => println!("Spectrum shifted successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "stretch" => {
            if parts.len() != 2 {
                println!("Usage: stretch <exponent>");
                println!("  exponent: Value > 1 compresses high frequencies, < 1 expands them (default: 1.3)");
                return;
            }

            // Parse the exponent
            let exponent = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: exponent must be a valid floating-point number");
                    return;
                }
            };

            println!(
                "Applying frequency spectrum stretch with exponent: {}",
                exponent
            );
            match processor.apply_stretch(exponent) {
                Ok(_) => println!("Frequency spectrum stretched successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "wobble" => {
            if parts.len() != 3 {
                println!("Usage: wobble <frequency> <amplitude>");
                println!("  frequency: Controls wobble cycles (default: 10.0)");
                println!("  amplitude: Controls displacement amount (0.0-0.1, default: 0.01)");
                return;
            }

            // Parse the parameters
            let frequency = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: frequency must be a valid floating-point number");
                    return;
                }
            };

            let amplitude = match parts[2].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: amplitude must be a valid floating-point number");
                    return;
                }
            };

            println!(
                "Applying wobble effect with frequency: {}, amplitude: {}",
                frequency, amplitude
            );
            match processor.apply_wobble(frequency, amplitude) {
                Ok(_) => println!("Wobble effect applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "threshold" => {
            if parts.len() != 2 && parts.len() != 3 {
                println!("Usage: threshold <level> [above]");
                println!("  level: Threshold level (default: 1.0)");
                println!("  above: If 'above' is specified, removes components above threshold;");
                println!("         otherwise, removes components below threshold (default)");
                return;
            }

            // Parse the threshold level
            let level = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: threshold level must be a valid floating-point number");
                    return;
                }
            };

            // Determine the threshold direction
            let remove_above = parts.len() == 3 && parts[2].to_lowercase() == "above";

            println!(
                "Applying threshold filter with level: {}, removing components {} threshold",
                level,
                if remove_above { "above" } else { "below" }
            );

            match processor.apply_threshold(level, remove_above) {
                Ok(_) => println!("Threshold filter applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "derivateamp" => {
            if parts.len() != 2 {
                println!("Usage: derivateamp <multiplier>");
                println!("  multiplier: Scaling factor for the derivative values (default: 1.0)");
                return;
            }

            // Parse the multiplier parameter
            let multiplier = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    println!("Error: multiplier must be a valid floating-point number");
                    return;
                }
            };

            println!(
                "Applying amplitude derivative with multiplier: {}",
                multiplier
            );
            match processor.apply_amplitude_derivative(multiplier) {
                Ok(_) => println!("Amplitude derivative applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "keeppeaks" => {
            if parts.len() != 1 {
                println!("Usage: keeppeaks");
                println!("  Keeps only the local maxima in the frequency spectrum");
                return;
            }

            println!("Applying keep peaks filter...");
            match processor.keep_peaks() {
                Ok(_) => println!("Keep peaks filter applied successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "split" => {
            if parts.len() < 3 || parts.len() > 4 {
                println!("Usage: split <filename> <num_parts> [group_size]");
                println!("  filename:  Base filename for output files (e.g., 'output.wav' will produce 'output_0.wav', etc.)");
                println!("  num_parts: Number of files to split into");
                println!("  group_size: Number of consecutive bins to group together (default: 1)");
                return;
            }

            let filename = parts[1];

            // Parse the num_parts parameter
            let num_parts = match parts[2].parse::<usize>() {
                Ok(value) => {
                    if value == 0 {
                        println!("Error: num_parts must be greater than zero");
                        return;
                    }
                    value
                }
                Err(_) => {
                    println!("Error: num_parts must be a valid positive integer");
                    return;
                }
            };

            // Parse the optional group_size parameter (default to 1 if not provided)
            let group_size = if parts.len() == 4 {
                match parts[3].parse::<usize>() {
                    Ok(value) => {
                        if value == 0 {
                            println!("Error: group_size must be greater than zero");
                            return;
                        }
                        value
                    }
                    Err(_) => {
                        println!("Error: group_size must be a valid positive integer");
                        return;
                    }
                }
            } else {
                1 // Default group size
            };

            println!(
                "Splitting frequency spectrum into {} parts with group size {}...",
                num_parts, group_size
            );

            // Generate output filenames based on the input filename
            let mut output_filenames = Vec::new();

            // Handle filenames with or without extension
            let filename_parts: Vec<&str> = filename.rsplitn(2, '.').collect();
            let (base_name, extension) = if filename_parts.len() == 2 {
                (filename_parts[1], format!(".{}", filename_parts[0]))
            } else {
                (filename, String::from(""))
            };

            for i in 0..num_parts {
                output_filenames.push(format!("{}_{}{}", base_name, i, extension));
            }

            // Ensure we have FFT data
            if !processor.has_frequency_data() {
                println!("Error: No frequency data available. Load a file first.");
                return;
            }

            // Process each part and save to a file
            for (i, output_file) in output_filenames.iter().enumerate() {
                println!("Processing part {}/{}...", i + 1, num_parts);

                // Prepare this part of the split
                match processor.prepare_split_part(i, num_parts, group_size) {
                    Ok(_) => {
                        println!("Prepared frequency bins for part {}", i);
                    }
                    Err(e) => {
                        println!("Error preparing part {}: {}", i, e);
                        continue;
                    }
                }

                // Explicitly perform the inverse FFT to convert filtered frequency data to time domain
                match processor.perform_ifft() {
                    Ok(_) => println!("Performed inverse FFT for part {}", i),
                    Err(e) => {
                        println!("Error performing inverse FFT for part {}: {}", i, e);
                        continue;
                    }
                }

                // Save this part to a file
                println!("Saving part {} to {}...", i, output_file);
                match utils::save_to_wav(processor, output_file) {
                    Ok(_) => println!("Part {} saved successfully to {}", i, output_file),
                    Err(e) => println!("Error saving part {}: {}", i, e),
                }

                // Reset the split to prepare for the next part
                // Use the lighter-weight reset_split instead of full reset_to_original
                match processor.reset_split() {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Error resetting split: {}", e);
                        return;
                    }
                }
            }

            println!("Frequency spectrum split completed");
        }
        "mix" => {
            if parts.len() < 2 {
                println!("Usage: mix <weight1> <weight2> ...");
                return;
            }

            // Parse the weights
            let mut weights = Vec::new();
            for i in 1..parts.len() {
                match parts[i].parse::<f64>() {
                    Ok(value) => weights.push(value),
                    Err(_) => {
                        println!("Error: weight {} must be a valid floating-point number", i);
                        return;
                    }
                }
            }

            println!("Mixing channels with weights: {:?}", weights);
            match processor.mix_channels(&weights) {
                Ok(_) => println!("Channels mixed successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        "info" => {
            if parts.len() != 1 {
                println!("Usage: info");
                return;
            }

            // Display information about the loaded audio and FFT data
            let info = processor.get_info();

            println!("===== Audio Information =====");
            println!("Sample rate: {} Hz", info.sample_rate);
            println!("Channels: {}", info.channels);

            if let Some(time_info) = &info.time_data {
                println!("\n----- Time Domain Data -----");
                println!("Channels: {}", time_info.num_channels);
                println!("Samples per channel: {}", time_info.samples_per_channel);
                println!(
                    "Duration: {}",
                    utils::format_time(time_info.duration_seconds)
                );
            } else {
                println!("\nNo time domain data available");
            }

            if let Some(fft_info) = &info.fft_data {
                println!("\n----- Frequency Domain Data -----");
                println!("FFT size: {}", info.fft_size);
                println!("Channels: {}", fft_info.num_channels);
                println!(
                    "Complex values per channel: {}",
                    fft_info.complex_values_per_channel
                );
                println!(
                    "Frequency resolution: {}",
                    utils::format_frequency(fft_info.frequency_resolution)
                );
                println!(
                    "Nyquist frequency: {}",
                    utils::format_frequency(info.sample_rate as f64 / 2.0)
                );
            }
        }
        "help" => print_help(),
        "quit" => {
            println!("Exiting...");
            process::exit(0);
        }
        _ => {
            println!("Unknown command: {}", parts[0]);
            println!("Type 'help' for available commands");
        }
    }
}

fn main() {
    println!(
        "mammut-fft FFT Audio Processor v{}",
        mammut_fft_lib::VERSION
    );
    println!("Type 'help' for available commands");

    // Initialize the library
    mammut_fft_lib::init();

    let mut audio_processor = AudioProcessor::new();
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                process_command(&line, &mut audio_processor);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
