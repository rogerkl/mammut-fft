use std::error::Error;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
use std::io::Cursor;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::sample::{i24, u24};

/// Reads an audio file and returns a vector of f64 vectors, where each inner vector represents a channel
/// and contains all the samples for that channel (non-interleaved format).
pub fn read_audio(mss: MediaSourceStream) -> Result<(u32, Vec<Vec<f64>>), Box<dyn Error>> {
    // Create a hint to help the format registry guess what format the file is
    let hint = Hint::new();

    // Get the format reader
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed =
        symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

    // Get the default track
    let track = probed
        .format
        .default_track()
        .ok_or("No default track found")?;

    // Create a decoder for the track
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .map_err(|_| "Unsupported codec")?;

    // Get the track's sample rate
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or("Sample rate not specified")?;

    // Get the number of channels
    let channels = track
        .codec_params
        .channels
        .ok_or("Channels not specified")?
        .count();

    // Create a buffer for each channel
    let mut channel_buffers: Vec<Vec<f64>> = vec![Vec::new(); channels];

    // Decode frames one by one
    let mut format = probed.format;
    loop {
        // Get the next packet from the format reader
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(err)) => {
                if err.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                return Err(Box::new(err));
            }
            Err(err) => return Err(Box::new(err)),
        };

        // Decode the packet
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(symphonia::core::errors::Error::IoError(err)) => {
                if err.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                return Err(Box::new(err));
            }
            Err(err) => return Err(Box::new(err)),
        };

        // Process the decoded audio buffer
        match decoded {
            AudioBufferRef::F32(buffer) => {
                for c in 0..channels {
                    let samples = buffer.chan(c);
                    for &sample in samples {
                        channel_buffers[c].push(sample as f64);
                    }
                }
            }
            AudioBufferRef::F64(buffer) => {
                for c in 0..channels {
                    let samples = buffer.chan(c);
                    for &sample in samples {
                        channel_buffers[c].push(sample);
                    }
                }
            }
            AudioBufferRef::U32(buffer) => {
                for c in 0..channels {
                    let samples = buffer.chan(c);
                    for &sample in samples {
                        // Convert to normalized f64 in the range [-1.0, 1.0]
                        channel_buffers[c].push((sample as f64 / u32::MAX as f64) * 2.0 - 1.0);
                    }
                }
            }
            AudioBufferRef::U24(buffer) => {
                for c in 0..channels {
                    let samples = buffer.chan(c);
                    for &sample in samples {
                        // Convert to normalized f64 in the range [-1.0, 1.0]
                        channel_buffers[c]
                            .push((sample.inner() as f64 / u24::MAX.inner() as f64) * 2.0 - 1.0);
                    }
                }
            }
            AudioBufferRef::U16(buffer) => {
                for c in 0..channels {
                    let samples = buffer.chan(c);
                    for &sample in samples {
                        // Convert to normalized f64 in the range [-1.0, 1.0]
                        channel_buffers[c].push((sample as f64 / u16::MAX as f64) * 2.0 - 1.0);
                    }
                }
            }
            AudioBufferRef::U8(buffer) => {
                for c in 0..channels {
                    let samples = buffer.chan(c);
                    for &sample in samples {
                        // Convert to normalized f64 in the range [-1.0, 1.0]
                        channel_buffers[c].push((sample as f64 / u8::MAX as f64) * 2.0 - 1.0);
                    }
                }
            }
            AudioBufferRef::S32(buffer) => {
                for c in 0..channels {
                    let samples = buffer.chan(c);
                    for &sample in samples {
                        channel_buffers[c].push(sample as f64 / i32::MAX as f64);
                    }
                }
            }
            AudioBufferRef::S24(buffer) => {
                for c in 0..channels {
                    let samples = buffer.chan(c);
                    for &sample in samples {
                        channel_buffers[c].push(sample.inner() as f64 / i24::MAX.inner() as f64);
                    }
                }
            }
            AudioBufferRef::S16(buffer) => {
                for c in 0..channels {
                    let samples = buffer.chan(c);
                    for &sample in samples {
                        channel_buffers[c].push(sample as f64 / i16::MAX as f64);
                    }
                }
            }
            _ => return Err("Unsupported audio format".into()),
        }
    }

    Ok((sample_rate, channel_buffers))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_audio_file<P: AsRef<Path>>(path: P) -> Result<(u32, Vec<Vec<f64>>), Box<dyn Error>> {
    // Open the file - File implements MediaSource directly
    let file = File::open(path)?;

    // Create a MediaSourceStream directly from the file
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Continue with the decoding process
    read_audio(mss)
}

// Function to handle byte data (e.g., from memory)
pub fn read_audio_bytes(data: Vec<u8>) -> Result<(u32, Vec<Vec<f64>>), Box<dyn Error>> {
    // Create a cursor over the bytes - Cursor<Vec<u8>> implements MediaSource
    let cursor = Cursor::new(data);

    // Create a MediaSourceStream
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    // Continue with the decoding process
    read_audio(mss)
}
