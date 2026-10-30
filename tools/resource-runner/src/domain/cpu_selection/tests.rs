use super::*;
fn core(group: u16, mask: usize) -> Core {
    Core {
        group,
        mask,
        smt: mask.count_ones() > 1,
        efficiency_class: 0,
    }
}

#[test]
fn smt_siblings_are_reported_but_default_stays_lowest_two() {
    let selected = compare(15, 0, Some(&[core(0, 3), core(0, 12)]));
    assert_eq!(selected.default_mask, 3);
    assert_eq!(selected.candidate_mask, 5);
    assert_eq!(selected.default_pair_same_core, Some(true));
    assert_eq!(selected.reason, Reason::DistinctCores);
    let separate = compare(15, 0, Some(&[core(0, 5), core(0, 10)]));
    assert_eq!(separate.candidate_mask, 3);
    assert_eq!(separate.default_pair_same_core, Some(false));
}

#[test]
fn group_and_partial_permissions_are_intersected() {
    let cores = [core(0, 3), core(1, 3), core(1, 12)];
    let selected = compare(10, 1, Some(&cores));
    assert_eq!(selected.candidate_mask, 10);
    for allowed in 0..16 {
        let selected = compare(allowed, 1, Some(&cores));
        assert_eq!(selected.candidate_mask & !allowed, 0);
        assert!(selected.candidate_mask.count_ones() <= 2);
        assert_eq!(selected.default_mask, first_two(allowed));
    }
}

#[test]
fn narrower_thread_scope_does_not_redefine_process_default() {
    let cores = [core(0, 3), core(0, 12)];
    let legacy = compare(15, 0, Some(&cores));
    let candidate = compare(12, 0, Some(&cores));
    assert_eq!(legacy.default_mask, 3);
    assert_eq!(legacy.default_pair_same_core, Some(true));
    assert_eq!(candidate.default_mask, 12);
    assert_eq!(candidate.candidate_mask, 12);
    assert_eq!(candidate.reason, Reason::SingleCore);
}

#[test]
fn unknown_incomplete_and_conflicting_topology_fall_back() {
    for (cores, reason) in [
        (None, Reason::QueryFailed),
        (Some(vec![]), Reason::IncompleteGroup),
        (Some(vec![core(0, 3)]), Reason::IncompleteGroup),
        (Some(vec![core(0, 3), core(0, 6)]), Reason::InvalidTopology),
        (Some(vec![core(0, 0)]), Reason::InvalidTopology),
    ] {
        let selected = compare(7, 0, cores.as_deref());
        assert_eq!(selected.candidate_mask, first_two(7));
        assert_eq!(selected.reason, reason);
        assert_eq!(selected.default_pair_same_core, None);
    }
}

#[test]
fn empty_one_cpu_and_one_core_do_not_invent_a_second_core() {
    assert_eq!(compare(0, 0, None).reason, Reason::EmptyAllowed);
    let one = compare(8, 0, Some(&[core(0, 12)]));
    assert_eq!((one.candidate_mask, one.reason), (8, Reason::SingleAllowed));
    let siblings = compare(3, 0, Some(&[core(0, 3)]));
    assert_eq!(
        (siblings.candidate_mask, siblings.reason),
        (3, Reason::SingleCore)
    );
    assert_eq!(siblings.default_pair_same_core, Some(true));
}
