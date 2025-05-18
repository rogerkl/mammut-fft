// lib/src/operations/phase.rs

//! Phase-related operations for the FFT Audio Processor.
//!
//! This module contains operations that modify the phase of frequency components,
//! such as phase shifting and phase multiplication.

use num_complex::Complex64;

use crate::processor::AudioProcessor;
use crate::Result;

impl AudioProcessor {
    /// Apply phase shift to all frequencies.
    ///
    /// This shifts the phase of all frequency components by the specified amount in radians.
    pub fn apply_phase_shift(&mut self, phase_shift: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];

                // Apply phase shift to all bins
                for bin in 0..channel_data.len() {
                    // Add phase shift while preserving amplitude
                    let (amp, phase) = channel_data[bin].to_polar();
                    channel_data[bin] =
                        Complex64::from_polar(amp, Self::normalize_phase(phase + phase_shift));
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }

    /// Apply phase multiplication to all frequencies.
    ///
    /// This multiplies the phase of each frequency component by the specified factor,
    /// creating interesting audio effects by disrupting the time-domain relationships.
    ///
    /// After multiplication, phases are normalized to the range [-π, π] using modulo operations.
    pub fn apply_phase_multiply(&mut self, factor: f64) -> Result<()> {
        if let Some(data_channels) = &mut self.fft_data {
            let num_channels = data_channels.len();

            // Process each channel
            for channel_idx in 0..num_channels {
                let channel_data = &mut data_channels[channel_idx];

                // Apply phase multiplication to all bins (excluding DC and Nyquist for safety)
                for bin in 0..channel_data.len() {
                    // DC (bin 0) and Nyquist (last bin) components should have zero phase
                    if bin == 0 || bin == channel_data.len() - 1 {
                        channel_data[bin] = Complex64::new(channel_data[bin].re, 0.0);
                        continue;
                    }
                    let (amp, phase) = channel_data[bin].to_polar();
                    channel_data[bin] =
                        Complex64::from_polar(amp, Self::normalize_phase(phase * factor));
                }
            }

            Ok(())
        } else {
            Err("No FFT data available. Load a file first.".to_string())
        }
    }
}
