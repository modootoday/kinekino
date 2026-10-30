//! Own every spawned child through observation or an explicit unresolved cleanup outcome.
use std::{fmt, io, process::Child};

pub trait ChildProcess {
    fn id(&self) -> u32;
    fn terminate(&mut self) -> io::Result<()>;
    fn wait(&mut self) -> io::Result<Option<i32>>;
}
impl ChildProcess for Child {
    fn id(&self) -> u32 {
        Child::id(self)
    }
    fn terminate(&mut self) -> io::Result<()> {
        self.kill()
    }
    fn wait(&mut self) -> io::Result<Option<i32>> {
        Child::wait(self).map(|status| status.code())
    }
}
#[derive(Debug)]
pub struct Cleanup {
    pub attempted: bool,
    pub reaped: bool,
    pub exit_code: Option<i32>,
    pub kill_error: Option<String>,
    pub wait_error: Option<String>,
}
#[derive(Debug)]
pub struct Failure {
    pub pid: u32,
    pub stage: &'static str,
    pub original_error: io::Error,
    pub cleanup: Cleanup,
    pub receipt_error: Option<String>,
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "owned child {} failed at {}: {}; cleanup={:?}; failure_receipt_error={:?}",
            self.pid, self.stage, self.original_error, self.cleanup, self.receipt_error
        )
    }
}
impl std::error::Error for Failure {}

pub trait Sink {
    fn started(&mut self, pid: u32, budget: (usize, u32)) -> io::Result<()>;
    fn completed(&mut self, code: i32) -> io::Result<()>;
    fn failed(&mut self, failure: &Failure) -> io::Result<()>;
}
struct Owner<C: ChildProcess> {
    child: C,
    reaped: bool,
    exit_code: Option<i32>,
}
impl<C: ChildProcess> Owner<C> {
    fn wait(&mut self) -> io::Result<i32> {
        let status = self.child.wait()?;
        self.reaped = true;
        self.exit_code = status;
        Ok(status.unwrap_or(125))
    }
    fn cleanup(&mut self) -> Cleanup {
        if self.reaped {
            return Cleanup {
                attempted: false,
                reaped: true,
                exit_code: self.exit_code,
                kill_error: None,
                wait_error: None,
            };
        }
        let kill_error = self.child.terminate().err().map(|error| error.to_string());
        let wait_error = match self.child.wait() {
            Ok(status) => {
                self.reaped = true;
                self.exit_code = status;
                None
            }
            Err(error) => Some(error.to_string()),
        };
        Cleanup {
            attempted: true,
            reaped: self.reaped,
            exit_code: self.exit_code,
            kill_error,
            wait_error,
        }
    }
    fn failure(&mut self, stage: &'static str, error: io::Error) -> Failure {
        Failure {
            pid: self.child.id(),
            stage,
            original_error: error,
            cleanup: self.cleanup(),
            receipt_error: None,
        }
    }
}
impl<C: ChildProcess> Drop for Owner<C> {
    fn drop(&mut self) {
        if !self.reaped {
            let result = self.cleanup();
            eprintln!(
                "Owned child {} final guard cleanup: {result:?}",
                self.child.id()
            );
        }
    }
}

pub fn supervise<C: ChildProcess>(
    child: C,
    verify: impl FnOnce(&C) -> io::Result<(usize, u32)>,
    sink: &mut impl Sink,
) -> Result<i32, Box<Failure>> {
    let mut owner = Owner {
        child,
        reaped: false,
        exit_code: None,
    };
    let result = (|| {
        let budget = verify(&owner.child).map_err(|error| ("budget_verify", error))?;
        sink.started(owner.child.id(), budget)
            .map_err(|error| ("started_receipt", error))?;
        let code = owner.wait().map_err(|error| ("child_wait", error))?;
        sink.completed(code)
            .map_err(|error| ("terminal_receipt", error))?;
        Ok(code)
    })();
    match result {
        Ok(code) => Ok(code),
        Err((stage, error)) => {
            let mut failure = owner.failure(stage, error);
            if let Err(error) = sink.failed(&failure) {
                failure.receipt_error = Some(error.to_string());
            }
            Err(Box::new(failure))
        }
    }
}

#[cfg(test)]
#[path = "child_execution/tests.rs"]
mod tests;
