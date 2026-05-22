use js_sys::Float32Array;
use mammut_fft_lib::audio_read_write::read_audio_bytes;
use mammut_fft_lib::utils::save_to_wav_bytes;
use mammut_fft_lib::{AudioProcessor, Result};
use serde::Serialize;
use wasm_bindgen::prelude::*;

// Set up panic hook for better error messages
fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(start)]
pub fn start() {
    init_panic_hook();
    mammut_fft_lib::init();
}

// Serde-compatible info structs for passing to JavaScript
#[derive(Serialize)]
struct AudioInfoJs {
    sample_rate: u32,
    channels: u16,
    fft_size: usize,
    time_data: Option<TimeDataInfoJs>,
    fft_data: Option<FFTDataInfoJs>,
}

#[derive(Serialize)]
struct TimeDataInfoJs {
    num_channels: usize,
    samples_per_channel: usize,
    duration_seconds: f64,
}

#[derive(Serialize)]
struct FFTDataInfoJs {
    num_channels: usize,
    complex_values_per_channel: usize,
    frequency_resolution: f64,
}

#[wasm_bindgen]
pub struct WasmAudioProcessor {
    processor: AudioProcessor,
    original_samples: Option<Vec<Vec<f64>>>,
}

#[wasm_bindgen]
impl WasmAudioProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        init_panic_hook();

        Self {
            processor: AudioProcessor::new(),
            original_samples: None,
        }
    }

    // Load audio data from a Float32Array (WASM-friendly)
    #[wasm_bindgen]
    pub fn load_audio_data(
        &mut self,
        channels: u16,
        sample_rate: u32,
        audio_data: &Float32Array,
    ) -> Result<()> {
        let mut samples = Vec::new();
        let length = audio_data.length() as usize;
        let channel_length = length / channels as usize;

        // Prepare to deinterleave the audio samples
        for _ in 0..channels {
            samples.push(vec![0.0; channel_length]);
        }

        // Copy audio data to Rust
        let mut buffer = vec![0.0; length];
        audio_data.copy_to(&mut buffer[..]);

        // Deinterleave
        for i in 0..channel_length {
            for ch in 0..channels as usize {
                samples[ch][i] = buffer[i * channels as usize + ch] as f64;
            }
        }

        // Store original samples
        self.original_samples = Some(samples.clone());

        // Set audio data in processor
        self.processor
            .set_audio_data(sample_rate, channels, samples)?;

        // Perform FFT
        self.processor.perform_fft()?;

        Ok(())
    }

    // Get info about the loaded audio
    #[wasm_bindgen]
    pub fn get_info(&self) -> JsValue {
        let info = self.processor.get_info();

        // Convert to our serde-compatible struct
        let js_info = AudioInfoJs {
            sample_rate: info.sample_rate,
            channels: info.channels,
            fft_size: info.fft_size,
            time_data: info.time_data.map(|td| TimeDataInfoJs {
                num_channels: td.num_channels,
                samples_per_channel: td.samples_per_channel,
                duration_seconds: td.duration_seconds,
            }),
            fft_data: info.fft_data.map(|fd| FFTDataInfoJs {
                num_channels: fd.num_channels,
                complex_values_per_channel: fd.complex_values_per_channel,
                frequency_resolution: fd.frequency_resolution,
            }),
        };

        serde_wasm_bindgen::to_value(&js_info).unwrap_or(JsValue::null())
    }

    #[wasm_bindgen]
    pub fn get_spectrum_data(
        &self,
        channel_index: usize,
        max_points: usize,
        use_log_scale: bool,
    ) -> Result<Float32Array> {
        if let Some(fft_data) = self.processor.fft_data() {
            if channel_index >= fft_data.len() {
                return Err(format!(
                    "Channel index {} out of bounds (max: {})",
                    channel_index,
                    fft_data.len() - 1
                ));
            }

            let channel_data = &fft_data[channel_index];
            let num_bins = channel_data.len();

            // First, extract the amplitude data from the complex values
            let mut amplitudes = Vec::with_capacity(num_bins);
            for value in channel_data.iter() {
                let (amplitude, _) = value.to_polar();
                amplitudes.push(amplitude as f32);
            }

            // Maximum frequency (Nyquist) in Hz
            let max_freq = self.processor.sample_rate() as f32 / 2.0;

            // If we need to downsample (for large files), or use log scale
            if num_bins > max_points || use_log_scale {
                let mut result = Vec::with_capacity(max_points);

                if use_log_scale {
                    // Logarithmic scaling for frequencies
                    // Calculate with log10 scale
                    // log10(20Hz) ? 1.3, log10(20kHz) ? 4.3
                    // We want to map this 1.3-4.3 range to 0-max_points
                    const MIN_FREQ: f32 = 20.0; // 20 Hz, lowest audible frequency
                    let log_min = MIN_FREQ.log10();
                    let log_max = max_freq.log10();
                    let log_range = log_max - log_min;

                    for i in 0..max_points {
                        // Calculate logarithmically spaced frequency point
                        let log_freq = log_min + (log_range * i as f32 / (max_points - 1) as f32);
                        let freq = 10.0f32.powf(log_freq);

                        // Convert frequency to bin index in the original FFT data
                        let bin_f = freq * (num_bins as f32) / max_freq;
                        let bin = bin_f as usize;

                        // Get amplitude using linear interpolation between bins
                        let amplitude = if bin >= num_bins - 1 {
                            amplitudes[num_bins - 1]
                        } else {
                            let frac = bin_f - bin as f32;
                            amplitudes[bin] * (1.0 - frac) + amplitudes[bin + 1] * frac
                        };

                        result.push(amplitude);
                    }
                } else {
                    // Linear frequency spacing, just downsample
                    for i in 0..max_points {
                        let bin = (i * (num_bins - 1)) / (max_points - 1);
                        result.push(amplitudes[bin]);
                    }
                }

                Ok(self.normalize_and_convert_to_float32_array(&result))
            } else {
                // No downsampling needed, just return the normalized data
                let mut result = Vec::with_capacity(num_bins);
                for (_, amplitude) in amplitudes.iter().enumerate() {
                    result.push(*amplitude);
                }
                Ok(self.normalize_and_convert_to_float32_array(&result))
            }
        } else {
            Err("No FFT data available".to_string())
        }
    }

    fn normalize_and_convert_to_float32_array(&self, amplitudes: &Vec<f32>) -> Float32Array {
        let result = Float32Array::new_with_length(amplitudes.len() as u32);
        let max_amplitude = Self::max_amplitude(amplitudes);
        if max_amplitude > 0.0 {
            for (pos, e) in amplitudes.iter().enumerate() {
                result.set_index(pos as u32, *e / max_amplitude);
            }
        } else {
            for (pos, e) in amplitudes.iter().enumerate() {
                result.set_index(pos as u32, *e);
            }
        }
        result
    }

    fn max_amplitude(amplitudes: &Vec<f32>) -> f32 {
        let max_amplitude = amplitudes
            .iter()
            .fold(f32::EPSILON, |max, &amp| max.max(amp));
        max_amplitude
    }

    // Apply power function to the amplitude
    #[wasm_bindgen]
    pub fn apply_pow(&mut self, exponent: f64) -> Result<()> {
        self.processor.apply_pow(exponent)
    }

    // Apply lowpass filter
    #[wasm_bindgen]
    pub fn apply_lowpass(&mut self, cutoff_hz: f64) -> Result<()> {
        self.processor.apply_lowpass(cutoff_hz)
    }

    // Apply highpass filter
    #[wasm_bindgen]
    pub fn apply_highpass(&mut self, cutoff_hz: f64) -> Result<()> {
        self.processor.apply_highpass(cutoff_hz)
    }

    // Apply bandpass filter
    #[wasm_bindgen]
    pub fn apply_bandpass(&mut self, low_hz: f64, high_hz: f64) -> Result<()> {
        self.processor.apply_bandpass(low_hz, high_hz)
    }

    // Apply phase shift
    #[wasm_bindgen]
    pub fn apply_phase_shift(&mut self, shift_radians: f64) -> Result<()> {
        self.processor.apply_phase_shift(shift_radians)
    }

    /// Apply spectrum shift
    #[wasm_bindgen]
    pub fn apply_spectrum_shift(&mut self, shift_hz: f64) -> Result<()> {
        self.processor.apply_spectrum_shift(shift_hz)
    }

    /// Apply frequency spectrum stretch
    #[wasm_bindgen]
    pub fn apply_stretch(&mut self, exponent: f64) -> Result<()> {
        self.processor.apply_stretch(exponent)
    }

    /// Apply wobble effect
    #[wasm_bindgen]
    pub fn apply_wobble(&mut self, frequency: f64, amplitude: f64) -> Result<()> {
        self.processor.apply_wobble(frequency, amplitude)
    }

    /// Apply threshold filter
    #[wasm_bindgen]
    pub fn apply_threshold(
        &mut self,
        threshold_level: f64,
        remove_above_threshold: bool,
    ) -> Result<()> {
        self.processor
            .apply_threshold(threshold_level, remove_above_threshold)
    }

    /// Apply amplitude derivative
    #[wasm_bindgen]
    pub fn apply_amplitude_derivative(&mut self, multiplier: f64) -> Result<()> {
        self.processor.apply_amplitude_derivative(multiplier)
    }

    /// Apply keep peaks filter
    #[wasm_bindgen]
    pub fn keep_peaks(&mut self) -> Result<()> {
        self.processor.keep_peaks()
    }

    // Explicitly perform inverse FFT
    #[wasm_bindgen]
    pub fn perform_ifft(&mut self) -> Result<()> {
        // The library now properly handles DC and Nyquist components internally
        self.processor.perform_ifft()
    }

    // Explicitly normalize time domain data
    #[wasm_bindgen]
    pub fn normalize_time_data(&mut self) -> Result<()> {
        self.processor.normalize_time_data()
    }

    // Mix channels
    #[wasm_bindgen]
    pub fn mix_channels(&mut self, weights: &js_sys::Float64Array) -> Result<()> {
        let length = weights.length() as usize;
        let mut rust_weights = vec![0.0; length];
        weights.copy_to(&mut rust_weights[..]);

        self.processor.mix_channels(&rust_weights)
    }

    // Get processed audio data, normalized since it's going to be converted to 16bit for the player
    #[wasm_bindgen]
    pub fn get_processed_audio_normalized(&mut self) -> Result<Float32Array> {
        // First perform IFFT if needed
        if !self.processor.has_time_data() {
            self.processor.perform_ifft()?;
        }

        if let Some(time_data) = self.processor.time_data() {
            let channels = time_data.len();
            let samples_per_channel = time_data[0].len();
            let total_samples = channels * samples_per_channel;

            // Find max
            let mut max = 0 as f64;
            for i in 0..samples_per_channel {
                for ch in 0..channels {
                    let sample = f64::abs(time_data[ch][i]);
                    if sample > max {
                        max = sample;
                    }
                }
            }
            if max == 0. {
                max = 1.;
            }

            // Create interleaved output
            let result = Float32Array::new_with_length(total_samples as u32);

            // Interleave the audio data
            for i in 0..samples_per_channel {
                for ch in 0..channels {
                    let index = (i * channels + ch) as u32;
                    result.set_index(index, (time_data[ch][i] / max) as f32);
                }
            }

            Ok(result)
        } else {
            Err("No time domain data available".to_string())
        }
    }

    #[wasm_bindgen]
    pub fn save_to_wav_bytes(&mut self) -> std::result::Result<js_sys::Uint8Array, JsValue> {
        // First perform IFFT if needed
        if !self.processor.has_time_data() {
            self.processor.perform_ifft()?;
        }

        // Convert the Rust Result to a JS-compatible result
        match save_to_wav_bytes(&mut self.processor) {
            Ok(bytes) => {
                // Convert Vec<u8> to a JS Uint8Array
                let array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
                array.copy_from(&bytes);
                Ok(array)
            }
            Err(e) => Err(JsValue::from_str(&format!("Error: {}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn read_audio_bytes(
        &mut self,
        data: js_sys::Uint8Array,
        buffer_multiplier: usize,
    ) -> Result<()> {
        // Convert input
        let data_vec = data.to_vec();

        // Process in a separate thread if expensive
        let result = read_audio_bytes(data_vec);

        match result {
            Ok((sample_rate, channels)) => {
                self.original_samples = Some(channels.clone());

                if buffer_multiplier > 1 {
                    self.processor.set_buffer_multiplier(buffer_multiplier);
                }

                // Set the audio data in the processor
                self.processor.set_audio_data(
                    sample_rate,
                    channels.len().try_into().unwrap(),
                    channels,
                )?;
                // Perform FFT on the loaded data
                self.processor.perform_fft()?;
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    // Reset to original audio data
    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<()> {
        if let Some(original_samples) = self.original_samples.clone() {
            let sample_rate = self.processor.sample_rate();
            let channels = self.processor.channels();

            // Reset the processor with original data
            self.processor
                .set_audio_data(sample_rate, channels, original_samples)?;
            self.processor.perform_fft()?;

            Ok(())
        } else {
            Err("No original audio data available".to_string())
        }
    }

    #[wasm_bindgen]
    pub fn apply_phase_multiply(&mut self, factor: f64) -> Result<()> {
        self.processor.apply_phase_multiply(factor)
    }

    #[wasm_bindgen]
    pub fn swap_bins(&mut self, block_size: f64, repeat: f64) -> Result<()> {
        self.processor.swap_bins(block_size, repeat)
    }

    #[wasm_bindgen]
    pub fn swap_channels(&mut self, repeat: f64) -> Result<()> {
        self.processor.swap_channels(repeat)
    }

    /// Reset the processor to just use the original polar FFT data,
    /// clearing any filtered Cartesian data created for splitting.
    #[wasm_bindgen]
    pub fn reset_split(&mut self) -> Result<()> {
        self.processor.reset_split()
    }

    /// Reset the processor to the original state completely,
    /// by redoing the FFT on the original time domain data.
    #[wasm_bindgen]
    pub fn reset_to_original(&mut self) -> Result<()> {
        self.processor.reset_to_original()
    }

    /// Prepare a specific part of a frequency spectrum split
    #[wasm_bindgen]
    pub fn prepare_split_part(
        &mut self,
        part_index: usize,
        num_parts: usize,
        group_size: usize,
        log: bool,
        octaves: u8,
        crossfade_factor: f64,
    ) -> Result<()> {
        self.processor
            .prepare_split_part(part_index, num_parts, group_size, log, octaves, crossfade_factor)
    }

    /// Apply a chord filter
    #[wasm_bindgen]
    pub fn apply_chord_filter(
        &mut self,
        freq1: f64,
        amp1: f64,
        freq2: f64,
        amp2: f64,
        freq3: f64,
        amp3: f64,
        freq4: f64,
        amp4: f64,
        freq5: f64,
        amp5: f64,
        width_cents: f64,
        harmonics_strength: f64,
    ) -> Result<()> {
        self.processor.apply_chord_filter(
            [freq1, freq2, freq3, freq4, freq5],
            [amp1, amp2, amp3, amp4, amp5],
            width_cents,
            harmonics_strength,
        )
    }

    #[wasm_bindgen]
    pub fn load_audio_data_with_padding(
        &mut self,
        channels: u16,
        sample_rate: u32,
        audio_data: &Float32Array,
        buffer_multiplier: usize,
    ) -> Result<()> {
        if buffer_multiplier < 1 {
            return Err("Buffer multiplier must be >= 1".to_string());
        }

        let mut samples = Vec::new();
        let length = audio_data.length() as usize;
        let channel_length = length / channels as usize;

        // Prepare to deinterleave the audio samples
        for _ in 0..channels {
            samples.push(vec![0.0; channel_length]);
        }

        // Copy audio data to Rust
        let mut buffer = vec![0.0; length];
        audio_data.copy_to(&mut buffer[..]);

        // Deinterleave
        for i in 0..channel_length {
            for ch in 0..channels as usize {
                samples[ch][i] = buffer[i * channels as usize + ch] as f64;
            }
        }

        // Store original samples
        self.original_samples = Some(samples.clone());

        // Set audio data in processor with padding
        self.processor.set_buffer_multiplier(buffer_multiplier);
        self.processor
            .set_audio_data(sample_rate, channels, samples)?;

        // Perform FFT
        self.processor.perform_fft()?;

        Ok(())
    }

    /// Convolve with another audio file
    #[wasm_bindgen]
    pub fn convolve_with_file(
        &mut self,
        audio_data: js_sys::Uint8Array,
        correlate: bool,
        wet_mix: f64,
    ) -> Result<()> {
        let data_vec = audio_data.to_vec();

        self.processor
            .convolve_with_bytes(data_vec, correlate, wet_mix)
    }
}
