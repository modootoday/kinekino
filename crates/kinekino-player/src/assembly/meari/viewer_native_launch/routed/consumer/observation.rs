//! Owned metadata after original ready-route revalidation; no cloned native proof or lease.
use crate::{
    application::ui_observation::*,
    assembly::{
        meari::{
            playback_child::routed::RoutedMediaReady, player_shell_host::RoutedFrameObservation,
        },
        native_playback::routed::ReadyRoutedNative,
    },
    domain::api_values::DecimalU64,
};
use meari_native::{
    application::window_lifecycle::{ChildLease, WindowStamp},
    infrastructure::{native_host::HostStamp, window::NativeFrameObservation},
};
use std::{io, sync::Arc, time::Instant};

pub(super) struct ReadyEmitter {
    snapshot: RoutedFrameObservation,
    started: Instant,
    attempted: bool,
}
impl Default for ReadyEmitter {
    fn default() -> Self {
        Self {
            snapshot: Default::default(),
            started: Instant::now(),
            attempted: false,
        }
    }
}
impl ReadyEmitter {
    pub(super) fn snapshot(&self) -> RoutedFrameObservation {
        self.snapshot.clone()
    }
    pub(super) fn capture(
        &mut self,
        ready: &RoutedMediaReady,
        stamp: &HostStamp,
        window: WindowStamp,
        frame: &NativeFrameObservation,
        lease: &ChildLease,
    ) -> Result<(), String> {
        if self.attempted {
            return Ok(());
        }
        if lease.window() != window || frame.document != *stamp {
            return Err("Original Ready frame and registered lease differ".into());
        }
        // This call reacquires the original State/native input; only the owned counter leaves it.
        let route_generation = ready
            .with_ready_route(|_, route| route.generation())
            .map_err(|error| error.to_string())?;
        // All checked State/native borrows have ended before serialization or output.
        self.attempted = true;
        if let Err(error) = self.encode_and_emit(stamp, window, frame, lease.id(), route_generation)
        {
            self.snapshot.diagnostic_error = Some(Arc::new(error));
        }
        Ok(())
    }
    fn encode_and_emit(
        &mut self,
        stamp: &HostStamp,
        window: WindowStamp,
        frame: &NativeFrameObservation,
        child: u64,
        route: u64,
    ) -> io::Result<()> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Instance {
            document: String,
            epoch: u64,
            id: u64,
        }
        // Core's original serialized identity, not Debug text or a new host identity.
        let instance: Instance = serde_json::from_value(
            serde_json::to_value(&stamp.owner.instance).map_err(io::Error::from)?,
        )
        .map_err(io::Error::from)?;
        let ticket = &frame.submission.ticket;
        let record = Arc::new(NativeRoutedFrameReadyV1 {
            schema_version: 1,
            status: ReadyStatus::Activated,
            document: DocumentObservation {
                host_id: DecimalU64(stamp.host_id),
                instance_document: instance.document,
                instance_epoch: DecimalU64(instance.epoch),
                instance_id: DecimalU64(instance.id),
                source_generation: DecimalU64(stamp.owner.source_generation),
                definition_revision: DecimalU64(stamp.definition_revision),
                document_revision: DecimalU64(stamp.document_revision),
                turn_sequence: DecimalU64(stamp.turn_sequence),
            },
            window: WindowObservation {
                owner: DecimalU64(window.owner()),
                epoch: DecimalU64(window.epoch()),
                surface_generation: DecimalU64(window.surface_generation()),
            },
            frame: FrameObservation {
                context_id: ticket.context_id.clone(),
                context_generation: DecimalU64(ticket.context_generation),
                scene_generation: DecimalU64(ticket.scene_generation),
            },
            child_id: DecimalU64(child),
            route_generation: DecimalU64(route),
        });
        let encoded = encoding::ready(&record, self.started.elapsed().as_millis());
        self.snapshot.ready = Some(record);
        let encoded = encoded?;
        if crate::infrastructure::diagnostics::enabled() {
            encoding::write_line(&encoded, &mut io::stdout().lock())?;
        }
        Ok(())
    }
}
