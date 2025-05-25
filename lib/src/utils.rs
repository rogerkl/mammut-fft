use crate::{AudioProcessor, Result};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::cell::RefCell;
use std::io::{Cursor, Seek, Write};
use std::rc::Rc;

// To get a hold on the buffer after calling WavWriter: use an Rc<RefCell<W>> to share ownership
pub struct SharedWriter<W> {
    writer: Rc<RefCell<Option<W>>>,
}

impl<W> SharedWriter<W> {
    pub fn new(writer: W) -> Self {
        SharedWriter {
            writer: Rc::new(RefCell::new(Some(writer))),
        }
    }

    pub fn clone_ref(&self) -> Self {
        SharedWriter {
            writer: Rc::clone(&self.writer),
        }
    }

    pub fn take_writer(self) -> Option<W> {
        self.writer.borrow_mut().take()
    }
}

impl<W: Write> Write for SharedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(writer) = &mut *self.writer.borrow_mut() {
            writer.write(buf)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Writer already taken",
            ))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(writer) = &mut *self.writer.borrow_mut() {
            writer.flush()
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Writer already taken",
            ))
        }
    }
}

impl<W: Seek> Seek for SharedWriter<W> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        if let Some(writer) = &mut *self.writer.borrow_mut() {
            writer.seek(pos)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Writer already taken",
            ))
        }
    }
}

/// Utility functions for file I/O and data conversion.
///
/// These functions are used by client applications but not by the core library.
/// They're included in the library for convenience.

/// Load audio data from a WAV file into an AudioProcessor.
///
/// This function handles reading the file, converting the samples to f64,
/// and setting up the AudioProcessor with the extracted data.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_from_wav(processor: &mut AudioProcessor, filename: &str) -> Result<()> {
    use crate::audio_read_write::read_audio_file;

    match read_audio_file(filename) {
        Ok((sample_rate, data)) => {
            // Set the audio data in the processor
            processor.set_audio_data(sample_rate, data.len().try_into().unwrap(), data)?;
            // Perform FFT on the loaded data
            processor.perform_fft()?;
            Ok(())
        }
        Err(error) => Err(format!("Error: {}", error)),
    }
}

/// Save audio data from an AudioProcessor to a Writer provided
///
/// This function handles converting the processor's time domain data
/// to an appropriate format and writing it to the Writer provided
///
fn save_to_wav_writer_provider<F, W, T>(
    processor: &mut AudioProcessor,
    provider: F,
    post_processor: impl FnOnce(W) -> T,
) -> Result<T>
where
    F: FnOnce() -> W,
    W: Write + Seek,
{
    if let Some(time_data_channels) = processor.time_data() {
        let spec = WavSpec {
            channels: processor.channels(),
            sample_rate: processor.sample_rate(),
            bits_per_sample: 32, // Using 32-bit float as the output format
            sample_format: SampleFormat::Float,
        };

        let buf_writer = provider();
        let shared_writer = SharedWriter::new(buf_writer);
        let writer_clone = shared_writer.clone_ref();

        let mut writer = WavWriter::new(shared_writer, spec)
            .map_err(|e| format!("Error creating output file: {}", e))?;

        // Interleave the channel data for writing
        let num_samples_per_channel = time_data_channels[0].len();
        for sample_idx in 0..num_samples_per_channel {
            for channel_idx in 0..time_data_channels.len() {
                // Convert f64 to f32 for writing
                writer
                    .write_sample(time_data_channels[channel_idx][sample_idx] as f32)
                    .map_err(|e| format!("Error writing sample: {}", e))?;
            }
        }

        writer
            .finalize()
            .map_err(|e| format!("Error finalizing output file: {}", e))?;

        let original_writer = writer_clone.take_writer().unwrap();

        Ok(post_processor(original_writer))
    } else {
        Err("No time domain data available for saving".to_string())
    }
}

/// Save audio data from an AudioProcessor to a WAV file.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_to_wav(processor: &mut AudioProcessor, filename: &str) -> Result<()> {
    use std::fs::File;
    use std::io::BufWriter;

    save_to_wav_writer_provider(
        processor,
        || {
            let file = File::create(filename).expect("Failed to create file");
            BufWriter::new(file)
        },
        |_| (), // Discard the writer
    )
}

pub fn save_to_wav_bytes(processor: &mut AudioProcessor) -> Result<Vec<u8>> {
    save_to_wav_writer_provider(
        processor,
        || Cursor::new(Vec::new()),
        |cursor| cursor.into_inner(), // Extract the Vec<u8>
    )
}

/// Create a format string for displaying a frequency value.
///
/// Returns a suitable string representation depending on the frequency range.
pub fn format_frequency(freq_hz: f64) -> String {
    if freq_hz >= 1000.0 {
        format!("{:.2} kHz", freq_hz / 1000.0)
    } else {
        format!("{:.2} Hz", freq_hz)
    }
}

/// Create a format string for displaying a time value.
///
/// Returns a suitable string representation depending on the time range.
pub fn format_time(time_sec: f64) -> String {
    if time_sec >= 60.0 {
        let minutes = (time_sec / 60.0).floor();
        let seconds = time_sec % 60.0;
        format!("{:.0}m {:.1}s", minutes, seconds)
    } else {
        format!("{:.2} sec", time_sec)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_from_wav_padded(
    processor: &mut AudioProcessor,
    filename: &str,
    buffer_multiplier: usize,
) -> Result<()> {
    use crate::audio_read_write::read_audio_file;

    match read_audio_file(filename) {
        Ok((sample_rate, data)) => {
            // Set the audio data in the processor with padding
            processor.set_buffer_multiplier(buffer_multiplier);
            processor.set_audio_data(sample_rate, data.len().try_into().unwrap(), data)?;

            // Perform FFT on the loaded data
            processor.perform_fft()?;
            Ok(())
        }
        Err(error) => Err(format!("Error: {}", error)),
    }
}
