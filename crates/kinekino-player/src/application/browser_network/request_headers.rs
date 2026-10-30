//! ExtraInfo correlation is scoped to one protocol session and a unique request ID.
use crate::domain::api_values::DecimalU64;
use crate::domain::request_referer::RequestReferer;
use serde::{Deserialize, Serialize};

/// Sanitized observation only; never permission to replay a request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialObservation {
    #[default]
    Unknown,
    Present,
    NoneObserved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestHeaderEvidence {
    #[serde(default)]
    pub credentials: CredentialObservation,
    pub referer: RequestReferer,
    pub received_unix_ms: Option<DecimalU64>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum HeaderAssociation {
    Unobserved,
    UniqueRequest,
    Ambiguous,
    BudgetExceeded,
}

#[derive(Clone)]
struct HeaderSlot {
    request_id: String,
    request_seen: bool,
    extra_seen: bool,
    ambiguous: bool,
    evidence: Option<RequestHeaderEvidence>,
}
#[derive(Clone, Default)]
pub(super) struct RequestHeaders {
    slots: Vec<HeaderSlot>,
}
impl RequestHeaders {
    fn slot(&mut self, id: &str) -> Option<&mut HeaderSlot> {
        if let Some(index) = self.slots.iter().position(|slot| slot.request_id == id) {
            return self.slots.get_mut(index);
        }
        if self.slots.len() >= 64 {
            return None;
        }
        self.slots.push(HeaderSlot {
            request_id: id.into(),
            request_seen: false,
            extra_seen: false,
            ambiguous: false,
            evidence: None,
        });
        self.slots.last_mut()
    }
    pub fn request(&mut self, id: &str, redirected: bool) {
        if let Some(slot) = self.slot(id) {
            slot.ambiguous |= slot.request_seen || redirected;
            slot.request_seen = true;
        }
    }
    #[cfg(test)]
    pub fn extra(&mut self, id: &str, referer: RequestReferer) {
        self.extra_at(id, referer, None);
    }
    #[cfg(test)]
    pub fn extra_at(
        &mut self,
        id: &str,
        referer: RequestReferer,
        received_unix_ms: Option<DecimalU64>,
    ) {
        self.extra_with_credentials(
            id,
            referer,
            received_unix_ms,
            CredentialObservation::Unknown,
        );
    }
    pub fn extra_with_credentials(
        &mut self,
        id: &str,
        referer: RequestReferer,
        received_unix_ms: Option<DecimalU64>,
        credentials: CredentialObservation,
    ) {
        if let Some(slot) = self.slot(id) {
            slot.ambiguous |= slot.extra_seen;
            slot.extra_seen = true;
            slot.evidence = Some(RequestHeaderEvidence {
                credentials,
                referer,
                received_unix_ms,
            });
        }
    }
    pub fn observed(&self, id: &str) -> Option<&RequestHeaderEvidence> {
        self.slots
            .iter()
            .find(|slot| {
                slot.request_id == id && slot.request_seen && slot.extra_seen && !slot.ambiguous
            })?
            .evidence
            .as_ref()
    }
    pub fn association(&self, id: &str) -> HeaderAssociation {
        match self.slots.iter().find(|slot| slot.request_id == id) {
            None => HeaderAssociation::BudgetExceeded,
            Some(slot) if slot.ambiguous => HeaderAssociation::Ambiguous,
            Some(slot) if slot.request_seen && slot.extra_seen => HeaderAssociation::UniqueRequest,
            Some(_) => HeaderAssociation::Unobserved,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_header_evidence_defaults_to_unknown_credentials() {
        let original = RequestHeaderEvidence {
            credentials: CredentialObservation::Present,
            referer: RequestReferer::Absent,
            received_unix_ms: None,
        };
        let mut json = serde_json::to_value(original).unwrap();
        json.as_object_mut().unwrap().remove("credentials");
        let historical: RequestHeaderEvidence = serde_json::from_value(json).unwrap();
        assert_eq!(historical.credentials, CredentialObservation::Unknown);
    }
    #[test]
    fn timestamps_are_observations_and_slot_limit_cannot_create_header_absence() {
        let mut headers = RequestHeaders::default();
        for index in 0..64 {
            headers.request(&format!("request-{index}"), false);
        }
        headers.extra_at("request-0", RequestReferer::Absent, Some(DecimalU64(17)));
        assert_eq!(
            headers.observed("request-0").unwrap().received_unix_ms,
            Some(DecimalU64(17))
        );
        headers.request("overflow", false);
        headers.extra("overflow", RequestReferer::Absent);
        assert!(headers.observed("overflow").is_none());
        assert!(matches!(
            headers.association("overflow"),
            HeaderAssociation::BudgetExceeded
        ));
        assert!(matches!(
            headers.association("request-1"),
            HeaderAssociation::Unobserved
        ));
    }
    #[test]
    fn either_event_order_binds_explicit_absence_and_empty_without_inference() {
        let mut headers = RequestHeaders::default();
        headers.extra("early", RequestReferer::Absent);
        assert!(headers.observed("early").is_none());
        headers.request("early", false);
        assert_eq!(
            headers.observed("early").map(|evidence| &evidence.referer),
            Some(&RequestReferer::Absent)
        );
        headers.request("late", false);
        assert!(headers.observed("late").is_none());
        headers.extra("late", RequestReferer::Empty);
        assert_eq!(
            headers.observed("late").map(|evidence| &evidence.referer),
            Some(&RequestReferer::Empty)
        );
    }
    #[test]
    fn redirect_or_duplicate_extra_info_never_guesses_a_hop() {
        let mut headers = RequestHeaders::default();
        headers.request("redirect", false);
        headers.extra("redirect", RequestReferer::Absent);
        headers.request("redirect", true);
        assert!(headers.observed("redirect").is_none());
        headers.request("duplicate", false);
        headers.extra("duplicate", RequestReferer::Absent);
        headers.extra("duplicate", RequestReferer::Absent);
        assert!(headers.observed("duplicate").is_none());
    }
}
