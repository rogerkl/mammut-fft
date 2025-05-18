// lib/src/operations/threshold.rs

//! Threshold operations for the FFT Audio Processor.
//!
//! This module contains operations that apply amplitude thresholds to
//! frequency components, such as noise gating or peak limiting.

use num_complex::Complex64;

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
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
}
