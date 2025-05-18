use std::sync::Arc;

use num_complex::Complex64;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

use crate::Result;

/// The core FFT audio processing struct.
///
/// This struct handles all FFT-related operations on audio data,
/// including transformations between time and frequency domains,
/// as well as spectral manipulations.
pub struct AudioProcessor {
    /// Audio sample rate in Hz
    sample_rate: u32,
    /// Number of audio channels
    channels: u16,
    /// FFT data in cartesian form (real/imaginary) for each channel
    fft_data: Option<Vec<Vec<Complex64>>>,
    /// Temporary copy of data for split operation
    tmp_fft_data: Option<Vec<Vec<Complex64>>>,
    /// Time domain audio data for each channel
    time_data: Option<Vec<Vec<f64>>>,
    /// FFT forward transform planner
    fft_r2c: Option<Arc<dyn RealToComplex<f64>>>,
    /// FFT inverse transform planner
    fft_c2r: Option<Arc<dyn ComplexToReal<f64>>>,
    /// FFT size (power of 2)
    fft_size: usize,
    /// max amplitude of all channels in fft_polar_data, used for scaling to max level=1.
    /// according to realfft documentation the scaling for forward+inverse fft should be 1/length
    /// so we should then scale with max_amplitude after doing inverse_fft
    max_amplitude: f64,
}

// Manual implementation of Debug since the FFT planners don't implement Debug
impl std::fmt::Debug for AudioProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioProcessor")
            .field("sample_rate", &self.sample_rate)
            .field("channels", &self.channels)
            .field("fft_data", &self.fft_data)
            .field("time_data", &self.time_data)
            .field("fft_r2c", &format_args!("<FFT Planner>"))
            .field("fft_c2r", &format_args!("<IFFT Planner>"))
            .field("fft_size", &self.fft_size)
            .finish()
    }
}

impl AudioProcessor {
    /// Create a new `AudioProcessor` instance.
    pub fn new() -> Self {
        AudioProcessor {
            sample_rate: 0,
            channels: 0,
            fft_data: None,
            time_data: None,
            tmp_fft_data: None,
            fft_r2c: None,
            fft_c2r: None,
            fft_size: 0,
            max_amplitude: 1.,
        }
    }

    /// Get the current sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the number of channels.
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Get the FFT size.
    pub fn fft_size(&self) -> usize {
        self.fft_size
    }

    /// Check if time domain data is available.
    pub fn has_time_data(&self) -> bool {
        self.time_data.is_some()
    }

    /// Check if frequency domain data is available.
    pub fn has_frequency_data(&self) -> bool {
        self.fft_data.is_some()
    }

    /// Set audio parameters and time domain data.
    ///
    /// This method is used to load audio data from any source,
    /// allowing the processor to work with data from files,
    /// memory, network, or other sources.
    pub fn set_audio_data(
        &mut self,
        sample_rate: u32,
        channels: u16,
        time_data: Vec<Vec<f64>>,
    ) -> Result<()> {
        if time_data.is_empty() {
            return Err("No audio data provided".to_string());
        }

        if time_data[0].is_empty() {
            return Err("Audio data contains empty channels".to_string());
        }

        // Ensure all channels have the same length
        let first_channel_len = time_data[0].len();
        for (i, channel) in time_data.iter().enumerate().skip(1) {
            if channel.len() != first_channel_len {
                return Err(format!(
                    "Inconsistent channel lengths: channel 0 has {} samples, channel {} has {} samples",
                    first_channel_len, i, channel.len()
                ));
            }
        }

        self.sample_rate = sample_rate;
        self.channels = channels;
        self.time_data = Some(time_data);

        // Clear existing FFT data
        self.fft_data = None;

        Ok(())
    }

    /// Get a reference to the time domain data.
    pub fn time_data(&self) -> Option<&Vec<Vec<f64>>> {
        self.time_data.as_ref()
    }

    /// Get a reference to the FFT data
    pub fn fft_data(&self) -> Option<&Vec<Vec<Complex64>>> {
        self.fft_data.as_ref()
    }

    /// Apply a power function to the amplitude of the FFT data.
    ///
    /// This raises each amplitude value to the specified power while
    /// preserving the phase information.
    pub fn apply_pow(&mut self, exponent: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            // Process each channel independently
            for channel_data in data_channels.iter_mut() {
                for i in 0..channel_data.len() {
                    // Get the current amplitude (real part)

                    let (amplitude, phase) = channel_data[i].to_polar();

                    // Raise amplitude to the specified power
                    // Amplitude in polar form is always non-negative
                    let powered_amplitude = amplitude.powf(exponent);

                    // Update amplitude while preserving phase
                    channel_data[i] = Complex64::from_polar(powered_amplitude, phase);
                }
            }
            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Perform the FFT on all audio channels.
    ///
    /// This transforms the time domain data into frequency domain data.
    pub fn perform_fft(&mut self) -> Result<()> {
        if let Some(time_data_channels) = &self.time_data {
            let mut planner = RealFftPlanner::<f64>::new();

            // Find next power of 2 for FFT size based on the first channel's length
            let mut fft_size = 1;
            while fft_size < time_data_channels[0].len() {
                fft_size *= 2;
            }
            self.fft_size = fft_size;

            // Create FFT objects for forward and inverse transforms
            let r2c = planner.plan_fft_forward(fft_size);
            let c2r = planner.plan_fft_inverse(fft_size);

            // Save the FFT objects for later use
            self.fft_r2c = Some(r2c);
            self.fft_c2r = Some(c2r);

            let num_channels = time_data_channels.len();
            let mut fft_data_channels = Vec::with_capacity(num_channels);

            self.max_amplitude = 0.0;

            // Process each channel independently
            for channel_idx in 0..num_channels {
                let channel_data = &time_data_channels[channel_idx];

                // Prepare input data for FFT (padding with zeros if necessary)
                let mut real_input = channel_data.clone();

                // Pad with zeros if necessary
                real_input.resize(fft_size, 0.0);

                // Create a buffer for the FFT output (real FFT output is only fft_size/2 + 1 complex values)
                let mut complex_output = vec![Complex64::new(0.0, 0.0); fft_size / 2 + 1];

                // Perform the FFT for this channel
                if let Some(r2c) = &self.fft_r2c {
                    r2c.process(&mut real_input, &mut complex_output)
                        .map_err(|e| format!("FFT error on channel {}: {}", channel_idx, e))?;
                } else {
                    return Err("FFT planner not initialized".to_string());
                }

                //Calculate max
                for c in &complex_output {
                    let (mut amplitude, _) = c.to_polar();
                    amplitude = amplitude.abs();
                    if amplitude > self.max_amplitude {
                        self.max_amplitude = amplitude;
                    }
                }
                // Store the FFT data for this channel
                fft_data_channels.push(complex_output);
            }

            // Normalize amplitudes based on max for all channels
            if self.max_amplitude > 0.0 {
                let scale = 1. / self.max_amplitude;
                for channel_idx in 0..num_channels {
                    let data = &mut fft_data_channels[channel_idx];
                    for i in 0..data.len() {
                        data[i] = data[i].scale(scale);
                    }
                }
            }

            // Store all channels' FFT data
            self.fft_data = Some(fft_data_channels);

            Ok(())
        } else {
            Err("No time data available for FFT".to_string())
        }
    }

    /// Prepare the FFT data for inverse transformation.
    ///
    /// This ensures the FFT data is properly formatted for a real FFT,
    /// particularly handling the DC and Nyquist components correctly.
    pub fn prepare_for_ifft(&mut self) -> Result<()> {
        if let Some(fft_data_channels) = &mut self.fft_data {
            // Process each channel independently
            for channel_data in fft_data_channels.iter_mut() {
                if !channel_data.is_empty() {
                    // Handle DC component (should be real)
                    channel_data[0] = Complex64::new(channel_data[0].re, 0.0);

                    // Handle Nyquist component (should be real)
                    if channel_data.len() > 1 {
                        let nyquist_idx = channel_data.len() - 1;
                        channel_data[nyquist_idx] =
                            Complex64::new(channel_data[nyquist_idx].re, 0.0);
                    }
                }
            }
        }
        Ok(())
    }

    /// Normalize time domain data to prevent clipping.
    ///
    /// This finds the peak amplitude across all channels and normalizes
    /// all samples accordingly, ensuring the output won't clip while
    /// maintaining the balance between channels.
    pub fn normalize_time_data(&mut self) -> Result<()> {
        if let Some(time_data_channels) = &mut self.time_data {
            // Only normalize if we have a non-zero maximum
            if self.max_amplitude > 0.0 {
                // Normalize all channels using the same scaling factor
                let scale_factor = self.max_amplitude / (self.fft_size as f64);

                for channel in time_data_channels.iter_mut() {
                    for sample in channel.iter_mut() {
                        *sample *= scale_factor;
                    }
                }

                println!("Normalized time data with scale factor: {}", scale_factor);
            }

            Ok(())
        } else {
            Err("No time domain data available for normalization".to_string())
        }
    }

    /// Perform the inverse FFT to convert frequency domain data back to time domain.
    pub fn perform_ifft(&mut self) -> Result<()> {
        // Prepare the data for a real IFFT
        self.prepare_for_ifft()?;

        if let Some(fft_data_channels) = &self.fft_data {
            let num_channels = fft_data_channels.len();
            let mut time_data_channels = Vec::with_capacity(num_channels);

            // Process each channel independently
            for channel_idx in 0..num_channels {
                let channel_fft_data = &fft_data_channels[channel_idx];

                // Clone the FFT data for inverse FFT
                let mut complex_input = channel_fft_data.clone();

                // Create a buffer for the time domain output
                let mut real_output = vec![0.0; self.fft_size];

                // Perform the inverse FFT for this channel
                if let Some(c2r) = &self.fft_c2r {
                    c2r.process(&mut complex_input, &mut real_output)
                        .map_err(|e| format!("IFFT error on channel {}: {}", channel_idx, e))?;
                } else {
                    return Err("IFFT planner not initialized".to_string());
                }

                // Store this channel's time domain data
                time_data_channels.push(real_output);
            }

            // Store all channels' time domain data
            self.time_data = Some(time_data_channels);

            // Normalize the time domain data to prevent clipping
            self.normalize_time_data()?;

            Ok(())
        } else {
            Err("No FFT data available for inverse FFT".to_string())
        }
    }

    /// Get information about the loaded audio data.
    ///
    /// Returns a structured collection of information about the current state.
    pub fn get_info(&self) -> AudioInfo {
        let time_data_info = self.time_data.as_ref().map(|data| TimeDataInfo {
            num_channels: data.len(),
            samples_per_channel: data[0].len(),
            duration_seconds: data[0].len() as f64 / self.sample_rate as f64,
        });

        let fft_data_info = self.fft_data.as_ref().map(|data| FFTDataInfo {
            num_channels: data.len(),
            complex_values_per_channel: data[0].len(),
            frequency_resolution: self.sample_rate as f64 / self.fft_size as f64,
        });

        AudioInfo {
            sample_rate: self.sample_rate,
            channels: self.channels,
            fft_size: self.fft_size,
            time_data: time_data_info,
            fft_data: fft_data_info,
        }
    }

    /// Apply a lowpass filter at the specified cutoff frequency.
    ///
    /// This attenuates frequencies above the cutoff frequency.
    pub fn apply_lowpass(&mut self, cutoff_hz: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Calculate the bin index corresponding to the cutoff frequency
            let cutoff_bin =
                (cutoff_hz * self.fft_size as f64 / self.sample_rate as f64).round() as usize;

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];

                // Apply filter: zero out amplitudes above cutoff
                for bin in cutoff_bin.min(channel_data.len())..channel_data.len() {
                    channel_data[bin] = Complex64::new(0.0, 0.0);
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply a highpass filter at the specified cutoff frequency.
    ///
    /// This attenuates frequencies below the cutoff frequency.
    pub fn apply_highpass(&mut self, cutoff_hz: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Calculate the bin index corresponding to the cutoff frequency
            let cutoff_bin =
                (cutoff_hz * self.fft_size as f64 / self.sample_rate as f64).round() as usize;

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];

                // Apply filter: zero out amplitudes below cutoff
                for bin in 0..cutoff_bin.min(channel_data.len()) {
                    channel_data[bin] = Complex64::new(0.0, 0.0);
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply a bandpass filter between the specified low and high cutoff frequencies.
    ///
    /// This attenuates frequencies outside the specified range.
    pub fn apply_bandpass(&mut self, low_cutoff_hz: f64, high_cutoff_hz: f64) -> Result<()> {
        if low_cutoff_hz >= high_cutoff_hz {
            return Err("Low cutoff frequency must be less than high cutoff frequency".to_string());
        }

        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Calculate the bin indices corresponding to the cutoff frequencies
            let low_bin =
                (low_cutoff_hz * self.fft_size as f64 / self.sample_rate as f64).round() as usize;
            let high_bin =
                (high_cutoff_hz * self.fft_size as f64 / self.sample_rate as f64).round() as usize;

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];

                // Apply filter: zero out amplitudes outside the band
                for bin in 0..channel_data.len() {
                    if bin < low_bin || bin > high_bin {
                        channel_data[bin] = Complex64::new(0.0, 0.0);
                    }
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply a chord filter to isolate specific frequencies and their harmonics
    ///
    /// This operation keeps only frequency bins that correspond to specified chord notes
    /// and their harmonics, attenuating everything else.
    ///
    /// # Parameters
    ///
    /// * `frequencies` - Array of frequencies to keep (Hz, 0 to skip)
    /// * `amplitudes` - Relative amplitude for each frequency (0-1, 0 to skip)
    /// * `width_cents` - Width around each frequency to keep (0-50 cents)
    /// * `harmonics_strength` - Harmonic strength factor (0-1)
    ///   - 0: Only fundamental frequencies
    ///   - 0.5: Harmonics follow sawtooth wave amplitude decay (1/n)
    ///   - 1: Harmonics have equal strength as fundamental
    pub fn apply_chord_filter(
        &mut self,
        frequencies: [f64; 5],
        amplitudes: [f64; 5],
        width_cents: f64,
        harmonics_strength: f64,
    ) -> Result<()> {
        if width_cents < 0.0 || width_cents > 50.0 {
            return Err("Width must be between 0 and 50 cents".to_string());
        }

        if harmonics_strength < 0.0 || harmonics_strength > 1.0 {
            return Err("Harmonics strength must be between 0 and 1".to_string());
        }

        if let Some(data_channels) = &mut self.fft_data {
            // Create a copy of the original data for calculating the filtered result
            let original_data = data_channels.clone();

            // Process each channel
            for (channel_idx, channel_data) in data_channels.iter_mut().enumerate() {
                // Calculate the frequency resolution (Hz per bin)
                let freq_resolution = self.sample_rate as f64 / self.fft_size as f64;

                // Create an amplitude envelope for each bin, initialized to zero
                let mut bin_amplitudes: Vec<f64> = vec![0.0; channel_data.len()];

                // Process each valid frequency in the chord
                for i in 0..5 {
                    let freq = frequencies[i];
                    let amp = amplitudes[i];

                    // Skip if frequency or amplitude is zero
                    if freq <= 0.0 || amp <= 0.0 {
                        continue;
                    }

                    // Number of harmonics to consider
                    // Limit to Nyquist frequency (sample_rate/2)
                    let nyquist_freq = self.sample_rate as f64 / 2.0;
                    let max_harmonic = if harmonics_strength > 0.0 {
                        (nyquist_freq / freq).floor() as usize
                    } else {
                        1 // Only fundamental if harmonics_strength is 0
                    };

                    // Process each harmonic
                    for harmonic in 1..=max_harmonic {
                        let harmonic_freq = freq * harmonic as f64;

                        // Skip if harmonic frequency exceeds Nyquist
                        if harmonic_freq >= nyquist_freq {
                            break;
                        }

                        // Calculate harmonic amplitude (1/n decay for sawtooth wave)
                        let harmonic_amp = if harmonic == 1 {
                            amp // Fundamental frequency has full amplitude
                        } else {
                            // Linear interpolation between no harmonics (0) and sawtooth harmonics (1)
                            if harmonics_strength > 0.5 {
                                let harm_amp_1_n = 1. / harmonic as f64;
                                let harm_amp_dist_to_1 = 1. - harm_amp_1_n;
                                amp * (harm_amp_1_n
                                    + (harmonics_strength - 0.5) * 2. * harm_amp_dist_to_1)
                            } else {
                                amp * 2. * harmonics_strength / harmonic as f64
                            }
                        };

                        // Calculate width in Hz based on cents
                        // One semitone = 100 cents = frequency ratio of 2^(1/12)
                        // So width_cents = width_ratio of 2^(width_cents/1200)
                        let width_ratio = 2.0f64.powf(width_cents / 1200.0);
                        let width_hz = harmonic_freq * (width_ratio - 1.0);

                        // Calculate the range of bins affected by this frequency and width
                        let center_bin = (harmonic_freq / freq_resolution).round() as usize;
                        let half_width_bins = (width_hz / freq_resolution).ceil() as usize;

                        // Apply amplitude envelope to bins around the harmonic frequency
                        let start_bin = if center_bin > half_width_bins {
                            center_bin - half_width_bins
                        } else {
                            1
                        };
                        let end_bin =
                            std::cmp::min(center_bin + half_width_bins, channel_data.len() - 1);

                        for bin in start_bin..=end_bin {
                            // Skip DC and Nyquist bins (0 and last)
                            if bin == 0 || bin == channel_data.len() - 1 {
                                continue;
                            }

                            // Calculate bin frequency
                            let bin_freq = bin as f64 * freq_resolution;

                            // Calculate distance in cents from the harmonic frequency
                            // cents = 1200 * log2(freq_ratio)
                            let cents_distance = 1200.0 * (bin_freq / harmonic_freq).abs().log2();

                            if cents_distance <= width_cents {
                                // Linear amplitude falloff based on cents distance
                                let distance_factor = 1.0 - cents_distance / width_cents;
                                let bin_contribution: f64 = harmonic_amp * distance_factor;

                                // Take the maximum contribution from all harmonics of all chord notes
                                bin_amplitudes[bin] = bin_amplitudes[bin].max(bin_contribution);
                            }
                        }
                    }
                }

                // Apply the amplitude envelope to the original data
                for bin in 0..channel_data.len() {
                    if bin == 0 {
                        // Preserve DC component
                        continue;
                    }

                    let (amp, phase) = original_data[channel_idx][bin].to_polar();
                    // Apply envelope to amplitude, preserving phase
                    let new_amp = amp * bin_amplitudes[bin];
                    channel_data[bin] = Complex64::from_polar(new_amp, phase);
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply phase shift to all frequencies.
    ///
    /// This shifts the phase of all frequency components by the specified amount in radians.
    pub fn apply_phase_shift(&mut self, phase_shift: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];

                // Apply phase shift to all bins
                for bin in 0..channel_data.len() {
                    // Add phase shift while preserving amplitude
                    let (amp, phase) = channel_data[bin].to_polar();
                    channel_data[bin] =
                        Complex64::from_polar(amp, Self::normalize_phase(phase + phase_shift));
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply phase multiplication to all frequencies.
    ///
    /// This multiplies the phase of each frequency component by the specified factor,
    /// creating interesting audio effects by disrupting the time-domain relationships.
    ///
    /// After multiplication, phases are normalized to the range [-?, ?] using modulo operations.
    pub fn apply_phase_multiply(&mut self, factor: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];

                // Apply phase multiplication to all bins (excluding DC and Nyquist for safety)
                for bin in 0..channel_data.len() {
                    // DC (bin 0) and Nyquist (last bin) components should have zero phase
                    if bin == 0 || bin == channel_data.len() - 1 {
                        channel_data[bin] = Complex64::new(channel_data[bin].re, 0.0);
                        continue;
                    }
                    let (amp, phase) = channel_data[bin].to_polar();
                    channel_data[bin] =
                        Complex64::from_polar(amp, Self::normalize_phase(phase * factor));
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    pub fn normalize_phase(phase: f64) -> f64 {
        // Normalize the phase to the range [-?, ?]
        // First get it to [0, 2?) with the modulo
        let normalized_phase = ((phase % (2.0 * std::f64::consts::PI))
            + (2.0 * std::f64::consts::PI))
            % (2.0 * std::f64::consts::PI);

        // Then shift values above ? to the [-?, ?] range
        if normalized_phase > std::f64::consts::PI {
            normalized_phase - 2.0 * std::f64::consts::PI
        } else {
            normalized_phase
        }
    }

    /// Swap random frequency bins to create interesting spectral effects.
    ///
    /// This function randomly swaps frequency bins within a specified range, causing
    /// frequency content to be relocated, which creates unique timbral changes.
    ///
    /// # Parameters
    ///
    /// * `block_size` - The maximum distance between swapped bins, as a percentage (0.0-100.0)
    ///   of the total frequency range
    /// * `repeat` - Number of swaps to perform per channel as a percentage (0.0-100.0)
    ///   of the number of bins
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Success or an error message
    pub fn swap_bins(&mut self, block_size: f64, repeat: f64) -> Result<()> {
        if block_size < 0.0 || block_size > 100.0 {
            return Err("Block size must be between 0 and 100 percent".to_string());
        }
        if repeat < 0.0 || repeat > 100.0 {
            return Err("Repeat size must be between 0 and 100 percent".to_string());
        }

        if let Some(data_channels) = &mut self.fft_data {
            // Import the random number generator
            use rand::Rng;

            // Initialize the random number generator
            let mut rng = rand::rng();

            // Process each channel independently
            for channel_idx in 0..data_channels.len() {
                let channel_data = &mut data_channels[channel_idx];
                let num_bins = channel_data.len();

                // The DC and Nyquist components need special handling,
                // so we'll exclude them from our swapping
                let first_bin = 1;
                let last_bin = num_bins - 2;

                if last_bin <= first_bin {
                    // Not enough bins to swap
                    return Err("Not enough frequency bins to perform swapping".to_string());
                }

                // Calculate the maximum swap distance in bins
                let max_distance = (block_size / 100.0 * (num_bins as f64)) as usize;
                if max_distance == 0 {
                    // Block size too small
                    return Err("Block size too small for the given FFT size".to_string());
                }
                let repeat_count = (repeat * (num_bins as f64)) as usize;

                // Perform the specified number of swaps
                for _ in 0..repeat_count {
                    // Choose a random bin within the valid range
                    let bin1 = rng.random_range(first_bin..=last_bin);

                    // Calculate the range for the second bin based on block_size
                    let min_offset = -(max_distance as i64 / 2);
                    let max_offset = max_distance as i64 / 2;

                    // Generate a random offset different from 0 (no swap with self)
                    let mut offset = 0;
                    while offset == 0 {
                        offset = rng.random_range(min_offset..=max_offset);
                    }

                    // Calculate second bin with wrap-around
                    let bin2_i64 = bin1 as i64 + offset;

                    // Ensure bin2 is within valid range
                    let bin2 = if bin2_i64 < first_bin as i64 {
                        // Wrap around to high end
                        last_bin - (first_bin as i64 - bin2_i64) as usize + 1
                    } else if bin2_i64 > last_bin as i64 {
                        // Wrap around to low end
                        first_bin + (bin2_i64 - last_bin as i64) as usize - 1
                    } else {
                        // Within range
                        bin2_i64 as usize
                    };

                    // Ensure we're not swapping with DC or Nyquist bins
                    if bin2 >= first_bin && bin2 <= last_bin && bin1 != bin2 {
                        // Swap the two bins
                        channel_data.swap(bin1, bin2);
                    }
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Randomly swap bins between different channels to create interesting stereo effects.
    ///
    /// This function swaps frequency data between different channels at randomly selected
    /// bin positions, which can create unique spatial and stereo effects.
    ///
    /// # Parameters
    ///
    /// * `repeat` - The number of bin swaps to perform as a percentage (0.0-100.0)
    ///   of the number of bins
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Success or an error message
    pub fn swap_channels(&mut self, repeat: f64) -> Result<()> {
        if repeat < 0.0 || repeat > 100.0 {
            return Err("Repeat size must be between 0 and 100 percent".to_string());
        }

        // Need at least 2 channels for swapping
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            if num_channels < 2 {
                return Err("Need at least 2 channels to perform channel swapping".to_string());
            }

            // Import the random number generator
            use rand::Rng;

            // Initialize the random number generator
            let mut rng = rand::rng();

            // Get the number of bins (should be the same for all channels)
            let num_bins = data_channels[0].len();

            let repeat_count = (repeat * (num_bins as f64)) as usize;

            // Perform the specified number of swaps
            for _ in 0..repeat_count {
                // Choose a random bin within the valid range (avoid DC and Nyquist)
                let bin = 1 + rng.random_range(0..(num_bins - 2));

                // Choose first channel randomly
                let chan1 = rng.random_range(0..num_channels);

                // Choose second channel randomly (must be different)
                let mut chan2 = chan1;
                while chan2 == chan1 {
                    chan2 = rng.random_range(0..num_channels);
                }

                // Swap the bin data between the two channels
                let temp = data_channels[chan1][bin];
                data_channels[chan1][bin] = data_channels[chan2][bin];
                data_channels[chan2][bin] = temp;
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply a spectrum shift to move frequency content up or down
    pub fn apply_spectrum_shift(&mut self, shift_hz: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Calculate bin shift based on frequency resolution
            let freq_resolution = self.sample_rate as f64 / self.fft_size as f64;
            let bin_shift = (shift_hz / freq_resolution).round() as i32;

            // Create temporary storage for the shifted data
            let mut shifted_data = Vec::with_capacity(num_channels);
            for channel_idx in 0..num_channels {
                let bins_per_channel = data_channels[channel_idx].len();
                shifted_data.push(vec![Complex64::new(0.0, 0.0); bins_per_channel]);
            }

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &data_channels[channel_idx];
                let channel_shifted = &mut shifted_data[channel_idx];
                let nyquist_bin = channel_data.len() - 1;

                // Process bins (exclude DC and Nyquist components)
                for i in 1..nyquist_bin {
                    let target_bin = i as i32 + bin_shift;

                    // Ensure target bin is within valid range (not DC or Nyquist)
                    if target_bin > 0 && target_bin < nyquist_bin as i32 {
                        channel_shifted[target_bin as usize] = channel_data[i];
                    }
                    // Bins shifted outside the range are discarded (set to zero)
                }

                // Always preserve DC component (bin 0)
                channel_shifted[0] = channel_data[0];

                // Always preserve Nyquist component (last bin)
                channel_shifted[nyquist_bin] = channel_data[nyquist_bin];
            }

            // Replace the original data with shifted data
            for channel_idx in 0..num_channels {
                data_channels[channel_idx] = shifted_data[channel_idx].clone();
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply a non-linear frequency stretch to the spectrum
    ///
    /// This operation raises the bin indices to a specified power, resulting
    /// in either compression or expansion of different parts of the spectrum.
    ///
    /// # Parameters
    ///
    /// * `exponent` - The power to raise bin indices to. Values > 1 compress high
    ///   frequencies and expand low frequencies, while values < 1 do the opposite.
    pub fn apply_stretch(&mut self, exponent: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Create temporary storage for the stretched data
            let mut stretched_data = Vec::with_capacity(num_channels);
            for channel_idx in 0..num_channels {
                let bins_per_channel = data_channels[channel_idx].len();
                stretched_data.push(vec![Complex64::new(0.0, 0.0); bins_per_channel]);
            }

            // Calculate the scaling factor to keep the maximum bin the same
            let max_bin = (data_channels[0].len() - 1) as f64;
            let scale = max_bin / (max_bin.powf(exponent));

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &data_channels[channel_idx];
                let channel_stretched = &mut stretched_data[channel_idx];
                let nyquist_bin = channel_data.len() - 1;

                // Always preserve DC component (bin 0)
                channel_stretched[0] = channel_data[0];

                // Process bins (exclude DC component)
                for i in 1..nyquist_bin {
                    // Calculate the source bin using the stretch formula
                    let i_f64 = i as f64;
                    let src_bin = (i_f64.powf(exponent) * scale).round() as usize;

                    // Ensure source bin is within valid range
                    if src_bin < nyquist_bin {
                        channel_stretched[i] = channel_data[src_bin];
                    }
                }

                // Always preserve Nyquist component (last bin)
                channel_stretched[nyquist_bin] = channel_data[nyquist_bin];
            }

            // Replace the original data with stretched data
            for channel_idx in 0..num_channels {
                data_channels[channel_idx] = stretched_data[channel_idx].clone();
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply a wobble effect to the frequency spectrum
    ///
    /// This operation creates a sinusoidal modulation of bin positions,
    /// resulting in a wobbling effect in the frequency domain.
    ///
    /// # Parameters
    ///
    /// * `frequency` - The frequency of the wobble modulation (higher values create more cycles)
    /// * `amplitude` - The amplitude of the wobble (between 0.0 and 1.0, controlling displacement amount)
    pub fn apply_wobble(&mut self, frequency: f64, amplitude: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Create temporary storage for the wobbled data
            let mut wobbled_data = Vec::with_capacity(num_channels);
            for channel_idx in 0..num_channels {
                let bins_per_channel = data_channels[channel_idx].len();
                wobbled_data.push(vec![Complex64::new(0.0, 0.0); bins_per_channel]);
            }

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &data_channels[channel_idx];
                let channel_wobbled = &mut wobbled_data[channel_idx];
                let nyquist_bin = channel_data.len() - 1;

                // Always preserve DC component (bin 0)
                channel_wobbled[0] = channel_data[0];

                // Process bins (exclude DC component)
                for i in 1..nyquist_bin {
                    // Calculate wobble using sine function, matching the C code's formula
                    let i_f64 = i as f64;
                    let wobble_factor = 0.5
                        * (f64::sin(
                            4.0 * std::f64::consts::PI * i_f64 * frequency / (nyquist_bin as f64),
                        ) + 1.0);
                    let displacement = wobble_factor * amplitude * (nyquist_bin as f64) / 4.0;
                    let src_bin = (i_f64 + displacement).round() as usize;

                    // Ensure source bin is within valid range
                    if src_bin > 0 && src_bin < nyquist_bin {
                        channel_wobbled[i] = channel_data[src_bin];
                    }
                }

                // Always preserve Nyquist component (last bin)
                channel_wobbled[nyquist_bin] = channel_data[nyquist_bin];
            }

            // Replace the original data with wobbled data
            for channel_idx in 0..num_channels {
                data_channels[channel_idx] = wobbled_data[channel_idx].clone();
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply a threshold filter to the frequency spectrum
    ///
    /// This operation removes frequency components based on their amplitude.
    /// It can either remove components below a threshold (noise gate)
    /// or above a threshold (peak limiter).
    ///
    /// # Parameters
    ///
    /// * `threshold_level` - The threshold level to compare amplitudes against
    /// * `remove_above_threshold` - If true, removes components above the threshold;
    ///   if false, removes components below the threshold
    pub fn apply_threshold(
        &mut self,
        threshold_level: f64,
        remove_above_threshold: bool,
    ) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];
                let nyquist_bin = channel_data.len() - 1;

                // Process bins (including all except DC component which we always keep)
                for i in 1..=nyquist_bin {
                    // Calculate amplitude from polar representation
                    let (mut amplitude, _) = channel_data[i].to_polar();
                    amplitude = amplitude.abs();

                    // Apply threshold operation
                    if remove_above_threshold {
                        if amplitude > threshold_level {
                            // Zero out amplitudes above threshold
                            channel_data[i] = Complex64::new(0.0, 0.0);
                        }
                    } else {
                        if amplitude < threshold_level {
                            // Zero out amplitudes below threshold
                            channel_data[i] = Complex64::new(0.0, 0.0);
                        }
                    }
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply an amplitude derivative operation to the frequency spectrum
    ///
    /// This operation replaces each frequency component's amplitude with
    /// the difference between its amplitude and the previous bin's amplitude,
    /// multiplied by a scaling factor. This creates interesting spectral effects
    /// by emphasizing changes in the spectrum.
    ///
    /// # Parameters
    ///
    /// * `multiplier` - A scaling factor for the derivative values
    pub fn apply_amplitude_derivative(&mut self, multiplier: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Process each channel independently
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];
                let bin_count = channel_data.len();

                // Always preserve DC component (bin 0)
                let mut last_amplitude = channel_data[0].re;

                // Process bins (excluding DC component)
                for i in 1..bin_count {
                    let (current_amplitude, phase) = channel_data[i].to_polar();

                    // Calculate amplitude derivative
                    let amplitude_derivative = (current_amplitude - last_amplitude) * multiplier;

                    // Replace amplitude with derivative, preserve phase
                    channel_data[i] = Complex64::from_polar(amplitude_derivative, phase);

                    // Update last amplitude for next iteration
                    last_amplitude = current_amplitude;
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply a "keep peaks" filter to the frequency spectrum
    ///
    /// This operation keeps only the local maxima in the frequency spectrum,
    /// zeroing out all bins that aren't local peaks. It compares each bin with
    /// its neighbors and keeps only those bins that have higher amplitude than
    /// both their neighbors.
    pub fn keep_peaks(&mut self) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Create temporary copy of the data for comparison
            let temp_data = data_channels.clone();

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel = &mut data_channels[channel_idx];
                let channel_temp = &temp_data[channel_idx];
                let bin_count = channel.len();

                // Always preserve DC component (bin 0)

                // Process bins (excluding DC and the last bin)
                for i in 1..(bin_count - 1) {
                    // Get amplitudes of current bin and its neighbors
                    let (amp_prev, _) = channel_temp[i - 1].to_polar();
                    let (amp_curr, phase) = channel_temp[i].to_polar();
                    let (amp_next, _) = channel_temp[i + 1].to_polar();

                    // Check if current bin is a local maximum
                    if amp_curr < amp_prev || amp_curr < amp_next {
                        // Not a peak, zero out the bin
                        channel[i] = Complex64::from_polar(0.0, phase);
                    }
                    // If it is a peak, keep it as is
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }
    /// Mix channels with specified weights.
    ///
    /// This creates a new single-channel output by mixing the input channels.
    pub fn mix_channels(&mut self, weights: &[f64]) -> Result<()> {
        if let Some(time_data_channels) = &self.time_data {
            let num_channels = time_data_channels.len();

            if weights.len() != num_channels {
                return Err(format!(
                    "Number of weights ({}) must match number of channels ({})",
                    weights.len(),
                    num_channels
                ));
            }

            // Create a new single channel for the mixed output
            let samples_per_channel = time_data_channels[0].len();
            let mut mixed_channel = vec![0.0; samples_per_channel];

            // Mix channels according to weights
            for i in 0..samples_per_channel {
                for (channel_idx, weight) in weights.iter().enumerate() {
                    mixed_channel[i] += time_data_channels[channel_idx][i] * weight;
                }
            }

            // Replace the time data with the new mixed channel
            self.time_data = Some(vec![mixed_channel]);
            self.channels = 1;

            // Clear FFT data as it's now invalid
            self.fft_data = None;

            Ok(())
        } else {
            Err("No time domain data available for channel mixing".to_string())
        }
    }

    /// Split the frequency spectrum into multiple parts, with each part
    /// containing only selected frequency bins.
    ///
    /// This function creates a new FFT representation that contains only a subset of
    /// the original frequency bins, with all other bins set to zero. This allows for
    /// separating different frequency components into different files.
    ///
    /// # Parameters
    ///
    /// * `part_index` - Which part of the split to prepare (0 to num_parts-1)
    /// * `num_parts` - The number of parts to split the spectrum into
    /// * `group_size` - How many consecutive bins to group together (default: 1)
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Success or an error message
    pub fn prepare_split_part(
        &mut self,
        part_index: usize,
        num_parts: usize,
        group_size: usize,
    ) -> Result<()> {
        if num_parts == 0 {
            return Err("Number of parts must be greater than zero".to_string());
        }

        if part_index >= num_parts {
            return Err(format!(
                "Part index {} is out of range (0 to {})",
                part_index,
                num_parts - 1
            ));
        }

        if group_size == 0 {
            return Err("Group size must be greater than zero".to_string());
        }

        if let Some(data) = &self.fft_data {
            let num_channels = data.len();
            let bins_per_channel = data[0].len();

            // Create a new Cartesian FFT data array with all zeros
            let mut new_fft_data = Vec::with_capacity(num_channels);
            for _ in 0..num_channels {
                new_fft_data.push(vec![Complex64::new(0.0, 0.0); bins_per_channel]);
            }

            // Copy only the bins that belong to this part and convert from polar to Cartesian
            for channel in 0..num_channels {
                // Special handling for DC component (bin 0)
                if part_index == 0 {
                    // DC component (bin 0) goes to the first part only
                    // DC should only have real component (amplitude)
                    new_fft_data[channel][0] = Complex64::new(data[channel][0].re, 0.0);
                }

                // For all other bins, assign based on the group index
                for bin in 1..bins_per_channel {
                    let group_index = (bin / group_size) % num_parts;

                    if group_index == part_index {
                        new_fft_data[channel][bin] =
                            Complex64::new(data[channel][bin].re, data[channel][bin].im);
                    }
                    // All other bins remain zero
                }
            }

            //
            self.tmp_fft_data = self.fft_data.take();
            // Store the new Cartesian FFT data for this part
            self.fft_data = Some(new_fft_data);

            Ok(())
        } else {
            Err("No FFT data available. Load a file and perform FFT first.".to_string())
        }
    }

    /// Reset the processor by setting back original data
    pub fn reset_split(&mut self) -> Result<()> {
        self.fft_data = self.tmp_fft_data.take();
        Ok(())
    }

    /// Reset the processor to the original state completely,
    /// by redoing the FFT on the original time domain data.
    pub fn reset_to_original(&mut self) -> Result<()> {
        if let Some(time_data) = &self.time_data {
            // Simply redo the FFT on the original time data
            let sample_rate = self.sample_rate;
            let channels = self.channels;

            // Make a deep copy of the time data
            let time_data_copy = time_data.clone();

            // Reset the processor with the original time data
            self.set_audio_data(sample_rate, channels, time_data_copy)?;
            self.perform_fft()?;

            Ok(())
        } else {
            Err("No original time domain data available".to_string())
        }
    }
}

/// Information about audio data loaded in the processor.
#[derive(Debug, Clone)]
pub struct AudioInfo {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of audio channels
    pub channels: u16,
    /// FFT size (power of 2)
    pub fft_size: usize,
    /// Information about time domain data (if available)
    pub time_data: Option<TimeDataInfo>,
    /// Information about frequency domain data (if available)
    pub fft_data: Option<FFTDataInfo>,
}

/// Information about time domain data.
#[derive(Debug, Clone)]
pub struct TimeDataInfo {
    /// Number of channels
    pub num_channels: usize,
    /// Number of samples per channel
    pub samples_per_channel: usize,
    /// Duration in seconds
    pub duration_seconds: f64,
}

/// Information about frequency domain data.
#[derive(Debug, Clone)]
pub struct FFTDataInfo {
    /// Number of channels
    pub num_channels: usize,
    /// Number of complex values per channel
    pub complex_values_per_channel: usize,
    /// Frequency resolution in Hz
    pub frequency_resolution: f64,
}
