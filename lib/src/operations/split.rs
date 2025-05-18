// lib/src/operations/split.rs

//! Split operations for the FFT Audio Processor.
//!
//! This module contains operations that split the frequency spectrum
//! into multiple parts, allowing different frequency components to be
//! manipulated independently.

use num_complex::Complex64;

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
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
