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
    /// Distribute bins linearly, using either real or complex layout
    /// num_freq_bin is the number if frequencies we have
    /// Example we do fft on 1024 sample points = 512 num_freq_bins
    /// Real fft: dc + num_freqs = 513 bins (0=dc, 512=nyquist)
    /// Complex fft: dc + num_freqs + num_freqs-1 = 1024 bins (0=dc, 512=nyquist)
    pub fn distribute_lin_bins(
        num_freq_bins: usize,
        num_parts: usize,
        complex: bool,
    ) -> Vec<usize> {
        let num_bins = if complex {
            num_freq_bins * 2
        } else {
            num_freq_bins + 1
        };
        let mut bin_assignments = vec![0; num_bins];

        // Assign each bin to a part
        for bin in 0..num_freq_bins + 1 {
            if bin == 0 {
                // DC component - assign to first part
                bin_assignments[bin] = 0;
            } else {
                bin_assignments[bin] = ((((bin - 1) as f32) / (num_freq_bins as f32))
                    * (num_parts as f32))
                    .floor() as usize;
                if complex && bin <= num_freq_bins {
                    bin_assignments[num_bins - bin] = bin_assignments[bin];
                }
            }
        }
        bin_assignments
    }

    /// Distribute bins logarithmicly, using either real or complex layout
    /// num_freq_bin is the number if frequencies we have
    /// Example we do fft on 1024 sample points = 512 num_freq_bins
    /// Real fft: dc + num_freqs = 513 bins (0=dc, 512=nyquist)
    /// Complex fft: dc + num_freqs + num_freqs-1 = 1024 bins (0=dc, 512=nyquist)
    pub fn distribute_log_bins(
        num_freq_bins: usize,
        num_parts: usize,
        complex: bool,
    ) -> Vec<usize> {
        let num_bins = if complex {
            num_freq_bins * 2
        } else {
            num_freq_bins + 1
        };
        let mut bin_assignments = vec![0; num_bins];

        // don't really need correct samplerate ...
        let sample_rate = num_freq_bins as f32;
        // Calculate frequency for each bin (excluding DC and Nyquist)
        // Start from bin 1 to avoid log(0)
        let min_freq = sample_rate / (2.0 * num_freq_bins as f32); // Frequency of bin 1
        let max_freq = sample_rate / 2.0; // Nyquist frequency

        // Calculate log frequency range
        let log_min = min_freq.ln();
        let log_max = max_freq.ln();
        let log_range = log_max - log_min;

        // Assign each bin to a part
        for bin in 0..num_freq_bins + 1 {
            if bin == 0 {
                // DC component - assign to first part
                bin_assignments[bin] = 0;
            } else {
                // Calculate frequency for this bin
                let freq = (bin as f32) * sample_rate / (2.0 * num_freq_bins as f32);

                // Convert to log scale
                let log_freq = freq.ln();

                // Normalize to 0-1 range
                let normalized = (log_freq - log_min) / log_range;

                // Calculate which part this bin belongs to
                let part = (normalized * num_parts as f32).floor() as usize;

                // Clamp to valid range (in case of floating point errors)
                bin_assignments[bin] = part.min(num_parts - 1);
                if complex && bin <= num_freq_bins {
                    bin_assignments[num_bins - bin] = bin_assignments[bin];
                }
            }
        }
        bin_assignments
    }

    /// Distribute bins with a group size, should pass either &distribute_lin_bins or &distribute_log_bins as the func parameter
    /// for linear distribution this is straightforward, say real fft, 16 bins,group size 2, num parts 4
    /// bin[0] = 0 (DC, special case)
    /// bin[1] = 0
    /// bin[2] = 0
    /// bin[3] = 1 // group size == 2, so change value after 2 bins
    /// bin[4] = 1
    /// bin[5] = 2
    /// bin[6] = 2
    /// bin[7] = 3
    /// bin[8] = 3
    /// bin[9] = 0 // number of parts reached, start at 0 again
    /// bin[10] = 0
    /// bin[11] = 1
    /// bin[12] = 1
    /// bin[13] = 2
    /// bin[14] = 2
    /// bin[15] = 3
    /// bin[16] = 4
    /// For the logarithmic case it's a little more complicated, the algorithm is:
    ///   we distribute first in num_freq_bins/group_size parts
    ///   then remap to num_parts using modulu operator
    /// The result is that we should still have the same musical frequencies per part as for
    /// the normal logarithmic distribution
    pub fn distribute_grouped(
        num_freq_bins: usize,
        num_parts: usize,
        complex: bool,
        group_size: usize,
        func: &dyn Fn(usize, usize, bool) -> Vec<usize>,
    ) -> Vec<usize> {
        if group_size < 1 {
            return func(num_freq_bins, num_parts, complex);
        }
        let mut bin_assignments = func(num_freq_bins, num_freq_bins / group_size, complex);
        for bin in 1..num_freq_bins + 1 {
            bin_assignments[bin] = bin_assignments[bin] % num_parts;
        }
        bin_assignments
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
        log: bool,
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

        let bin_assignments = if group_size == 0 {
            if log {
                Self::distribute_log_bins(self.fft_size()/2, num_parts, false)
            }
            else {
                Self::distribute_lin_bins(self.fft_size()/2, num_parts, false)
            }
        }
        else {
          Self::distribute_grouped(self.fft_size()/2, num_parts, false, group_size, if log {&Self::distribute_log_bins} else {&Self::distribute_lin_bins})
        };

        if let Some(data) = &self.fft_data {
            let num_channels = data.len();
            let bins_per_channel = data[0].len();

            if bins_per_channel != bin_assignments.len() {
                return Err("bins_per_channel <> bin_assignments".to_string());
            }

            // Create a new Cartesian FFT data array with all zeros
            let mut new_fft_data = Vec::with_capacity(num_channels);
            for _ in 0..num_channels {
                new_fft_data.push(vec![Complex64::new(0.0, 0.0); bins_per_channel]);
            }

            // Copy only the bins that belong to this part
            for channel in 0..num_channels {
                for bin in 1..bins_per_channel {
                    if(bin_assignments[bin] == part_index) {
                        new_fft_data[channel][bin] =
                            Complex64::new(data[channel][bin].re, data[channel][bin].im);
                    }
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
