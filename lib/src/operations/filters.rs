// lib/src/operations/filters.rs

//! Filter operations for the FFT Audio Processor.
//!
//! This module contains various frequency domain filters such as lowpass,
//! highpass, bandpass, and chord filters that can be applied to the audio data.

use num_complex::Complex64;

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
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
}
