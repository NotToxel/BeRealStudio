use crate::pipeline::types::SpeedMode;

/// Calculate per-image display durations for the Recapper slideshow.
///
/// - `total_duration`: audio duration in seconds
/// - `count`: number of images
/// - `start_padding`: extra time for first image
/// - `end_padding`: extra time for last image
/// - `speed_mode`: Ramp, Even, Accelerate, Decelerate, Wave
pub fn calculate_durations(
    total_duration: f64,
    count: usize,
    start_padding: f64,
    end_padding: f64,
    speed_mode: &SpeedMode,
) -> Vec<f64> {
    if count == 0 || !total_duration.is_finite() || total_duration <= 0.0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![total_duration];
    }

    // Sanitize paddings: cannot exceed 80% of total_duration combined to prevent desync
    let max_combined_padding = total_duration * 0.8;
    let raw_start = start_padding.max(0.0);
    let raw_end = end_padding.max(0.0);
    let (clean_start_pad, clean_end_pad) = if raw_start + raw_end > max_combined_padding {
        let ratio = max_combined_padding / (raw_start + raw_end).max(0.001);
        (raw_start * ratio, raw_end * ratio)
    } else {
        (raw_start, raw_end)
    };

    let active = (total_duration - clean_start_pad - clean_end_pad).max(0.001);

    let mut durations: Vec<f64> = match speed_mode {
        SpeedMode::Ramp => {
            // Quadratic ramp: w = 1 + 2x² where x ∈ [-1, 1] (slow start & end, fast middle)
            let weights: Vec<f64> = (0..count)
                .map(|i| {
                    let x = -1.0 + 2.0 * i as f64 / (count - 1) as f64;
                    1.0 + 2.0 * x * x
                })
                .collect();
            let weight_sum: f64 = weights.iter().sum::<f64>().max(0.0001);
            weights.iter().map(|w| w / weight_sum * active).collect()
        }
        SpeedMode::Even => vec![active / count as f64; count],
        SpeedMode::Accelerate => {
            // Accelerate: starts slow and speeds up towards the finale
            let weights: Vec<f64> = (0..count)
                .map(|i| {
                    let t = i as f64 / (count - 1) as f64;
                    1.0 + 2.5 * (1.0 - t) * (1.0 - t)
                })
                .collect();
            let weight_sum: f64 = weights.iter().sum::<f64>().max(0.0001);
            weights.iter().map(|w| w / weight_sum * active).collect()
        }
        SpeedMode::Decelerate => {
            // Decelerate: starts fast with high energy and slows down for an emotional finale
            let weights: Vec<f64> = (0..count)
                .map(|i| {
                    let t = i as f64 / (count - 1) as f64;
                    1.0 + 2.5 * t * t
                })
                .collect();
            let weight_sum: f64 = weights.iter().sum::<f64>().max(0.0001);
            weights.iter().map(|w| w / weight_sum * active).collect()
        }
        SpeedMode::Wave => {
            // Wave: oscillating speed rhythm
            let weights: Vec<f64> = (0..count)
                .map(|i| {
                    let t = i as f64 / (count - 1) as f64;
                    1.0 + (std::f64::consts::PI * 3.0 * t).sin().abs()
                })
                .collect();
            let weight_sum: f64 = weights.iter().sum::<f64>().max(0.0001);
            weights.iter().map(|w| w / weight_sum * active).collect()
        }
    };

    // Apply padding to first and last
    if let Some(first) = durations.first_mut() {
        *first += clean_start_pad;
    }
    if let Some(last) = durations.last_mut() {
        *last += clean_end_pad;
    }

    durations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_even_sums_to_total() {
        let durations = calculate_durations(60.0, 10, 2.0, 3.0, &SpeedMode::Even);
        let sum: f64 = durations.iter().sum();
        assert!((sum - 60.0).abs() < 0.001, "sum={}", sum);
    }

    #[test]
    fn test_ramp_sums_to_total() {
        let durations = calculate_durations(120.0, 20, 2.0, 3.0, &SpeedMode::Ramp);
        let sum: f64 = durations.iter().sum();
        assert!((sum - 120.0).abs() < 0.001, "sum={}", sum);
    }

    #[test]
    fn test_accelerate_sums_to_total() {
        let durations = calculate_durations(60.0, 10, 1.0, 1.0, &SpeedMode::Accelerate);
        let sum: f64 = durations.iter().sum();
        assert!((sum - 60.0).abs() < 0.001, "sum={}", sum);
        assert!(durations[0] > durations[8]);
    }

    #[test]
    fn test_decelerate_sums_to_total() {
        let durations = calculate_durations(60.0, 10, 1.0, 1.0, &SpeedMode::Decelerate);
        let sum: f64 = durations.iter().sum();
        assert!((sum - 60.0).abs() < 0.001, "sum={}", sum);
        assert!(durations[8] > durations[1]);
    }

    #[test]
    fn test_wave_sums_to_total() {
        let durations = calculate_durations(60.0, 10, 1.0, 1.0, &SpeedMode::Wave);
        let sum: f64 = durations.iter().sum();
        assert!((sum - 60.0).abs() < 0.001, "sum={}", sum);
    }
}
