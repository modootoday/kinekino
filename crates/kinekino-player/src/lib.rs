// Linux build: Windows-only adapters are cfg-gated, so their callees look unused here. Removed once kinekino-platform split lands.
#![cfg_attr(
    not(windows),
    allow(dead_code, unused_imports, unused_variables, unreachable_code)
)]
#[cfg(any(test, all(windows, feature = "native-proposal-window")))]
extern crate self as kinekino_player;

pub mod application;
pub mod assembly;
pub mod domain;
pub mod infrastructure;
#[cfg(windows)]
pub mod presentation;
#[cfg(test)]
mod test_support;

/// Unit tests use the fixture location layout that a source plugin would supply.
#[cfg(test)]
#[ctor::ctor(unsafe)]
fn install_fixture_rules() {
    kinekino_source_host::plugin_host::install_fixture_rules();
}
