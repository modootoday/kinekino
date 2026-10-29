use super::*;
impl ParentHost {
    pub(super) fn has_unassigned_owner(&self) -> bool {
        self.launch_failure
            .as_ref()
            .is_some_and(|failure| failure.owner.is_some())
    }
    pub(super) fn poll_unassigned(&mut self) -> (ProcessClosure, Option<io::Error>) {
        if self.unassigned_receipt.is_some() {
            return (ProcessClosure::UnassignedClosed, None);
        }
        let Some(owner) = self
            .launch_failure
            .as_mut()
            .and_then(|failure| failure.owner.as_mut())
        else {
            return (ProcessClosure::NotCreated, None);
        };
        // Original launch rollback already attempted termination. Do not retry
        // a failed termination or claim membership using a Job's empty counter.
        let observed = (|| -> io::Result<_> {
            let exit = owner.poll_child()?;
            let job = owner.poll_owned_job()?;
            Ok((exit, job))
        })();
        match observed {
            Ok((Some(exit), FamilyObservation::Empty)) => {
                self.unassigned_receipt = Some(UnassignedCloseReceipt {
                    identity: owner.identity().clone(),
                    child_exit: exit,
                    original_job: FamilyObservation::Empty,
                    launch_state: owner.state(),
                });
                (ProcessClosure::UnassignedClosed, None)
            }
            Ok(_) => (ProcessClosure::Pending, None),
            Err(error) => (ProcessClosure::Pending, Some(error)),
        }
    }
}
