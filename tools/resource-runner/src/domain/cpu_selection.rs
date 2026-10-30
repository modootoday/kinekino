#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Core {
    pub group: u16,
    pub mask: usize,
    pub smt: bool,
    pub efficiency_class: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    EmptyAllowed,
    QueryFailed,
    InvalidTopology,
    IncompleteGroup,
    SingleAllowed,
    SingleCore,
    DistinctCores,
}
impl Reason {
    pub fn label(self) -> &'static str {
        match self {
            Self::EmptyAllowed => "empty_allowed_mask",
            Self::QueryFailed => "topology_query_failed",
            Self::InvalidTopology => "invalid_topology",
            Self::IncompleteGroup => "incomplete_current_group",
            Self::SingleAllowed => "only_one_allowed_logical_cpu",
            Self::SingleCore => "only_one_allowed_physical_core",
            Self::DistinctCores => "different_core_comparison_candidate",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Selection {
    pub default_mask: usize,
    pub candidate_mask: usize,
    pub reason: Reason,
    /// None means the default pair's physical relationship was not established.
    pub default_pair_same_core: Option<bool>,
}

/// Existing runner policy, unchanged: lowest two allowed bits, without topology.
pub fn first_two(mask: usize) -> usize {
    let first = mask.isolate_lowest_one();
    let rest = mask & !first;
    first | rest.isolate_lowest_one()
}

pub fn first_n(mut mask: usize, count: u32) -> usize {
    let mut selected = 0;
    for _ in 0..count {
        let bit = mask.isolate_lowest_one();
        selected |= bit;
        mask &= !bit;
    }
    selected
}

/// Diagnostic comparison only. Never selects outside the current group/mask.
pub fn compare(allowed: usize, group: u16, cores: Option<&[Core]>) -> Selection {
    let default_mask = first_two(allowed);
    let fallback = |reason| Selection {
        default_mask,
        candidate_mask: default_mask,
        reason,
        default_pair_same_core: None,
    };
    if allowed == 0 {
        return fallback(Reason::EmptyAllowed);
    }
    let Some(cores) = cores else {
        return fallback(Reason::QueryFailed);
    };
    let mut covered = 0;
    let mut eligible = Vec::new();
    for core in cores.iter().filter(|c| c.group == group) {
        if core.mask == 0 || covered & core.mask != 0 {
            return fallback(Reason::InvalidTopology);
        }
        covered |= core.mask;
        if core.mask & allowed != 0 {
            eligible.push(core.mask & allowed);
        }
    }
    if allowed & !covered != 0 {
        return fallback(Reason::IncompleteGroup);
    }
    eligible.sort_unstable_by_key(|mask| mask.trailing_zeros());
    let pair = (default_mask.count_ones() == 2).then(|| {
        eligible
            .iter()
            .any(|mask| mask & default_mask == default_mask)
    });
    if allowed.count_ones() == 1 {
        return fallback(Reason::SingleAllowed);
    }
    if eligible.len() < 2 {
        return Selection {
            default_pair_same_core: pair,
            ..fallback(Reason::SingleCore)
        };
    }
    Selection {
        default_mask,
        candidate_mask: eligible[0].isolate_lowest_one() | eligible[1].isolate_lowest_one(),
        reason: Reason::DistinctCores,
        default_pair_same_core: pair,
    }
}

#[cfg(test)]
#[path = "cpu_selection/tests.rs"]
mod tests;
