//! Prepared property projection. Missing, disabled and unresolved options remain distinct.
use crate::domain::playback_options::{ObservedTrackChoice, TrackId};
use serde_json::Value;

pub(super) fn rate(value: &Value) -> Option<f64> {
    value.as_f64().filter(|rate| rate.is_finite() && *rate > 0.)
}
pub(super) fn choice(value: &Value) -> Option<ObservedTrackChoice> {
    match value {
        Value::Bool(false) => Some(ObservedTrackChoice::Disabled),
        Value::String(text) if text == "no" => Some(ObservedTrackChoice::Disabled),
        Value::String(text) if text == "auto" => Some(ObservedTrackChoice::Unresolved),
        Value::Number(number) => Some(ObservedTrackChoice::Selected {
            track_id: TrackId::new(number.as_u64()?).ok()?,
        }),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn disabled_is_observed_and_auto_is_not_a_selected_track() {
        assert_eq!(choice(&Value::Null), None);
        assert_eq!(choice(&json!(false)), Some(ObservedTrackChoice::Disabled));
        assert_eq!(choice(&json!("no")), Some(ObservedTrackChoice::Disabled));
        assert_eq!(
            choice(&json!("auto")),
            Some(ObservedTrackChoice::Unresolved)
        );
        assert_eq!(
            choice(&json!(2)),
            Some(ObservedTrackChoice::Selected {
                track_id: TrackId::new(2).unwrap()
            })
        );
        for value in [
            json!(0),
            json!(-1),
            json!(true),
            json!("untrusted"),
            json!(4294967296u64),
        ] {
            assert_eq!(choice(&value), None);
        }
    }
    #[test]
    fn engine_rate_observation_does_not_clamp_to_requested_policy_bounds() {
        assert_eq!(rate(&json!(1.25)), Some(1.25));
        assert_eq!(rate(&json!(8.)), Some(8.));
        assert_eq!(rate(&json!(0.125)), Some(0.125));
        for value in [json!(0), json!(-1), json!("1.0"), Value::Null] {
            assert_eq!(rate(&value), None);
        }
    }
}
