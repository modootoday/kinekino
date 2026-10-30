//! Original own-window supervisor parameters; common owner/deadline logic is reused.
pub(super) use crate::test_support::native_window_supervision::RunFailure;
pub(super) const CHILD_MODE: &str = "KINEKINO_QUARANTINE_OWN_WINDOW_CHILD";
pub(super) const CHILD_EVIDENCE: &str = "KINEKINO_QUARANTINE_OWN_WINDOW_EVIDENCE";
pub(super) const EXACT_TEST: &str = "assembly::meari::quarantine_surface::tests::own_window::original_registrar_frame_disabled_child_and_cleanup";
pub(super) fn parent_run() -> Result<(), Box<RunFailure>> {
    crate::test_support::native_window_supervision::parent_run_for(
        EXACT_TEST,
        CHILD_MODE,
        CHILD_EVIDENCE,
        "quarantine-own-window-inner-v1",
        "runtime/quality-review/ui-own-window-process",
    )
}
