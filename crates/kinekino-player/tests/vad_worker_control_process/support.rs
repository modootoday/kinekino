use super::*;
impl Case {
    pub(super) fn wait_ready(&mut self, name: &str) {
        loop {
            match crate::fixture_data::ordinary_paths::file(&self.root.join(name)) {
                Ok(mut file) => {
                    let mut raw = Vec::new();
                    Read::by_ref(&mut file)
                        .take(7)
                        .read_to_end(&mut raw)
                        .unwrap();
                    assert_eq!(raw, b"ready\n");
                    self.guards.push(file);
                    return;
                }
                Err(error) if error.code == ErrorCode::NotFound => {}
                Err(error) => panic!("Original ready observation failed: {error:?}"),
            }
            if self
                .owner
                .as_mut()
                .unwrap()
                .poll_state()
                .unwrap()
                .child_exit
                .is_some()
            {
                self.export();
                panic!("Child exited before original ready marker");
            }
            assert!(
                Instant::now() < self.operation_end,
                "Original operation expired before ready"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    pub(super) fn release(&mut self, name: &str) {
        self.guards
            .push(fixture_barrier::publish(&self.root, name, b"release\n").unwrap());
    }
    pub(super) fn wait_exit(&mut self) {
        loop {
            let receipt = self.owner.as_mut().unwrap().poll_state().unwrap();
            if receipt.child_exit.is_some() && receipt.family == FamilyObservation::Empty {
                break;
            }
            assert!(
                Instant::now() < self.end,
                "Original cleanup end reached; owner must be retained"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        self.export();
    }
    pub(super) fn export(&mut self) {
        if self.exported {
            return;
        }
        if let Some(owner) = &self.owner {
            for (name, file) in [
                ("stdout", &owner.evidence().stdout),
                ("stderr", &owner.evidence().stderr),
            ] {
                let mut input = file.open_read().unwrap();
                let length = input.metadata().unwrap().len();
                let mut bytes = Vec::new();
                Read::by_ref(&mut input)
                    .take(kinekino_vad::protocol::application::MAX_OUTCOME_BYTES + 1)
                    .read_to_end(&mut bytes)
                    .unwrap();
                fs::write(self.root.join(format!("{name}.bin")), &bytes).unwrap();
                fs::write(self.root.join(format!("{name}-observation.json")), serde_json::to_vec_pretty(&json!({"original":file.path(),"observed_length":length,"preserved_bytes":bytes.len(),"complete":length==bytes.len() as u64})).unwrap()).unwrap();
            }
            JsonReceipt(self.root.join("observed.json"))
                .record(owner.receipt())
                .unwrap();
            self.exported = true;
        }
    }
    pub(super) fn decoded(&self) -> kinekino_vad::protocol::domain::WorkerOutcomeCandidate {
        let raw = fs::read(self.root.join("stdout.bin")).unwrap();
        kinekino_vad::protocol::application::decode_worker_outcome(
            raw.as_slice(),
            raw.len() as u64,
            &self.data.expected,
        )
        .unwrap()
    }
    pub(super) fn close(&mut self) -> bool {
        if let Some(transport) = &mut self.native {
            loop {
                match transport.poll_close(self.end) {
                    Ok(kinekino_jobs::application::owned_attempt::OwnerObservation::Exited {
                        family_empty,
                        streams_eof,
                        ..
                    }) => {
                        assert!(family_empty && streams_eof);
                        if let Some(error) = &self.close_error {
                            panic!(
                                "Original native cleanup error retained in transport: {error:?}"
                            );
                        }
                        return true;
                    }
                    Ok(kinekino_jobs::application::owned_attempt::OwnerObservation::NotStarted) => {
                        return true;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        self.close_error.get_or_insert(error);
                    }
                }
                if Instant::now() >= self.end {
                    return false;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if let Some(owner) = &mut self.owner {
            let mut sequence = 0;
            loop {
                let progress = owner.poll_cleanup(self.deadline, &SystemClock);
                let ready = progress.physical_closed();
                if ready || progress.first_error().is_some() || progress.deadline_expired() {
                    sequence += 1;
                    let _result = owner.record_cleanup(
                        &mut JsonReceipt(self.root.join(format!("cleanup-{sequence}.json"))),
                        &SystemClock,
                    );
                }
                let progress = owner.cleanup_progress().unwrap();
                if progress.settled() {
                    self.terminal = Some(progress.receipt().clone());
                    assert!(
                        progress.first_error().is_none()
                            && progress.secondary_errors().is_empty()
                            && !progress.deadline_expired(),
                        "Original cleanup error/expiry is not success"
                    );
                    return true;
                }
                if Instant::now() >= self.end {
                    return false;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        true
    }
}
