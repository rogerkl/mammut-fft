// lib/src/operations/bin_swap.rs

//! Bin swapping operations for the FFT Audio Processor.
//!
//! This module contains operations that randomly swap frequency bins within
//! or between channels, creating interesting spectral effects.

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
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
        if !(0.0..=100.0).contains(&block_size) {
            return Err("Block size must be between 0 and 100 percent".to_string());
        }
        if !(0.0..=100.0).contains(&repeat) {
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
        if !(0.0..=100.0).contains(&repeat) {
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
}
