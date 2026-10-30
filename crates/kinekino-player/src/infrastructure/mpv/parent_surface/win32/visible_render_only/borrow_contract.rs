//! Compile-pass controls inside the actual private module; no construction or OS execution.
use super::{VisibleRenderOnlyChild, VisibleRenderOnlyProof};

fn scoped_result(owner: &VisibleRenderOnlyChild) {
    let _owned = owner.with_visible_render_only(|proof| proof.verify().is_ok());
}
fn borrow_without_cloning(proof: &VisibleRenderOnlyProof<'_>) {
    let borrowed: &VisibleRenderOnlyProof<'_> = proof;
    let _ = borrowed.verify();
}
fn use_proof_on_current_thread(proof: VisibleRenderOnlyProof<'_>) {
    let _ = proof.verify();
}
fn serialize_owned_observation(proof: &VisibleRenderOnlyProof<'_>) {
    let _ = serde_json::to_string(&proof.verify().is_ok());
}
fn use_owner_on_current_thread(owner: VisibleRenderOnlyChild) {
    let _ = owner.first_error();
}
