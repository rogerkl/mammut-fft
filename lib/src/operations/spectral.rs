// lib/src/operations/spectral.rs

//! Spectral operations for the FFT Audio Processor.
//!
//! This module contains operations that manipulate the spectrum as a whole,
//! such as frequency shifting and stretching.

use num_complex::Complex64;

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
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
}
