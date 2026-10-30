use crate::execution_profile::{Interval, Profile, Snapshot};

#[derive(Clone, Copy, Debug)]
pub struct QueryFailure {
    pub stage: &'static str,
    pub os_code: Option<i32>,
}
pub trait Resources {
    fn snapshot(&mut self) -> Result<Snapshot, QueryFailure>;
    /// Wait at least one second and return actual monotonic elapsed milliseconds.
    fn wait_interval(&mut self) -> u128;
}
#[derive(Debug, Default)]
pub struct Admission {
    pub selected_mask: Option<usize>,
    pub intervals: Vec<Interval>,
    pub rejection: Option<&'static str>,
    pub query_failure: Option<QueryFailure>,
}
impl Admission {
    pub fn rejected(reason: &'static str) -> Self {
        Self {
            rejection: Some(reason),
            ..Self::default()
        }
    }
    pub fn query_failed(error: QueryFailure) -> Self {
        Self {
            query_failure: Some(error),
            ..Self::rejected("resource_query_failed")
        }
    }
}

/// No Job, child, process mutation or retry is permitted during admission.
pub fn admit(profile: Profile, permitted: usize, probe: &mut impl Resources) -> Admission {
    let Some(selected_mask) = profile.select(permitted) else {
        return Admission::rejected("insufficient_permitted_cpus");
    };
    let mut result = Admission {
        selected_mask: Some(selected_mask),
        ..Admission::default()
    };
    if profile == Profile::Conservative2 {
        return result;
    }
    let mut previous = match probe.snapshot() {
        Ok(value) => value,
        Err(error) => return Admission::query_failed(error),
    };
    for _ in 0..2 {
        let elapsed = probe.wait_interval();
        let current = match probe.snapshot() {
            Ok(value) => value,
            Err(error) => {
                result.query_failure = Some(error);
                result.rejection = Some("resource_query_failed");
                return result;
            }
        };
        let Some(interval) = Interval::between(previous, current, elapsed) else {
            result.rejection = Some("invalid_cpu_counters");
            return result;
        };
        result.intervals.push(interval);
        if let Some(reason) = interval.rejection() {
            result.rejection = Some(reason);
            return result;
        }
        previous = current;
    }
    result
}

pub fn prepare(profile: Option<Profile>) -> Admission {
    let Some(profile) = profile else {
        return Admission::rejected("unknown_profile");
    };
    let permitted = match profile {
        Profile::Conservative2 => {
            crate::windows_budget::Budget::permitted_mask().map_err(|error| QueryFailure {
                stage: "process_affinity",
                os_code: error.raw_os_error(),
            })
        }
        Profile::Build4 => crate::windows_topology::scope()
            .map(|scope| scope.allowed_mask)
            .map_err(|error| QueryFailure {
                stage: error.stage,
                os_code: error.os_code,
            }),
    };
    match permitted {
        Ok(mask) => admit(
            profile,
            mask,
            &mut crate::windows_resources::WindowsResources,
        ),
        Err(error) => Admission::query_failed(error),
    }
}

#[cfg(test)]
#[path = "execution_admission/tests.rs"]
mod tests;
