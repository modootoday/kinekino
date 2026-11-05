//! Resume and skip-range storage: ports, in-memory fakes, and shared adapter contracts.
//! JSON file adapters live in `resume_file` and `skip_range_file`.
use super::resume::ResumePoint;
use super::skip_range::SkipRange;
use crate::port::error::{ErrorCode, PortError, PortResult};
use std::collections::BTreeMap;

/// Keyed by a stable media key (e.g. a canonical local path or a playable item id).
pub trait ResumeStore {
    fn get(&self, key: &str) -> PortResult<Option<ResumePoint>>;
    fn put(&mut self, key: &str, point: ResumePoint) -> PortResult<()>;
    /// Returns whether an entry existed.
    fn clear(&mut self, key: &str) -> PortResult<bool>;
}

#[derive(Debug, Default)]
pub struct MemoryResumeStore(BTreeMap<String, ResumePoint>);

impl ResumeStore for MemoryResumeStore {
    fn get(&self, key: &str) -> PortResult<Option<ResumePoint>> {
        Ok(self.0.get(key).copied())
    }
    fn put(&mut self, key: &str, point: ResumePoint) -> PortResult<()> {
        self.0.insert(key.into(), point);
        Ok(())
    }
    fn clear(&mut self, key: &str) -> PortResult<bool> {
        Ok(self.0.remove(key).is_some())
    }
}

/// Shared contract for [`ResumeStore`] adapters; `store` must start empty.
pub fn check_resume_store(store: &mut dyn ResumeStore) -> PortResult<()> {
    let fail = |m: &str| PortError::new(ErrorCode::Internal, format!("contract: {m}"));
    let p = ResumePoint {
        position_us: 42_000_000,
        duration_us: 1_200_000_000,
    };
    if store.get("a")?.is_some() || store.clear("a")? {
        return Err(fail("store must start empty"));
    }
    store.put("a", p)?;
    store.put(
        "b",
        ResumePoint {
            position_us: 1,
            ..p
        },
    )?;
    store.put(
        "a",
        ResumePoint {
            position_us: 50_000_000,
            ..p
        },
    )?;
    if store.get("a")?.map(|x| x.position_us) != Some(50_000_000) || store.get("b")?.is_none() {
        return Err(fail("put must upsert per key"));
    }
    if !store.clear("a")? || store.get("a")?.is_some() || store.get("b")?.is_none() {
        return Err(fail("clear must remove only its key"));
    }
    Ok(())
}

/// Keyed like [`ResumeStore`]; values are the span the user marked for that item.
pub trait SkipRangeStore {
    fn get(&self, key: &str) -> PortResult<Option<SkipRange>>;
    fn put(&mut self, key: &str, range: SkipRange) -> PortResult<()>;
    /// Returns whether an entry existed.
    fn clear(&mut self, key: &str) -> PortResult<bool>;
}

#[derive(Debug, Default)]
pub struct MemorySkipRangeStore(BTreeMap<String, SkipRange>);

impl SkipRangeStore for MemorySkipRangeStore {
    fn get(&self, key: &str) -> PortResult<Option<SkipRange>> {
        Ok(self.0.get(key).copied())
    }
    fn put(&mut self, key: &str, range: SkipRange) -> PortResult<()> {
        self.0.insert(key.into(), range);
        Ok(())
    }
    fn clear(&mut self, key: &str) -> PortResult<bool> {
        Ok(self.0.remove(key).is_some())
    }
}

/// Shared contract for [`SkipRangeStore`] adapters; `store` must start empty.
pub fn check_skip_range_store(store: &mut dyn SkipRangeStore) -> PortResult<()> {
    let fail = |m: &str| PortError::new(ErrorCode::Internal, format!("contract: {m}"));
    let a = SkipRange::new(10.0, 90.0).expect("valid");
    let b = SkipRange::new(5.0, 15.0).expect("valid");
    if store.get("a")?.is_some() || store.clear("a")? {
        return Err(fail("store must start empty"));
    }
    store.put("a", a)?;
    store.put("b", b)?;
    let a2 = SkipRange::new(20.0, 40.0).expect("valid");
    store.put("a", a2)?;
    if store.get("a")? != Some(a2) || store.get("b")? != Some(b) {
        return Err(fail("put must upsert per key"));
    }
    if !store.clear("a")? || store.get("a")?.is_some() || store.get("b")?.is_none() {
        return Err(fail("clear must remove only its key"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_meets_contract() {
        check_resume_store(&mut MemoryResumeStore::default()).unwrap();
    }

    #[test]
    fn memory_skip_range_store_meets_contract() {
        check_skip_range_store(&mut MemorySkipRangeStore::default()).unwrap();
    }
}
