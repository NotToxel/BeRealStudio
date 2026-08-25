use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    codecs::DecoderOptions,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioAnalysis {
    pub duration: f64,
    pub sample_rate: u32,
    pub channels: u32,
    pub waveform: Vec<f32>,
}

/// Load an audio file, analyze duration, and compute a normalized peak amplitude waveform.
pub fn analyze_audio(path: &Path, buckets: usize) -> Result<AudioAnalysis> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Cannot open audio file: {}", path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .context("Unsupported audio format. Supported: MP3, WAV, M4A, AAC, FLAC, OGG")?;

    let mut format = probed.format;
    let track = format.tracks().first().context("No audio tracks found")?.clone();
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count() as u32).unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .context("Could not create audio decoder")?;

    let bucket_count = buckets.max(30).min(500);
    let mut raw_samples: Vec<f32> = Vec::new();
    let mut total_frames = 0u64;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(e.into()),
        };

        if packet.track_id() != track.id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(buf_ref) => {
                total_frames += buf_ref.frames() as u64;

                // Extract and downsample channel 0 samples for waveform
                match buf_ref {
                    AudioBufferRef::F32(buf) => {
                        for &s in buf.chan(0).iter().step_by(16) {
                            raw_samples.push(s.abs());
                        }
                    }
                    AudioBufferRef::S16(buf) => {
                        for &s in buf.chan(0).iter().step_by(16) {
                            raw_samples.push((s as f32 / 32768.0).abs());
                        }
                    }
                    AudioBufferRef::S32(buf) => {
                        for &s in buf.chan(0).iter().step_by(16) {
                            raw_samples.push((s as f32 / 2147483648.0).abs());
                        }
                    }
                    AudioBufferRef::U8(buf) => {
                        for &s in buf.chan(0).iter().step_by(16) {
                            raw_samples.push(((s as f32 - 128.0) / 128.0).abs());
                        }
                    }
                    _ => {}
                }
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        }
    }

    let duration = total_frames as f64 / sample_rate as f64;

    // Resample raw samples into fixed number of normalized peak buckets
    let mut waveform = vec![0.05f32; bucket_count];
    if !raw_samples.is_empty() {
        let chunk_size = (raw_samples.len() as f32 / bucket_count as f32).max(1.0);
        let mut max_peak = 0.001f32;

        for (i, slot) in waveform.iter_mut().enumerate() {
            let start_idx = (i as f32 * chunk_size) as usize;
            let end_idx = (((i + 1) as f32 * chunk_size) as usize).min(raw_samples.len());

            if start_idx < raw_samples.len() {
                let slice = &raw_samples[start_idx..end_idx.max(start_idx + 1)];
                let peak = slice.iter().copied().fold(0.0f32, f32::max);
                *slot = peak;
                if peak > max_peak {
                    max_peak = peak;
                }
            }
        }

        // Normalize peaks 0.05 to 1.0 for aesthetic display
        for v in waveform.iter_mut() {
            *v = ((*v / max_peak).max(0.04) * 0.95 + 0.05).min(1.0);
        }
    }

    Ok(AudioAnalysis {
        duration,
        sample_rate,
        channels,
        waveform,
    })
}

/// Load an audio file and return its duration in seconds.
/// Supports: MP3, WAV, M4A, AAC, FLAC, OGG via Symphonia.
pub fn get_audio_duration(path: &Path) -> Result<f64> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Cannot open audio file: {}", path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .context("Unsupported audio format. Supported: MP3, WAV, M4A, AAC, FLAC, OGG")?;

    let format = probed.format;

    // Try to get duration from track metadata first (faster)
    if let Some(track) = format.tracks().first() {
        let params = &track.codec_params;
        if let (Some(n_frames), Some(sample_rate)) = (params.n_frames, params.sample_rate) {
            if sample_rate > 0 {
                return Ok(n_frames as f64 / sample_rate as f64);
            }
        }
        // Try time base
        if let Some(tb) = params.time_base {
            if let Some(dur) = track.codec_params.n_frames {
                let secs = dur as f64 * tb.numer as f64 / tb.denom as f64;
                if secs > 0.0 {
                    return Ok(secs);
                }
            }
        }
    }

    // Fallback: decode entire file to count frames
    decode_for_duration(format)
}

fn decode_for_duration(
    mut format: Box<dyn symphonia::core::formats::FormatReader>,
) -> Result<f64> {
    let track = format.tracks().first().context("No audio tracks found")?.clone();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .context("Could not create audio decoder")?;

    let mut total_frames = 0u64;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(e.into()),
        };

        if packet.track_id() != track.id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                total_frames += decoded.frames() as u64;
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(total_frames as f64 / sample_rate as f64)
}
