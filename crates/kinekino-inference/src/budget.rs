//! Cost guard and local ledger. Unknown estimate or crossing `confirm_ratio` asks the user;
//! exceeding any cap refuses.
use crate::port::InferenceError;
use crate::settings::Budget;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub month: String,
    pub backend: String,
    pub model: String,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuardDecision {
    Allow,
    /// Ask the user (unknown cost or near a cap); carries the reason shown in the dialog.
    Confirm(String),
}

#[derive(Debug, Clone, Default)]
pub struct CostGuard {
    budget: Budget,
    ledger: Vec<LedgerEntry>,
}

impl CostGuard {
    pub fn new(budget: Budget) -> Self {
        Self {
            budget,
            ledger: Vec::new(),
        }
    }

    pub fn from_jsonl(budget: Budget, text: &str) -> Result<Self, String> {
        let ledger = text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        Ok(Self { budget, ledger })
    }

    pub fn to_jsonl(&self) -> String {
        self.ledger
            .iter()
            .filter_map(|e| serde_json::to_string(e).ok())
            .map(|l| l + "\n")
            .collect()
    }

    pub fn total(&self) -> f64 {
        self.ledger.iter().map(|e| e.cost_usd).sum()
    }

    pub fn month_total(&self, month: &str) -> f64 {
        self.ledger
            .iter()
            .filter(|e| e.month == month)
            .map(|e| e.cost_usd)
            .sum()
    }

    pub fn check(
        &self,
        month: &str,
        estimate: Option<f64>,
    ) -> Result<GuardDecision, InferenceError> {
        let b = &self.budget;
        let Some(cost) = estimate else {
            return Ok(GuardDecision::Confirm("cost estimate unknown".into()));
        };
        if cost > b.per_call_usd {
            return Err(InferenceError::Budget(format!(
                "estimate ${cost:.4} > per-call ${:.2}",
                b.per_call_usd
            )));
        }
        let (total, monthly) = (self.total() + cost, self.month_total(month) + cost);
        if total > b.total_usd || monthly > b.monthly_usd {
            return Err(InferenceError::Budget(format!(
                "would reach total ${total:.4}/{:.2}, month ${monthly:.4}/{:.2}",
                b.total_usd, b.monthly_usd
            )));
        }
        if total >= b.total_usd * b.confirm_ratio || monthly >= b.monthly_usd * b.confirm_ratio {
            return Ok(GuardDecision::Confirm(format!(
                "near limit: total ${total:.2}/{:.2}, month ${monthly:.2}/{:.2}",
                b.total_usd, b.monthly_usd
            )));
        }
        Ok(GuardDecision::Allow)
    }

    pub fn record(&mut self, entry: LedgerEntry) {
        self.ledger.push(entry);
    }

    pub fn reset(&mut self) {
        self.ledger.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(cost: f64) -> LedgerEntry {
        LedgerEntry {
            month: "2026-10".into(),
            backend: "mock".into(),
            model: "m".into(),
            cost_usd: cost,
        }
    }

    #[test]
    fn guard_allows_confirms_and_refuses() {
        let mut g = CostGuard::new(Budget::default());
        assert_eq!(g.check("2026-10", Some(0.1)).unwrap(), GuardDecision::Allow);
        assert!(matches!(
            g.check("2026-10", None).unwrap(),
            GuardDecision::Confirm(_)
        ));
        assert!(matches!(
            g.check("2026-10", Some(0.6)),
            Err(InferenceError::Budget(_))
        ));
        for _ in 0..8 {
            g.record(entry(0.5));
        }
        assert!(matches!(
            g.check("2026-10", Some(0.1)).unwrap(),
            GuardDecision::Confirm(_)
        ));
        g.record(entry(0.5));
        g.record(entry(0.5));
        assert!(matches!(
            g.check("2026-10", Some(0.01)),
            Err(InferenceError::Budget(_))
        ));
        let back = CostGuard::from_jsonl(Budget::default(), &g.to_jsonl()).unwrap();
        assert!((back.total() - 5.0).abs() < 1e-9);
        g.reset();
        assert_eq!(g.total(), 0.0);
    }
}
