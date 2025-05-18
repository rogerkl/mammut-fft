// lib/src/operations/amplitude.rs

//! Amplitude-related operations for the FFT Audio Processor.
//!
//! This module contains operations that modify the amplitude of frequency components,
//! such as raising amplitudes to a power or applying derivative-based effects.

use num_complex::Complex64;

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
    /// Apply a power function to the amplitude of the FFT data.
    ///
    /// This raises each amplitude value to the specified power while
    /// preserving the phase information.
    pub fn apply_pow(&mut self, exponent: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            // Process each channel independently
            for channel_data in data_channels.iter_mut() {
                for i in 0..channel_data.len() {
                    // Get the current amplitude (real part)
                    let (amplitude, phase) = channel_data[i].to_polar();

                    // Raise amplitude to the specified power
                    // Amplitude in polar form is always non-negative
                    let powered_amplitude = amplitude.powf(exponent);

                    // Update amplitude while preserving phase
                    channel_data[i] = Complex64::from_polar(powered_amplitude, phase);
                }
            }
            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply an amplitude derivative operation to the frequency spectrum
    ///
    /// This operation replaces each frequency component's amplitude with
    /// the difference between its amplitude and the previous bin's amplitude,
    /// multiplied by a scaling factor. This creates interesting spectral effects
    /// by emphasizing changes in the spectrum.
    ///
    /// # Parameters
    ///
    /// * `multiplier` - A scaling factor for the derivative values
    pub fn apply_amplitude_derivative(&mut self, multiplier: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Process each channel independently
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];
                let bin_count = channel_data.len();

                // Always preserve DC component (bin 0)
                let mut last_amplitude = channel_data[0].re;

                // Process bins (excluding DC component)
                for i in 1..bin_count {
                    let (current_amplitude, phase) = channel_data[i].to_polar();

                    // Calculate amplitude derivative
                    let amplitude_derivative = (current_amplitude - last_amplitude) * multiplier;

                    // Replace amplitude with derivative, preserve phase
                    channel_data[i] = Complex64::from_polar(amplitude_derivative, phase);

                    // Update last amplitude for next iteration
                    last_amplitude = current_amplitude;
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }
}
