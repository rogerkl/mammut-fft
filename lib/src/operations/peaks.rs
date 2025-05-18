// lib/src/operations/peaks.rs

//! Peak detection operations for the FFT Audio Processor.
//!
//! This module contains operations related to detecting and preserving
//! peak frequency components in the spectrum.

use num_complex::Complex64;

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
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
}
