use anyhow::{Context, Result};
use std::path::Path;
use symphonia::core::{
    codecs::DecoderOptions,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

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
