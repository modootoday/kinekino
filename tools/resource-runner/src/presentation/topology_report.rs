use crate::{
    cpu_selection::{Core, Selection},
    windows_topology::{Failure, ProcessScope},
};
use std::io::{self, Write};

fn failure(out: &mut impl Write, error: Option<&Failure>) -> io::Result<()> {
    if let Some(error) = error {
        // Stage is an adapter-owned ASCII identifier, never an external error string.
        write!(out, "{{\"stage\":\"{}\",\"os_code\":", error.stage)?;
        if let Some(code) = error.os_code {
            write!(out, "{code}")?;
        } else {
            write!(out, "null")?;
        }
        write!(out, "}}")
    } else {
        write!(out, "null")
    }
}

pub fn write(
    out: &mut impl Write,
    scope: &Result<ProcessScope, Failure>,
    cores: &Result<Vec<Core>, Failure>,
    legacy: Option<&Selection>,
    candidate: Option<&Selection>,
) -> io::Result<()> {
    write!(
        out,
        "{{\"schema\":\"resource-topology-diagnostic/1\",\"read_only\":true,\"default_policy\":\"first_two\",\"policy_changed\":false,\"candidate_applied\":false,\"scope\":"
    )?;
    if let Ok(scope) = scope {
        write!(
            out,
            "{{\"group\":{},\"process_mask\":{},\"system_mask\":{},\"thread_mask\":{},\"allowed_mask\":{}}}",
            scope.group,
            scope.process_mask,
            scope.system_mask,
            scope.thread_mask,
            scope.allowed_mask
        )?;
    } else {
        write!(out, "null")?;
    }
    write!(out, ",\"scope_error\":")?;
    failure(out, scope.as_ref().err())?;
    write!(out, ",\"core_records\":")?;
    if let Ok(cores) = cores {
        write!(out, "[")?;
        for (index, core) in cores.iter().enumerate() {
            if index != 0 {
                write!(out, ",")?;
            }
            write!(
                out,
                "{{\"group\":{},\"mask\":{},\"smt\":{},\"efficiency_class\":{}}}",
                core.group, core.mask, core.smt, core.efficiency_class
            )?;
        }
        write!(out, "]")?;
    } else {
        write!(out, "null")?;
    }
    write!(out, ",\"topology_error\":")?;
    failure(out, cores.as_ref().err())?;
    write!(out, ",\"comparison\":")?;
    if let (Some(legacy), Some(selection)) = (legacy, candidate) {
        write!(
            out,
            "{{\"default_mask\":{},\"default_topology_reason\":\"{}\",\"default_pair_same_core\":",
            legacy.default_mask,
            legacy.reason.label()
        )?;
        if let Some(same) = legacy.default_pair_same_core {
            write!(out, "{same}")?;
        } else {
            write!(out, "null")?;
        }
        write!(
            out,
            ",\"candidate_scope_first_two_mask\":{},\"candidate_mask\":{},\"candidate_reason\":\"{}\"",
            selection.default_mask,
            selection.candidate_mask,
            selection.reason.label()
        )?;
        write!(out, "}}")?;
    } else {
        write!(out, "null")?;
    }
    writeln!(out, ",\"performance_gain_measured\":false}}")
}
