//! FFT Audio Processor Library
//!
//! A library for processing audio data using Fast Fourier Transform (FFT).
//! Provides functionality for loading, manipulating, and saving audio data
//! in both time and frequency domains.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

mod processor;
pub mod utils;

pub use num_complex::Complex64;
pub use processor::AudioProcessor;

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the library
///
/// This is a placeholder for future initialization code.
/// For WASM targets, this will set up any necessary browser bindings.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn init() {
    #[cfg(feature = "wasm")]
    {
        // Set up console error panic hook for better error messages in WebAssembly
        #[cfg(feature = "wasm")]
        console_error_panic_hook::set_once();
    }
}

/// Result type for audio processing operations
pub type Result<T> = std::result::Result<T, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
        // Just checking that it doesn't panic
        assert!(true);
    }
}
