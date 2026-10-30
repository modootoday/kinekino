use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Run {
    pub wall_seconds: f64,
    pub media_seconds: f64,
    pub samples: usize,
}

pub fn eligible_media(time: f64, duration: f64, target: f64) -> bool {
    time.is_finite()
        && duration.is_finite()
        && target.is_finite()
        && target > 0.
        && time > 0.
        && duration >= target
        && time <= duration + 1.
}

pub fn longest(points: impl Iterator<Item = Option<(f64, f64)>>) -> Run {
    let mut best = Run::default();
    let mut start = None;
    let mut previous: Option<(f64, f64)> = None;
    let mut stall = None;
    let mut count = 0;
    for point in points {
        let Some(point) = point else {
            start = None;
            previous = None;
            stall = None;
            count = 0;
            continue;
        };
        if let Some(prev) = previous {
            let dt = point.0 - prev.0;
            let advance = point.1 - prev.1;
            stall = if advance < 0.05 {
                Some(stall.unwrap_or(prev.0))
            } else {
                None
            };
            if dt <= 0.0
                || dt > 5.0
                || advance < -0.25
                || advance > dt * 1.25 + 0.5
                || stall.is_some_and(|s| point.0 - s > 10.0)
            {
                start = None;
                count = 0;
                stall = None;
            }
        }
        let first = *start.get_or_insert(point);
        previous = Some(point);
        count += 1;
        let wall = point.0 - first.0;
        let media = point.1 - first.1;
        if wall.min(media) > best.wall_seconds.min(best.media_seconds) {
            best = Run {
                wall_seconds: wall,
                media_seconds: media,
                samples: count,
            };
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_episodes_without_a_movie_specific_duration_gate() {
        assert!(eligible_media(30., 1420., 20.));
        assert!(eligible_media(3601., 7206., 3600.));
        assert!(!eligible_media(1., 5., 20.));
        assert!(!eligible_media(2000., 1420., 20.));
        assert!(!eligible_media(1., 1420., f64::NAN));
    }

    #[test]
    fn counts_real_progress_and_rejects_seek_stall_and_missing_samples() {
        let run = longest((0..61).map(|i| Some((i as f64, i as f64 + 1.0))));
        assert_eq!(run.media_seconds, 60.0);
        let seek = longest([Some((0.0, 1.0)), Some((1.0, 3601.0))].into_iter());
        assert_eq!(seek.media_seconds, 0.0);
        let frozen = longest((0..61).map(|i| Some((i as f64, 1.0))));
        assert_eq!(frozen.media_seconds, 0.0);
        let gap = longest([Some((0.0, 1.0)), Some((10.0, 11.0))].into_iter());
        assert_eq!(gap.media_seconds, 0.0);
    }
}
