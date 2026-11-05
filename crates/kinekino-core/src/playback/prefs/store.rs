//! Port for loading and saving [`PlayerPrefs`] (M1-7). The JSON file adapter lives in `file`.
use super::value::PlayerPrefs;
use crate::port::error::PortResult;

/// One prefs document per store. Adapters start from defaults when nothing is saved.
pub trait PlayerPrefsStore {
    fn load(&self) -> PortResult<PlayerPrefs>;
    fn save(&mut self, prefs: &PlayerPrefs) -> PortResult<()>;
}

/// In-memory prefs for tests; starts at [`PlayerPrefs::default`].
#[derive(Debug, Clone, Default)]
pub struct MemoryPlayerPrefsStore(PlayerPrefs);

impl MemoryPlayerPrefsStore {
    pub fn with(prefs: PlayerPrefs) -> Self {
        Self(prefs.clamped())
    }

    pub fn current(&self) -> PlayerPrefs {
        self.0
    }
}

impl PlayerPrefsStore for MemoryPlayerPrefsStore {
    fn load(&self) -> PortResult<PlayerPrefs> {
        Ok(self.0)
    }
    fn save(&mut self, prefs: &PlayerPrefs) -> PortResult<()> {
        self.0 = prefs.checked()?;
        Ok(())
    }
}

/// Shared contract for [`PlayerPrefsStore`] adapters.
pub fn check_player_prefs_store(store: &mut dyn PlayerPrefsStore) -> PortResult<()> {
    use crate::port::error::{ErrorCode, PortError};
    let fail = |m: &str| PortError::new(ErrorCode::Internal, format!("contract: {m}"));
    let first = store.load()?;
    let changed = PlayerPrefs {
        volume: 42.0,
        muted: true,
        speed: 1.25,
        auto_next: true,
    };
    store.save(&changed)?;
    let loaded = store.load()?;
    if loaded != changed.clamped() {
        return Err(fail("save then load must round-trip"));
    }
    store.save(&first)?;
    if store.load()? != first.clamped() {
        return Err(fail("save must overwrite the whole document"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_meets_contract() {
        check_player_prefs_store(&mut MemoryPlayerPrefsStore::default()).unwrap();
    }
}
