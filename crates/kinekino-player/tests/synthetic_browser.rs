//! OS-neutral run of the synthetic browser contract (site-neutral fixture rules). The same
//! module is also mounted by the Windows `jobs` suite, which drives it through real job hosts.
#[path = "jobs/synthetic_browser.rs"]
mod synthetic_browser;
