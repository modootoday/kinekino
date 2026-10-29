use super::MAX_SCORES;
use serde::{
    Deserialize, Deserializer,
    de::{self, SeqAccess, Visitor},
};
use std::fmt;

#[derive(Debug, Default)]
pub(super) struct Scores(pub Vec<f32>);
impl<'de> Deserialize<'de> for Scores {
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct ScoreVisitor;
        impl<'de> Visitor<'de> for ScoreVisitor {
            type Value = Scores;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bounded finite VAD scores")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Scores, A::Error> {
                let mut scores = Vec::new();
                scores
                    .try_reserve_exact(MAX_SCORES)
                    .map_err(|_| de::Error::custom("Score allocation failed"))?;
                loop {
                    if scores.len() == MAX_SCORES {
                        if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                            return Err(de::Error::custom("Too many scores"));
                        }
                        break;
                    }
                    let Some(score) = sequence.next_element::<f32>()? else {
                        break;
                    };
                    if !score.is_finite() || !(0.0..=1.0).contains(&score) {
                        return Err(de::Error::custom("Invalid/oversized scores"));
                    }
                    scores.push(score);
                }
                Ok(Scores(scores))
            }
        }
        decoder.deserialize_seq(ScoreVisitor)
    }
}
