// lib/src/operations/convolution.rs

//! Convolution and correlation operations for the FFT Audio Processor.
//!
//! This module contains operations for convolving and correlating audio signals
//! in the frequency domain, which can be used for effects like reverb,
//! echo simulation, and audio analysis.

use num_complex::Complex64;
use log::{info, warn};

use crate::audio_read_write::read_audio_bytes;
use crate::processor::AudioProcessor;
use crate::Result;

#[cfg(not(target_arch = "wasm32"))]
use crate::audio_read_write::read_audio_file;

impl AudioProcessor {
    /// Loads another audio file and performs convolution or correlation with the current audio.
    ///
    /// # Parameters
    ///
    /// * `file_data` - Audio file data as bytes
    /// * `correlate` - If true, performs correlation; if false, performs convolution
    /// * `wet_mix` - Mix ratio between original and processed signal (0.0 = only original, 1.0 = only processed)
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Success or an error message
    pub fn convolve_with_bytes(
        &mut self,
        file_data: Vec<u8>,
        correlate: bool,
        wet_mix: f64,
    ) -> Result<()> {
        if !(0.0..=1.0).contains(&wet_mix) {
            return Err("Wet mix must be between 0.0 and 1.0".to_string());
        }

        // Ensure we have FFT data
        if self.fft_data.is_none() {
            return Err("No FFT data available in the current processor".to_string());
        }

        // Load the impulse response file
        let (ir_sample_rate, ir_channels) = match read_audio_bytes(file_data) {
            Ok((sample_rate, channels)) => (sample_rate, channels),
            Err(e) => return Err(format!("Error reading impulse response file: {}", e)),
        };

        // Check if sample rates match
        if ir_sample_rate != self.sample_rate {
            warn!("Sample rate mismatch: current audio is at {} Hz, impulse response is at {} Hz",
                self.sample_rate, ir_sample_rate);
        }

        // Create a temporary processor for the impulse response
        let mut ir_processor = AudioProcessor::new();
        ir_processor.set_audio_data_with_length(
            ir_sample_rate,
            ir_channels.len() as u16,
            ir_channels,
            self.time_data().unwrap()[0].len(),
        )?;

        // For convolution, we need to time-reverse the impulse response if not correlating
        if correlate {
            if let Some(time_data) = &mut ir_processor.time_data {
                for channel in time_data.iter_mut() {
                    channel.reverse();
                }
            }
        }

        // Perform FFT on the impulse response
        ir_processor.perform_fft()?;

        // Now perform complex multiplication in the frequency domain
        self.multiply_spectra(&ir_processor, wet_mix)?;
        info!("Convolution done.");
        Ok(())
    }

    /// Loads another audio file and performs convolution or correlation with the current audio.
    ///
    /// # Parameters
    ///
    /// * `filename` - Path to the audio file
    /// * `correlate` - If true, performs correlation; if false, performs convolution
    /// * `wet_mix` - Mix ratio between original and processed signal (0.0 = only original, 1.0 = only processed)
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Success or an error message
    #[cfg(not(target_arch = "wasm32"))]
    pub fn convolve_with_file(
        &mut self,
        filename: &str,
        correlate: bool,
        wet_mix: f64,
    ) -> Result<()> {
        if !(0.0..=1.0).contains(&wet_mix) {
            return Err("Wet mix must be between 0.0 and 1.0".to_string());
        }

        // Ensure we have FFT data
        if self.fft_data.is_none() {
            return Err("No FFT data available in the current processor".to_string());
        }

        // Load the impulse response file
        let (ir_sample_rate, ir_channels) = match read_audio_file(filename) {
            Ok((sample_rate, channels)) => (sample_rate, channels),
            Err(e) => return Err(format!("Error reading impulse response file: {}", e)),
        };

        // Check if sample rates match
        if ir_sample_rate != self.sample_rate {
            warn!("Sample rate mismatch: current audio is at {} Hz, impulse response is at {} Hz",
                self.sample_rate, ir_sample_rate);
        }

        // Create a temporary processor for the impulse response
        let mut ir_processor = AudioProcessor::new();
        ir_processor.set_audio_data_with_length(
            ir_sample_rate,
            ir_channels.len() as u16,
            ir_channels,
            self.time_data().unwrap()[0].len(),
        )?;

        if correlate {
            if let Some(time_data) = &mut ir_processor.time_data {
                for channel in time_data.iter_mut() {
                    channel.reverse();
                }
            }
        }

        // Perform FFT on the impulse response
        ir_processor.perform_fft()?;

        // Now perform complex multiplication in the frequency domain
        self.multiply_spectra(&ir_processor, wet_mix)?;
        info!("Convolution done.");
        Ok(())
    }

    /// Helper function to perform complex multiplication in the frequency domain.
    ///
    /// # Parameters
    ///
    /// * `other` - Another AudioProcessor containing the FFT data to multiply with
    /// * `wet_mix` - Mix ratio between original and processed signal
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Success or an error message
    fn multiply_spectra(&mut self, other: &AudioProcessor, wet_mix: f64) -> Result<()> {
        if let (Some(self_fft_data), Some(other_fft_data)) = (&mut self.fft_data, &other.fft_data) {
            if self_fft_data.len() != other_fft_data.len() {
                return Err("Number of channels must be the same".to_string());
            }
            let num_channels = self_fft_data.len();

            let num_bins = self_fft_data[0].len();

            // Process each channel
            for channel_idx in 0..num_channels {
                let self_channel = &mut self_fft_data[channel_idx];
                let other_channel = &other_fft_data[channel_idx];

                if self_channel.len() != other_channel.len() || self_channel.len() != num_bins {
                    return Err("Number of bins must be the same".to_string());
                }

                // Make a copy of the original data for wet/dry mixing
                let original_data: Vec<Complex64> = self_channel[0..num_bins].to_vec();

                // Perform complex multiplication
                for bin in 0..num_bins {
                    // DC (bin 0) and Nyquist (if present) need special handling
                    if bin == 0 || bin == num_bins - 1 {
                        let self_amp = self_channel[bin].norm();
                        let other_amp = other_channel[bin].norm();

                        // Multiply real parts (imaginary parts should be zero)
                        self_channel[bin] =
                            Complex64::new(self_channel[bin].re * other_channel[bin].re, 0.0);

                        // Normalize to avoid extreme amplitude changes
                        if other_amp > 0.0 {
                            self_channel[bin] = self_channel[bin] * (self_amp / other_amp).sqrt();
                        }
                    } else {
                        // Regular bins: perform complex multiplication
                        self_channel[bin] = self_channel[bin] * other_channel[bin];
                    }
                }

                // Apply wet/dry mix if not 100% wet
                if wet_mix < 1.0 {
                    let dry_mix = 1.0 - wet_mix;
                    for bin in 0..num_bins {
                        self_channel[bin] =
                            original_data[bin] * dry_mix + self_channel[bin] * wet_mix;
                    }
                }
            }

            Ok(())
        } else {
            Err("FFT data missing in one or both processors".to_string())
        }
    }
}
