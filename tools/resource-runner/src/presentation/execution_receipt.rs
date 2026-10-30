use crate::child_execution::{Failure, Sink};
use crate::{execution_admission::Admission, execution_profile::Profile};
use std::time::Instant;
use std::{fs, io, path::Path};

#[derive(Clone, Copy)]
pub struct Receipt<'a> {
    pub profile: Option<Profile>,
    pub requested_profile: &'a str,
    pub admission: &'a Admission,
    pub child: Option<(u32, usize, u32)>,
    pub terminal: bool,
    pub exit: Option<(i32, u128)>,
}
pub struct LaunchSink<'a> {
    pub receipt: Receipt<'a>,
    pub path: &'a Path,
}
impl crate::launch_stage::Sink for LaunchSink<'_> {
    fn launch_failed(&mut self, failure: &crate::launch_stage::Failure) -> io::Result<()> {
        fs::write(self.path, self.receipt.render_launch_failure(failure))
    }
}
impl Receipt<'_> {
    fn render_launch_failure(&self, failure: &crate::launch_stage::Failure) -> String {
        let mut rendered = Self {
            child: None,
            terminal: true,
            exit: Some((125, 0)),
            ..*self
        }
        .render();
        rendered.truncate(rendered.len() - 2);
        rendered.push_str(&format!(
            ",\"launch_failure\":{{\"stage\":{},\"original_error\":{}}}}}\n",
            quote(failure.stage),
            quote(&failure.original_error.to_string())
        ));
        rendered
    }
    pub fn write(&self, path: &Path) -> io::Result<()> {
        fs::write(path, self.render())
    }
    fn render(&self) -> String {
        self.render_failure(None)
    }
    fn render_failure(&self, failure: Option<&Failure>) -> String {
        let admission = self.admission;
        let samples = admission.intervals.iter().map(|sample| format!(
            "{{\"busy_ticks\":{},\"total_ticks\":{},\"minimum_free_bytes\":{},\"elapsed_ms\":{}}}",
            sample.busy, sample.total, sample.minimum_free_bytes, sample.elapsed_ms
        )).collect::<Vec<_>>().join(",");
        let rejection = admission.rejection.map_or_else(|| "null".into(), quote);
        let query_failure = admission.query_failure.map_or_else(
            || "null".into(),
            |failure| {
                format!(
                    "{{\"stage\":{},\"os_code\":{}}}",
                    quote(failure.stage),
                    failure
                        .os_code
                        .map_or_else(|| "null".into(), |code| code.to_string())
                )
            },
        );
        let child = self.child.map_or_else(|| failure.map_or_else(String::new, |failure| format!(",\"child_pid\":{}", failure.pid)), |(pid, mask, priority)| format!(
            ",\"child_pid\":{pid},\"affinity\":{mask},\"logical_cpus\":{},\"priority\":{priority},\"job_applies_to_descendants\":true", mask.count_ones()
        ));
        let exit = self.exit.map_or_else(String::new, |(code, elapsed)| {
            format!(",\"exit_code\":{code},\"elapsed_ms\":{elapsed}")
        });
        let failure_fields = failure.map_or_else(String::new, |failure| {
            let cleanup = &failure.cleanup;
            format!(",\"failure\":{{\"stage\":{},\"original_error\":{},\"cleanup\":{{\"attempted\":{},\"reaped\":{},\"exit_code\":{},\"kill_error\":{},\"wait_error\":{}}}}}",
                quote(failure.stage), quote(&failure.original_error.to_string()), cleanup.attempted,cleanup.reaped,
                cleanup.exit_code.map_or_else(|| "null".into(), |code|code.to_string()),
                cleanup.kill_error.as_deref().map_or_else(|| "null".into(),quote),
                cleanup.wait_error.as_deref().map_or_else(|| "null".into(),quote))
        });
        let terminal = failure.map_or(self.terminal, |failure| failure.cleanup.reaped);
        format!(
            "{{\"launcher_pid\":{},\"profile\":{},\"requested_profile\":{},\"requested_logical_cpus\":{},\"effective_logical_cpus\":{},\"admission\":{{\"accepted\":{},\"selected_mask\":{},\"rejection\":{rejection},\"query_failure\":{query_failure},\"intervals\":[{samples}]}}{child},\"terminal\":{}{exit}{failure_fields}}}\n",
            std::process::id(),
            self.profile
                .map_or_else(|| "null".into(), |p| quote(p.label())),
            quote(self.requested_profile),
            self.profile.map_or(0, Profile::requested_cpus),
            self.child.map_or(0, |(_, mask, _)| mask.count_ones()),
            admission.rejection.is_none(),
            admission
                .selected_mask
                .map_or_else(|| "null".into(), |mask| mask.to_string()),
            terminal
        )
    }
}
pub struct ExecutionSink<'a> {
    receipt: Receipt<'a>,
    path: &'a Path,
    started: Instant,
}
impl<'a> ExecutionSink<'a> {
    pub fn new(receipt: Receipt<'a>, path: &'a Path) -> Self {
        Self {
            receipt,
            path,
            started: Instant::now(),
        }
    }
}
impl Sink for ExecutionSink<'_> {
    fn started(&mut self, pid: u32, budget: (usize, u32)) -> io::Result<()> {
        self.receipt.child = Some((pid, budget.0, budget.1));
        self.receipt.write(self.path)
    }
    fn completed(&mut self, code: i32) -> io::Result<()> {
        self.receipt.terminal = true;
        self.receipt.exit = Some((code, self.started.elapsed().as_millis()));
        self.receipt.write(self.path)
    }
    fn failed(&mut self, failure: &Failure) -> io::Result<()> {
        self.receipt.exit = Some((125, self.started.elapsed().as_millis()));
        fs::write(self.path, self.receipt.render_failure(Some(failure)))
    }
}
fn quote(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            ch if ch.is_control() => result.push_str(&format!("\\u{:04x}", u32::from(ch))),
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn untrusted_profile_cannot_inject_receipt_fields() {
        assert_eq!(super::quote("bad\"\n\\"), "\"bad\\\"\\u000a\\\\\"");
    }
    #[test]
    fn rejection_has_no_child_identity_or_claimed_budget() {
        let admission = Admission::rejected("unknown_profile");
        let rendered = Receipt {
            profile: None,
            requested_profile: "build8",
            admission: &admission,
            child: None,
            terminal: true,
            exit: Some((125, 0)),
        }
        .render();
        assert!(!rendered.contains("\"child_pid\""));
        assert!(!rendered.contains("\"job_applies_to_descendants\""));
        assert!(rendered.contains("\"accepted\":false"));
        assert!(rendered.contains("\"exit_code\":125"));
        assert!(rendered.contains("\"effective_logical_cpus\":0"));
    }
    #[test]
    fn unresolved_child_overrides_any_previous_terminal_flag() {
        let admission = Admission::default();
        let failure = Failure {
            pid: 77,
            stage: "budget_verify",
            original_error: io::Error::other("mismatch"),
            cleanup: crate::child_execution::Cleanup {
                attempted: true,
                reaped: false,
                exit_code: None,
                kill_error: Some("denied".into()),
                wait_error: Some("unknown".into()),
            },
            receipt_error: None,
        };
        let rendered = Receipt {
            profile: Some(Profile::Conservative2),
            requested_profile: "conservative2",
            admission: &admission,
            child: None,
            terminal: true,
            exit: Some((125, 0)),
        }
        .render_failure(Some(&failure));
        assert!(rendered.contains("\"terminal\":false"));
        assert!(rendered.contains("\"child_pid\":77"));
        assert!(rendered.contains("\"reaped\":false"));
        assert!(rendered.contains("\"original_error\":\"mismatch\""));
        assert!(!rendered.contains("\"job_applies_to_descendants\""));
    }
    #[test]
    fn prechild_launch_failure_finishes_receipt_without_child_or_job_claim() {
        let admission = Admission::default();
        let report = Receipt {
            profile: Some(Profile::Conservative2),
            requested_profile: "conservative2",
            admission: &admission,
            child: None,
            terminal: false,
            exit: None,
        };
        for stage in ["stdout_create", "stderr_create", "child_spawn"] {
            let failure = crate::launch_stage::Failure {
                stage,
                original_error: io::Error::other("launch unavailable"),
                receipt_error: None,
            };
            let rendered = report.render_launch_failure(&failure);
            assert!(rendered.contains("\"terminal\":true"));
            assert!(rendered.contains("\"exit_code\":125"));
            assert!(rendered.contains(stage));
            assert!(rendered.contains("launch unavailable"));
            assert!(!rendered.contains("\"child_pid\""));
            assert!(!rendered.contains("\"job_applies_to_descendants\""));
        }
    }
}
