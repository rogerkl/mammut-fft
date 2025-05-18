// lib/src/processor.rs

//! Core AudioProcessor implementation.
//!
//! This module defines the AudioProcessor struct and its core methods for
//! performing FFT/IFFT operations and maintaining audio data state.
//! Specific audio processing operations are implemented in the operations module.

use std::sync::Arc;

use num_complex::Complex64;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

use crate::Result;

/// The core FFT audio processing struct.
///
/// This struct handles all FFT-related operations on audio data,
/// including transformations between time and frequency domains,
/// as well as spectral manipulations.
pub struct AudioProcessor {
    /// Audio sample rate in Hz
    pub(crate) sample_rate: u32,
    /// Number of audio channels
    pub(crate) channels: u16,
    /// FFT data in cartesian form (real/imaginary) for each channel
    pub(crate) fft_data: Option<Vec<Vec<Complex64>>>,
    /// Temporary copy of data for split operation
    pub(crate) tmp_fft_data: Option<Vec<Vec<Complex64>>>,
    /// Time domain audio data for each channel
    pub(crate) time_data: Option<Vec<Vec<f64>>>,
    /// FFT forward transform planner
    pub(crate) fft_r2c: Option<Arc<dyn RealToComplex<f64>>>,
    /// FFT inverse transform planner
    pub(crate) fft_c2r: Option<Arc<dyn ComplexToReal<f64>>>,
    /// FFT size (power of 2)
    pub(crate) fft_size: usize,
    /// max amplitude of all channels in fft_polar_data, used for scaling to max level=1.
    /// according to realfft documentation the scaling for forward+inverse fft should be 1/length
    /// so we should then scale with max_amplitude after doing inverse_fft
    pub(crate) max_amplitude: f64,
}

// Manual implementation of Debug since the FFT planners don't implement Debug
impl std::fmt::Debug for AudioProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioProcessor")
            .field("sample_rate", &self.sample_rate)
            .field("channels", &self.channels)
            .field("fft_data", &self.fft_data)
            .field("time_data", &self.time_data)
            .field("fft_r2c", &format_args!("<FFT Planner>"))
            .field("fft_c2r", &format_args!("<IFFT Planner>"))
            .field("fft_size", &self.fft_size)
            .finish()
    }
}

impl AudioProcessor {
    /// Create a new `AudioProcessor` instance.
    pub fn new() -> Self {
        AudioProcessor {
            sample_rate: 0,
            channels: 0,
            fft_data: None,
            time_data: None,
            tmp_fft_data: None,
            fft_r2c: None,
            fft_c2r: None,
            fft_size: 0,
            max_amplitude: 1.,
        }
    }

    /// Get the current sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the number of channels.
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Get the FFT size.
    pub fn fft_size(&self) -> usize {
        self.fft_size
    }

    /// Check if time domain data is available.
    pub fn has_time_data(&self) -> bool {
        self.time_data.is_some()
    }

    /// Check if frequency domain data is available.
    pub fn has_frequency_data(&self) -> bool {
        self.fft_data.is_some()
    }

    /// Set audio parameters and time domain data.
    ///
    /// This method is used to load audio data from any source,
    /// allowing the processor to work with data from files,
    /// memory, network, or other sources.
    pub fn set_audio_data(
        &mut self,
        sample_rate: u32,
        channels: u16,
        time_data: Vec<Vec<f64>>,
    ) -> Result<()> {
        if time_data.is_empty() {
            return Err("No audio data provided".to_string());
        }

        if time_data[0].is_empty() {
            return Err("Audio data contains empty channels".to_string());
        }

        // Ensure all channels have the same length
        let first_channel_len = time_data[0].len();
        for (i, channel) in time_data.iter().enumerate().skip(1) {
            if channel.len() != first_channel_len {
                return Err(format!(
                    "Inconsistent channel lengths: channel 0 has {} samples, channel {} has {} samples",
                    first_channel_len, i, channel.len()
                ));
            }
        }

        self.sample_rate = sample_rate;
        self.channels = channels;
        self.time_data = Some(time_data);

        // Clear existing FFT data
        self.fft_data = None;

        Ok(())
    }

    /// Get a reference to the time domain data.
    pub fn time_data(&self) -> Option<&Vec<Vec<f64>>> {
        self.time_data.as_ref()
    }

    /// Get a reference to the FFT data
    pub fn fft_data(&self) -> Option<&Vec<Vec<Complex64>>> {
        self.fft_data.as_ref()
    }

    /// Perform the FFT on all audio channels.
    ///
    /// This transforms the time domain data into frequency domain data.
    pub fn perform_fft(&mut self) -> Result<()> {
        if let Some(time_data_channels) = &self.time_data {
            let mut planner = RealFftPlanner::<f64>::new();

            // Find next power of 2 for FFT size based on the first channel's length
            let mut fft_size = 1;
            while fft_size < time_data_channels[0].len() {
                fft_size *= 2;
            }
            self.fft_size = fft_size;

            // Create FFT objects for forward and inverse transforms
            let r2c = planner.plan_fft_forward(fft_size);
            let c2r = planner.plan_fft_inverse(fft_size);

            // Save the FFT objects for later use
            self.fft_r2c = Some(r2c);
            self.fft_c2r = Some(c2r);

            let num_channels = time_data_channels.len();
            let mut fft_data_channels = Vec::with_capacity(num_channels);

            self.max_amplitude = 0.0;

            // Process each channel independently
            for channel_idx in 0..num_channels {
                let channel_data = &time_data_channels[channel_idx];

                // Prepare input data for FFT (padding with zeros if necessary)
                let mut real_input = channel_data.clone();

                // Pad with zeros if necessary
                real_input.resize(fft_size, 0.0);

                // Create a buffer for the FFT output (real FFT output is only fft_size/2 + 1 complex values)
                let mut complex_output = vec![Complex64::new(0.0, 0.0); fft_size / 2 + 1];

                // Perform the FFT for this channel
                if let Some(r2c) = &self.fft_r2c {
                    r2c.process(&mut real_input, &mut complex_output)
                        .map_err(|e| format!("FFT error on channel {}: {}", channel_idx, e))?;
                } else {
                    return Err("FFT planner not initialized".to_string());
                }

                //Calculate max
                for c in &complex_output {
                    let (mut amplitude, _) = c.to_polar();
                    amplitude = amplitude.abs();
                    if amplitude > self.max_amplitude {
                        self.max_amplitude = amplitude;
                    }
                }
                // Store the FFT data for this channel
                fft_data_channels.push(complex_output);
            }

            // Normalize amplitudes based on max for all channels
            if self.max_amplitude > 0.0 {
                let scale = 1. / self.max_amplitude;
                for channel_idx in 0..num_channels {
                    let data = &mut fft_data_channels[channel_idx];
                    for i in 0..data.len() {
                        data[i] = data[i].scale(scale);
                    }
                }
            }

            // Store all channels' FFT data
            self.fft_data = Some(fft_data_channels);

            Ok(())
        } else {
            Err("No time data available for FFT".to_string())
        }
    }

    /// Prepare the FFT data for inverse transformation.
    ///
    /// This ensures the FFT data is properly formatted for a real FFT,
    /// particularly handling the DC and Nyquist components correctly.
    pub fn prepare_for_ifft(&mut self) -> Result<()> {
        if let Some(fft_data_channels) = &mut self.fft_data {
            // Process each channel independently
            for channel_data in fft_data_channels.iter_mut() {
                if !channel_data.is_empty() {
                    // Handle DC component (should be real)
                    channel_data[0] = Complex64::new(channel_data[0].re, 0.0);

                    // Handle Nyquist component (should be real)
                    if channel_data.len() > 1 {
                        let nyquist_idx = channel_data.len() - 1;
                        channel_data[nyquist_idx] =
                            Complex64::new(channel_data[nyquist_idx].re, 0.0);
                    }
                }
            }
        }
        Ok(())
    }

    /// Normalize time domain data to prevent clipping.
    ///
    /// This finds the peak amplitude across all channels and normalizes
    /// all samples accordingly, ensuring the output won't clip while
    /// maintaining the balance between channels.
    pub fn normalize_time_data(&mut self) -> Result<()> {
        if let Some(time_data_channels) = &mut self.time_data {
            // Only normalize if we have a non-zero maximum
            if self.max_amplitude > 0.0 {
                // Normalize all channels using the same scaling factor
                let scale_factor = self.max_amplitude / (self.fft_size as f64);

                for channel in time_data_channels.iter_mut() {
                    for sample in channel.iter_mut() {
                        *sample *= scale_factor;
                    }
                }

                println!("Normalized time data with scale factor: {}", scale_factor);
            }

            Ok(())
        } else {
            Err("No time domain data available for normalization".to_string())
        }
    }

    /// Perform the inverse FFT to convert frequency domain data back to time domain.
    pub fn perform_ifft(&mut self) -> Result<()> {
        // Prepare the data for a real IFFT
        self.prepare_for_ifft()?;

        if let Some(fft_data_channels) = &self.fft_data {
            let num_channels = fft_data_channels.len();
            let mut time_data_channels = Vec::with_capacity(num_channels);

            // Process each channel independently
            for channel_idx in 0..num_channels {
                let channel_fft_data = &fft_data_channels[channel_idx];

                // Clone the FFT data for inverse FFT
                let mut complex_input = channel_fft_data.clone();

                // Create a buffer for the time domain output
                let mut real_output = vec![0.0; self.fft_size];

                // Perform the inverse FFT for this channel
                if let Some(c2r) = &self.fft_c2r {
                    c2r.process(&mut complex_input, &mut real_output)
                        .map_err(|e| format!("IFFT error on channel {}: {}", channel_idx, e))?;
                } else {
                    return Err("IFFT planner not initialized".to_string());
                }

                // Store this channel's time domain data
                time_data_channels.push(real_output);
            }

            // Store all channels' time domain data
            self.time_data = Some(time_data_channels);

            // Normalize the time domain data to prevent clipping
            self.normalize_time_data()?;

            Ok(())
        } else {
            Err("No FFT data available for inverse FFT".to_string())
        }
    }

    /// Get information about the loaded audio data.
    ///
    /// Returns a structured collection of information about the current state.
    pub fn get_info(&self) -> AudioInfo {
        let time_data_info = self.time_data.as_ref().map(|data| TimeDataInfo {
            num_channels: data.len(),
            samples_per_channel: data[0].len(),
            duration_seconds: data[0].len() as f64 / self.sample_rate as f64,
        });

        let fft_data_info = self.fft_data.as_ref().map(|data| FFTDataInfo {
            num_channels: data.len(),
            complex_values_per_channel: data[0].len(),
            frequency_resolution: self.sample_rate as f64 / self.fft_size as f64,
        });

        AudioInfo {
            sample_rate: self.sample_rate,
            channels: self.channels,
            fft_size: self.fft_size,
            time_data: time_data_info,
            fft_data: fft_data_info,
        }
    }

    /// Utility function to normalize a phase value to the range [-π, π].
    pub fn normalize_phase(phase: f64) -> f64 {
        // Normalize the phase to the range [-π, π]
        // First get it to [0, 2π) with the modulo
        let normalized_phase = ((phase % (2.0 * std::f64::consts::PI))
            + (2.0 * std::f64::consts::PI))
            % (2.0 * std::f64::consts::PI);

        // Then shift values above π to the [-π, π] range
        if normalized_phase > std::f64::consts::PI {
            normalized_phase - 2.0 * std::f64::consts::PI
        } else {
            normalized_phase
        }
    }
}

/// Information about audio data loaded in the processor.
#[derive(Debug, Clone)]
pub struct AudioInfo {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of audio channels
    pub channels: u16,
    /// FFT size (power of 2)
    pub fft_size: usize,
    /// Information about time domain data (if available)
    pub time_data: Option<TimeDataInfo>,
    /// Information about frequency domain data (if available)
    pub fft_data: Option<FFTDataInfo>,
}

/// Information about time domain data.
#[derive(Debug, Clone)]
pub struct TimeDataInfo {
    /// Number of channels
    pub num_channels: usize,
    /// Number of samples per channel
    pub samples_per_channel: usize,
    /// Duration in seconds
    pub duration_seconds: f64,
}

/// Information about frequency domain data.
#[derive(Debug, Clone)]
pub struct FFTDataInfo {
    /// Number of channels
    pub num_channels: usize,
    /// Number of complex values per channel
    pub complex_values_per_channel: usize,
    /// Frequency resolution in Hz
    pub frequency_resolution: f64,
}
