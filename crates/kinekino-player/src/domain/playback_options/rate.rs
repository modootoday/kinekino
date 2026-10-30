use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PlaybackRate(f64);
impl PlaybackRate {
    pub const MIN: f64 = 0.25;
    pub const MAX: f64 = 4.0;
    pub fn new(value: f64) -> Result<Self, &'static str> {
        if !value.is_finite() || !(Self::MIN..=Self::MAX).contains(&value) {
            return Err("Playback rate must be finite and between 0.25 and 4.0");
        }
        Ok(Self(value))
    }
    pub fn get(self) -> f64 {
        self.0
    }
}
impl<'de> Deserialize<'de> for PlaybackRate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(f64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_and_json_rates_share_finite_bounds_without_clamping() {
        for value in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            0.,
            -1.,
            0.249,
            4.001,
        ] {
            assert!(PlaybackRate::new(value).is_err());
        }
        for value in [0.25, 1., 1.25, 4.] {
            let rate = PlaybackRate::new(value).unwrap();
            assert_eq!(rate.get(), value);
            assert_eq!(
                serde_json::from_value::<PlaybackRate>(serde_json::json!(value)).unwrap(),
                rate
            );
        }
        for value in [
            serde_json::json!("1.25"),
            serde_json::json!(0),
            serde_json::json!(5),
            serde_json::Value::Null,
        ] {
            assert!(serde_json::from_value::<PlaybackRate>(value).is_err());
        }
    }
}
