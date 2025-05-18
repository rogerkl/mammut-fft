// lib/src/operations/mixing.rs

//! Channel mixing operations for the FFT Audio Processor.
//!
//! This module contains operations that mix multiple audio channels
//! together with specified weights.

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
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
}
