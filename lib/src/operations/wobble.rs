// lib/src/operations/wobble.rs

//! Wobble effect for the FFT Audio Processor.
//!
//! This module contains the wobble effect operation that creates a
//! sinusoidal modulation of frequency bin positions.

use num_complex::Complex64;

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
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
}
