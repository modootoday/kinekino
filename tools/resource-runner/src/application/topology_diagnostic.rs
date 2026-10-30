use crate::{cpu_selection, topology_report, windows_topology};
use std::io;

pub fn run() -> io::Result<i32> {
    let scope = windows_topology::scope();
    let cores = windows_topology::cores();
    let legacy = scope.as_ref().ok().map(|scope| {
        cpu_selection::compare(scope.process_mask, scope.group, cores.as_deref().ok())
    });
    let candidate = scope.as_ref().ok().map(|scope| {
        cpu_selection::compare(scope.allowed_mask, scope.group, cores.as_deref().ok())
    });
    topology_report::write(
        &mut io::stdout().lock(),
        &scope,
        &cores,
        legacy.as_ref(),
        candidate.as_ref(),
    )?;
    Ok(0)
}
