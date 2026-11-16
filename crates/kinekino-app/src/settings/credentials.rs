//! Credential store selection for the composition root.
//!
//! The OS store comes first. When it is `Unavailable` the in-process session store is used
//! instead, and the choice is reported as [`CredentialBacking::SessionFallback`] with the
//! reason, so a secret that will not outlive the process is never presented as saved.
//! `KINEKINO_CREDENTIALS=os|session` pins the choice: `os` refuses the fallback and
//! `session` never touches the OS store (headless runs and tests).
use crate::Env;
use kinekino_core::port::credential::CredentialStore;
use kinekino_core::port::{ErrorCode, PortError, PortResult};

const MODE_VAR: &str = "KINEKINO_CREDENTIALS";

/// Which store answers credential calls, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialBacking {
    /// The OS credential store.
    Os,
    /// The session store, chosen with `KINEKINO_CREDENTIALS=session`.
    Session,
    /// The session store, because the OS store was unavailable.
    SessionFallback { reason: String },
}

impl CredentialBacking {
    /// Stable report name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Os => "os",
            Self::Session => "session",
            Self::SessionFallback { .. } => "session-fallback",
        }
    }

    /// Whether secrets written now are kept after the process exits.
    pub fn persistent(&self) -> bool {
        matches!(self, Self::Os)
    }
}

/// A selected store with its backing.
pub struct SelectedStore {
    pub store: Box<dyn CredentialStore>,
    pub backing: CredentialBacking,
}

enum Mode {
    Auto,
    Os,
    Session,
}

fn mode(env: &Env) -> PortResult<Mode> {
    match env.get(MODE_VAR) {
        None => Ok(Mode::Auto),
        Some("os") => Ok(Mode::Os),
        Some("session") => Ok(Mode::Session),
        Some(other) => Err(PortError::new(
            ErrorCode::InvalidInput,
            format!("{MODE_VAR}: unknown store `{other}` (expected os or session)"),
        )),
    }
}

fn session() -> Box<dyn CredentialStore> {
    Box::new(kinekino_platform::session::SessionCredentialStore::new())
}

/// Selects a store; `open_os` is called at most once and only when the mode allows it.
pub fn select(
    env: &Env,
    open_os: impl FnOnce() -> PortResult<Box<dyn CredentialStore>>,
) -> PortResult<SelectedStore> {
    let os = |store| SelectedStore {
        store,
        backing: CredentialBacking::Os,
    };
    match mode(env)? {
        Mode::Session => Ok(SelectedStore {
            store: session(),
            backing: CredentialBacking::Session,
        }),
        Mode::Os => open_os().map(os),
        Mode::Auto => match open_os() {
            Ok(store) => Ok(os(store)),
            Err(e) if e.code == ErrorCode::Unavailable => Ok(SelectedStore {
                store: session(),
                backing: CredentialBacking::SessionFallback { reason: e.message },
            }),
            Err(e) => Err(e),
        },
    }
}

/// [`select`] with this build's OS store.
pub fn open(env: &Env) -> PortResult<SelectedStore> {
    select(env, kinekino_platform::credential::open_os_store)
}
