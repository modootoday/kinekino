use super::*;
impl ParentStream {
    pub fn receive(&mut self, frame: Frame, now: u64) -> Result<(), ParentError> {
        self.healthy()?;
        let result = self.receive_inner(frame, now);
        if let Err(error) = result {
            self.first.get_or_insert(error);
        }
        result
    }
    fn receive_inner(&mut self, frame: Frame, now: u64) -> Result<(), ParentError> {
        if frame.kind == Kind::Data {
            if self.chunks.len() >= MAX_PENDING_CHUNKS
                || self
                    .queued_bytes
                    .checked_add(frame.payload.len() as u64)
                    .is_none_or(|n| n > MAX_CREDIT_BYTES)
            {
                return Err(ParentError::Limit);
            }
            self.chunks.try_reserve(1).map_err(|_| ParentError::Limit)?;
        }
        self.exchange.accept(Direction::Child, &frame, now)?;
        match frame.kind {
            Kind::Metadata => {
                let metadata =
                    HttpMetadata::decode_exact(&frame.payload).map_err(ParentError::Metadata)?;
                metadata
                    .check_request(&self.expected_url, self.maximum)
                    .map_err(ParentError::Metadata)?;
                self.metadata = Some(metadata);
                Ok(())
            }
            Kind::Data => {
                let total = self.exchange.offset();
                if total > self.maximum
                    || self
                        .metadata
                        .as_ref()
                        .and_then(HttpMetadata::response_length)
                        .is_some_and(|n| total > n)
                {
                    return Err(ParentError::Length);
                }
                self.queued_bytes += frame.payload.len() as u64;
                self.chunks.push_back(Chunk {
                    bytes: frame.payload,
                    offset: 0,
                });
                Ok(())
            }
            Kind::BodyEnd => {
                if self
                    .metadata
                    .as_ref()
                    .and_then(HttpMetadata::response_length)
                    .is_some_and(|n| self.exchange.offset() != n)
                {
                    return Err(ParentError::Length);
                }
                Ok(())
            }
            Kind::Failure => Err(ParentError::WorkerFailed),
            _ => Err(ParentError::Order),
        }
    }
}
