use kinekino_media_contracts::errors::{ApiError, ErrorCode};

const MAX_SECONDARY: usize = 8;

/// Bounded immutable copy; ApiError.details/request labels are not authority.
#[derive(Debug, PartialEq, Eq)]
pub struct AttemptFailure {
    code: ErrorCode,
    message: String,
    remediation: String,
}

impl AttemptFailure {
    pub fn capture(error: &ApiError) -> Self {
        Self {
            code: error.code,
            message: bounded(&error.message, 4096),
            remediation: bounded(&error.remediation, 256),
        }
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn remediation(&self) -> &str {
        &self.remediation
    }
}

#[derive(Debug, Default)]
pub struct AttemptDiagnostic {
    first: Option<AttemptFailure>,
    secondary: Vec<AttemptFailure>,
    secondary_overflow: bool,
}

impl AttemptDiagnostic {
    pub fn first(&self) -> Option<&AttemptFailure> {
        self.first.as_ref()
    }

    pub fn secondary(&self) -> &[AttemptFailure] {
        &self.secondary
    }

    pub fn secondary_overflow(&self) -> bool {
        self.secondary_overflow
    }

    pub(in crate::application) fn record(&mut self, error: &ApiError) {
        let failure = AttemptFailure::capture(error);
        if self.first.is_none() {
            self.first = Some(failure);
        } else if self.secondary.len() < MAX_SECONDARY {
            self.secondary.push(failure);
        } else {
            self.secondary_overflow = true;
        }
    }
}

fn bounded(value: &str, limit: usize) -> String {
    let mut end = value.len().min(limit);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}
