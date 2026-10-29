//! Native window embedding helpers shared by adapters that host foreign windows.
#[cfg(target_os = "linux")]
pub mod x11;
