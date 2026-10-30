mod annotation;
mod policy;
pub use annotation::{ManualSegment, SegmentKind, SegmentOrigin, SegmentTimebase};
pub use policy::{
    BlockReason, Decision, Evaluation, Intent, PassState, Seekability, SkipMode, SkipPolicy,
    evaluate,
};
#[cfg(test)]
mod tests;
