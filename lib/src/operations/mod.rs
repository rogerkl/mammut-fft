// lib/src/operations/mod.rs

//! Audio processing operations for the FFT Audio Processor.
//!
//! This module contains various operations that can be applied to audio data
//! in the frequency domain, organized by category for better maintainability.

pub mod amplitude;
pub mod bin_swap;
pub mod convolution; // New module for convolution operations
pub mod filters;
pub mod mixing;
pub mod peaks;
pub mod phase;
pub mod spectral;
pub mod split;
pub mod threshold;
pub mod wobble;
