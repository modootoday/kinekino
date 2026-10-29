use super::*;
use crate::parent_host::ParentReceiptSink;
use kinekino_process::application::raw_records::RecordPoll;
impl<G: Send> ParentLaunch<G> {
    /// Same guards and owner remain present on failure or external sink panic.
    /// Never renew the operation or cleanup budget and never retry a launch.
    pub fn poll_close(
        &mut self,
        requested: Deadline,
        sink: &mut impl ParentReceiptSink,
    ) -> LaunchCloseFacts {
        self.closing = true;
        self.deadline = self.deadline.constrained_by(requested);
        if let Some(host) = &mut self.host {
            let facts = host.poll_close(self.deadline, sink);
            return LaunchCloseFacts {
                host: Some(facts),
                prepared_ipc: None,
                raw_eof: [facts.stdout_eof, facts.stderr_eof],
                launch_unreturned: false,
            };
        }
        let Some(prepared) = &mut self.prepared else {
            return LaunchCloseFacts {
                host: None,
                prepared_ipc: None,
                raw_eof: self.raw_eof,
                launch_unreturned: self.unreturned,
            };
        };
        // No child exists. Release original evidence writers before observing EOF.
        drop(prepared.evidence.take());
        let ipc = prepared.ipc.poll_close();
        if ipc.had_error {
            self.first.get_or_insert(OpenOnceError::Ipc);
        }
        for index in 0..2 {
            if self.raw_eof[index] || self.raw_failed[index] {
                continue;
            }
            let raw = if index == 0 {
                &self.evidence.stdout
            } else {
                &self.evidence.stderr
            };
            match self.cursors[index].poll(raw, 4096, true) {
                Ok(RecordPoll::Pending) => self.raw_eof[index] = true,
                Ok(RecordPoll::Record { .. }) => {}
                Err(error) => {
                    self.raw_failed[index] = true;
                    self.remember_io(error);
                }
            }
        }
        LaunchCloseFacts {
            host: None,
            prepared_ipc: Some(ipc),
            raw_eof: self.raw_eof,
            launch_unreturned: false,
        }
    }
}
