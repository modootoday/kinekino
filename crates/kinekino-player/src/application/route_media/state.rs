use super::*;

enum Phase<S> {
    Active(Option<ViewingTicket<S>>),
    Retiring(RetireTicket<S>),
    Retired(RetireTicket<S>),
    Viewing {
        retired: RetireTicket<S>,
        ticket: ViewingTicket<S>,
    },
}
/// Policy only; the native adapter retains every partial cleanup/registration owner separately.
pub struct RouteMediaState<S> {
    owner: S,
    identity: Arc<()>,
    generation: u64,
    phase: Phase<S>,
    first_error: Option<RouteMediaError>,
}
impl<S: Clone + PartialEq> RouteMediaState<S> {
    pub fn new(owner: S) -> Self {
        Self {
            owner,
            identity: Arc::new(()),
            generation: 1,
            phase: Phase::Active(None),
            first_error: None,
        }
    }
    fn owner(&self, owner: &S) -> Result<(), RouteMediaError> {
        if owner == &self.owner {
            Ok(())
        } else {
            Err(conflict())
        }
    }
    fn retire_identity(&self, ticket: &RetireTicket<S>) -> Result<(), RouteMediaError> {
        self.owner(&ticket.owner)?;
        if Arc::ptr_eq(&self.identity, &ticket.identity) && ticket.generation == self.generation {
            Ok(())
        } else {
            Err(conflict())
        }
    }
    fn viewing_identity(&self, ticket: &ViewingTicket<S>) -> Result<(), RouteMediaError> {
        self.owner(&ticket.owner)?;
        if Arc::ptr_eq(&self.identity, &ticket.identity) && ticket.generation == self.generation {
            Ok(())
        } else {
            Err(conflict())
        }
    }
    fn healthy(&self) -> Result<(), RouteMediaError> {
        match &self.first_error {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
    pub fn begin_retire(&mut self, owner: &S) -> Result<RetireTicket<S>, RouteMediaError> {
        self.owner(owner)?;
        self.healthy()?;
        match &self.phase {
            Phase::Retiring(ticket) | Phase::Retired(ticket) => Ok(ticket.clone()),
            Phase::Viewing { .. } => Err(unavailable()),
            Phase::Active(_) => {
                let ticket = RetireTicket {
                    owner: self.owner.clone(),
                    generation: self.generation,
                    identity: self.identity.clone(),
                };
                self.phase = Phase::Retiring(ticket.clone());
                Ok(ticket)
            }
        }
    }
    pub fn poll_retire(
        &self,
        ticket: &RetireTicket<S>,
    ) -> Result<RouteMediaProgress, RouteMediaError> {
        self.retire_identity(ticket)?;
        self.healthy()?;
        match &self.phase {
            Phase::Retiring(_) => Ok(RouteMediaProgress::Pending),
            Phase::Retired(_) => Ok(RouteMediaProgress::Complete),
            _ => Err(conflict()),
        }
    }
    pub fn complete_retire(&mut self, ticket: &RetireTicket<S>) -> Result<(), RouteMediaError> {
        self.poll_retire(ticket)?;
        self.phase = Phase::Retired(ticket.clone());
        Ok(())
    }
    pub fn fail_retire(
        &mut self,
        ticket: &RetireTicket<S>,
        error: RouteMediaError,
    ) -> Result<(), RouteMediaError> {
        self.retire_identity(ticket)?;
        if !matches!(self.phase, Phase::Retiring(_) | Phase::Retired(_)) {
            return Err(conflict());
        }
        self.first_error.get_or_insert(error);
        Ok(())
    }
    pub fn cancel_retire(&mut self, ticket: &RetireTicket<S>) -> Result<(), RouteMediaError> {
        self.fail_retire(
            ticket,
            Arc::new(ApiError::new(
                ErrorCode::Cancelled,
                "Original media retirement cancelled; cleanup owners remain retained",
            )),
        )
    }
    pub fn begin_viewing(
        &mut self,
        owner: &S,
        retired: &RetireTicket<S>,
    ) -> Result<ViewingTicket<S>, RouteMediaError> {
        self.owner(owner)?;
        self.healthy()?;
        if let Phase::Viewing {
            retired: previous,
            ticket,
        } = &self.phase
        {
            return if Arc::ptr_eq(&previous.identity, &retired.identity)
                && previous.generation == retired.generation
                && previous.owner == retired.owner
            {
                Ok(ticket.clone())
            } else {
                Err(conflict())
            };
        }
        if self.poll_retire(retired)? != RouteMediaProgress::Complete {
            return Err(unavailable());
        }
        self.generation = self.generation.checked_add(1).ok_or_else(conflict)?;
        let ticket = ViewingTicket {
            owner: self.owner.clone(),
            generation: self.generation,
            identity: self.identity.clone(),
        };
        self.phase = Phase::Viewing {
            retired: retired.clone(),
            ticket: ticket.clone(),
        };
        Ok(ticket)
    }
    pub fn poll_viewing(
        &self,
        ticket: &ViewingTicket<S>,
    ) -> Result<RouteMediaProgress, RouteMediaError> {
        self.viewing_identity(ticket)?;
        self.healthy()?;
        match &self.phase {
            Phase::Viewing { .. } => Ok(RouteMediaProgress::Pending),
            Phase::Active(Some(current)) if current.generation == ticket.generation => {
                Ok(RouteMediaProgress::Complete)
            }
            _ => Err(conflict()),
        }
    }
    pub fn complete_viewing(&mut self, ticket: &ViewingTicket<S>) -> Result<(), RouteMediaError> {
        self.poll_viewing(ticket)?;
        self.phase = Phase::Active(Some(ticket.clone()));
        Ok(())
    }
    pub fn fail_viewing(
        &mut self,
        ticket: &ViewingTicket<S>,
        error: RouteMediaError,
    ) -> Result<(), RouteMediaError> {
        self.viewing_identity(ticket)?;
        self.first_error.get_or_insert(error);
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
