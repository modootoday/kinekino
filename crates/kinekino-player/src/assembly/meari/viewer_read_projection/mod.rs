//! Original accepted envelopes survive ACK and DOM projection failures.
pub(crate) mod native_playback;
pub(crate) mod range_feedback;
use crate::{
    application::{playback_session::EpisodePlaybackObservation, playback_status::PlaybackStatus},
    assembly::meari::viewer_callbacks::codec::CapabilitiesReply,
    presentation::web_ui_adapter::player_shell::read_observation::{ReadPanel, ReadPhase},
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::TerminalAcceptance;
use meari::application::plugins::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct Retained {
    ticket: PluginTicket,
    event: Option<PluginEvent>,
    owner_ack: Option<Result<(), PluginError>>,
}
#[derive(Default)]
pub struct ReadProjection {
    pub(crate) range_feedback: range_feedback::RangeFeedback,
    pub(crate) diagnostics_expanded: bool,
    pub(crate) window_ui: crate::assembly::meari::viewer_host::window_ui::WindowUiProjection,
    pub(crate) automatic_enabled: bool,
    pub(crate) automatic_error: String,
    entries: Vec<Retained>,
    pub panel: ReadPanel,
    pub dirty: bool,
    pub(crate) native: Rc<RefCell<native_playback::NativePlaybackProjection>>,
    pub(crate) native_admission: Rc<Cell<bool>>,
    pub(crate) catalog:
        Rc<RefCell<crate::assembly::meari::viewer_callbacks::catalog::CatalogConsumer>>,
    pub(crate) catalog_projection_fault: Rc<Cell<bool>>,
}
pub(crate) fn is_native_control(command: &str) -> bool {
    matches!(
        command,
        "playback.play" | "playback.pause" | "playback.seek" | "playback.set_volume"
    )
}
fn index(command: &str) -> Option<usize> {
    match command {
        "capabilities" => Some(0),
        "playback.engine" => Some(1),
        "playback.status" => Some(2),
        _ => None,
    }
}
impl ReadProjection {
    pub(crate) fn set_native_range_draft(
        &mut self,
        seek: bool,
        value: &str,
    ) -> Result<(), PluginError> {
        if !self.native_admission.get() {
            return Err(PluginError::new(
                PluginErrorCode::MissingCapability,
                "Refresh original native observation before editing its range",
            ));
        }
        self.native
            .try_borrow_mut()
            .map_err(|_| PluginError::new(PluginErrorCode::Busy, "Native range borrowed"))?
            .set_range_draft(seek, value)?;
        self.dirty = true;
        Ok(())
    }
    pub(crate) fn set_native_draft(
        &mut self,
        seek: bool,
        value: String,
    ) -> Result<(), PluginError> {
        self.native
            .try_borrow_mut()
            .map_err(|_| PluginError::new(PluginErrorCode::Busy, "Native control draft borrowed"))?
            .set_draft(seek, value);
        self.dirty = true;
        Ok(())
    }
    pub fn retained_event(&self, ticket: &PluginTicket) -> Option<&PluginEvent> {
        self.entries
            .iter()
            .find(|entry| entry.ticket == *ticket)?
            .event
            .as_ref()
    }
    pub fn contains(&self, ticket: &PluginTicket) -> bool {
        self.entries.iter().any(|entry| entry.ticket == *ticket)
    }
    pub fn requested(&mut self, ticket: &PluginTicket) {
        if ticket.capability() == "playback.engine"
            && let Ok(mut catalog) = self.catalog.try_borrow_mut()
        {
            catalog.engine_queries = catalog.engine_queries.saturating_add(1);
        }
        if matches!(
            ticket.capability(),
            "playback.previous" | "playback.next" | "playback.transition.commit"
        ) {
            self.native_admission.set(false);
            if ticket.capability() == "playback.transition.commit"
                && let Ok(mut native) = self.native.try_borrow_mut()
            {
                native.unavailable("회차 교체 요청 후 원래 엔진 관측을 다시 확인하세요".into());
            }
        }
        if crate::assembly::meari::viewer_callbacks::catalog::is_command(ticket.capability()) {
            match self.catalog.try_borrow_mut() {
                Ok(mut catalog) => {
                    if let Err(error) = catalog.requested(ticket) {
                        self.catalog_projection_fault.set(true);
                        self.panel.projection_error = Some(error.message);
                    }
                }
                Err(_) => {
                    self.catalog_projection_fault.set(true);
                    self.panel.projection_error = Some(
                        "Original catalog projection borrowed; new catalog inputs closed".into(),
                    );
                }
            }
            self.dirty = true;
        }
        if ticket.capability() == "playback.engine" || is_native_control(ticket.capability()) {
            self.native_admission.set(false);
            match self.native.try_borrow_mut() {
                Ok(mut native) => native.requested(ticket),
                Err(_) => {
                    self.panel.projection_error =
                        Some("Native requested projection borrowed; controls closed".into())
                }
            }
            self.dirty = true;
        }
        self.entries
            .retain(|entry| entry.ticket.capability() != ticket.capability());
        if !self.entries.iter().any(|entry| entry.ticket == *ticket) {
            self.entries.push(Retained {
                ticket: ticket.clone(),
                event: None,
                owner_ack: None,
            });
        }
        if let Some(i) = index(ticket.capability()) {
            self.panel.rows[i].phase = ReadPhase::Pending;
            self.dirty = true;
        }
    }
    pub fn retain(&mut self, acceptance: TerminalAcceptance) {
        let ticket = &acceptance.event.ticket;
        // The pump already owns Core's accepted event before SDK ACK. Retain our copy before decoding.
        let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.ticket == *ticket)
        else {
            return;
        };
        entry.event = Some(acceptance.event.clone());
        entry.owner_ack = Some(acceptance.owner_ack.clone());
        if ticket.capability() == "capabilities" {
            if let Ok(mut catalog) = self.catalog.try_borrow_mut() {
                catalog.capabilities(&acceptance.event, &acceptance.owner_ack);
            } else {
                self.catalog_projection_fault.set(true);
            }
        }
        if crate::assembly::meari::viewer_callbacks::catalog::is_command(ticket.capability()) {
            match self.catalog.try_borrow_mut() {
                Ok(mut catalog) => catalog.retain(&acceptance.event, &acceptance.owner_ack),
                Err(_) => {
                    self.catalog_projection_fault.set(true);
                    self.panel.projection_error =
                        Some("Original catalog terminal retained; projection borrowed".into());
                }
            }
        }
        self.retain_native(
            &acceptance.event,
            acceptance.owner_ack.as_ref().map_err(|e| e.message.clone()),
        );
        if self
            .catalog
            .try_borrow()
            .map(|catalog| catalog.transition_blocks_controls())
            .unwrap_or(true)
        {
            self.native_admission.set(false);
        }
        self.dirty = true;
        let Some(i) = index(ticket.capability()) else {
            return;
        };
        let row = &mut self.panel.rows[i];
        match decode(&acceptance.event) {
            Ok(value) => {
                row.value = Some(value);
                row.diagnostic = None;
                row.error_key = None;
                row.phase = ReadPhase::Received;
            }
            Err(error) => {
                row.diagnostic = Some(error);
                row.error_key = Some(
                    if matches!(acceptance.event.event, PluginEventKind::Failed(_)) {
                        "viewer.read.sdk_failed"
                    } else {
                        "viewer.read.decode_failed"
                    },
                );
                row.phase = ReadPhase::Failed;
            }
        }
        if let Err(error) = acceptance.owner_ack {
            row.phase = ReadPhase::AckFailed;
            row.error_key = Some("viewer.read.ack_failed");
            row.diagnostic = Some(error.message);
        }
        self.dirty = true;
    }
    pub fn fault(&mut self, ticket: &PluginTicket, error: &PluginError) {
        if crate::assembly::meari::viewer_callbacks::catalog::is_command(ticket.capability()) {
            self.dirty = true;
            if let Ok(mut catalog) = self.catalog.try_borrow_mut() {
                catalog.fault(error);
            } else {
                self.catalog_projection_fault.set(true);
            }
        }
        if ticket.capability() == "playback.engine" || is_native_control(ticket.capability()) {
            self.native_admission.set(false);
            if let Ok(mut native) = self.native.try_borrow_mut() {
                if ticket.capability() == "playback.engine" {
                    native.unavailable(error.message.clone());
                }
                native.control_failed(error.message.clone(), true);
            }
        }
        if let Some(i) = index(ticket.capability()) {
            self.panel.rows[i].diagnostic = Some(error.message.clone());
            self.panel.rows[i].error_key = Some("viewer.read.consumer_failed");
            self.panel.rows[i].phase = ReadPhase::Failed;
            self.dirty = true;
        }
    }
    pub fn projection_failed(&mut self, error: String) {
        self.panel.projection_error = Some(error);
    }
    /// Re-decode retained envelopes only. SDK acceptance/ACK is not called again.
    pub fn retry_decode(&mut self) {
        if let Ok(mut native) = self.native.try_borrow_mut() {
            for entry in &self.entries {
                if is_native_control(entry.ticket.capability())
                    && entry.owner_ack.as_ref().is_some_and(Result::is_ok)
                    && entry.event.as_ref().is_some_and(|event| {
                        matches!(
                            event.event,
                            PluginEventKind::Completed(_) | PluginEventKind::Failed(_)
                        )
                    })
                {
                    native.retire_accepted_terminal(&entry.ticket);
                }
            }
        } else {
            self.native_admission.set(false);
            self.panel.projection_error =
                Some("Native terminal reprojection borrowed; controls closed".into());
        }
        for entry in &self.entries {
            let Some(event) = &entry.event else {
                continue;
            };
            let Some(i) = index(entry.ticket.capability()) else {
                continue;
            };
            let row = &mut self.panel.rows[i];
            match decode(event) {
                Ok(value) => {
                    row.value = Some(value);
                    row.diagnostic = None;
                    row.error_key = None;
                    row.phase = ReadPhase::Received;
                }
                Err(error) => {
                    row.diagnostic = Some(error);
                    row.error_key = Some(if matches!(event.event, PluginEventKind::Failed(_)) {
                        "viewer.read.sdk_failed"
                    } else {
                        "viewer.read.decode_failed"
                    });
                    row.phase = ReadPhase::Failed;
                }
            }
            if let Some(Err(error)) = &entry.owner_ack {
                row.phase = ReadPhase::AckFailed;
                row.error_key = Some("viewer.read.ack_failed");
                row.diagnostic = Some(error.message.clone());
            }
        }
        self.dirty = true;
    }
    pub fn committed(&mut self) {
        self.dirty = false;
        self.panel.projection_error = None;
    }
    pub fn props(
        &self,
    ) -> Result<std::collections::BTreeMap<String, meari::domain::Value>, PluginError> {
        let native = self.native.try_borrow().map_err(|_| {
            PluginError::new(PluginErrorCode::Busy, "Native playback projection borrowed")
        })?;
        let mut props = self.panel.props();
        props.insert(
            "diagnosticsExpanded".into(),
            meari::domain::Value::Bool(self.diagnostics_expanded),
        );
        props.extend(self.window_ui.props());
        props.extend(native.props());
        let catalog = self.catalog.try_borrow().map_err(|_| {
            PluginError::new(
                PluginErrorCode::Busy,
                "Original catalog projection borrowed",
            )
        })?;
        props.extend(catalog.props()?);
        props.extend(catalog.episode_identity_props(native.observation()));
        props.extend(catalog.transition_props(
            native.settled_observation(),
            self.automatic_enabled && !self.catalog_projection_fault.get(),
            self.native_admission.get(),
        ));
        props.insert(
            "automaticAvailable".into(),
            meari::domain::Value::Bool(self.automatic_enabled),
        );
        props.insert(
            "automaticError".into(),
            meari::domain::Value::String(
                if self.automatic_error.is_empty() && catalog.engine_queries >= 120 {
                    "재생 상태 확인이 지연되고 있습니다. 재생 상태를 다시 확인하세요".into()
                } else {
                    self.automatic_error.clone()
                },
            ),
        );
        if !self.automatic_enabled {
            props.insert("catalogBusy".into(), meari::domain::Value::Bool(true));
        }
        if self.catalog_projection_fault.get() {
            props.insert("catalogBusy".into(), meari::domain::Value::Bool(true));
        }
        let (title, episode) = native
            .observation()
            .and_then(|observation| catalog.current_titles(observation))
            .map(|(title, episode)| (title.to_owned(), episode.to_owned()))
            .unwrap_or_default();
        props.insert(
            "nativeCurrentTitle".into(),
            meari::domain::Value::String(title),
        );
        props.insert(
            "nativeCurrentEpisode".into(),
            meari::domain::Value::String(episode.clone()),
        );
        let caption = native
            .observation()
            .and_then(|o| catalog.current_titles(o))
            .map(|(title, episode)| format!("{title} · {episode}"))
            .unwrap_or_default();
        props.insert(
            "nativeCurrentCaption".into(),
            meari::domain::Value::String(caption),
        );
        if !self.native_admission.get() {
            props.insert(
                "nativeControlsDisabled".into(),
                meari::domain::Value::Bool(true),
            );
            props.insert(
                "nativeSeekRangeDisabled".into(),
                meari::domain::Value::Bool(true),
            );
            props.insert(
                "nativeVolumeRangeDisabled".into(),
                meari::domain::Value::Bool(true),
            );
        }
        Ok(props)
    }
    fn retain_native(&mut self, event: &PluginEvent, ack: Result<&(), String>) {
        let command = event.ticket.capability();
        if command != "playback.engine" && !is_native_control(command) {
            return;
        }
        let acked_terminal = ack.is_ok()
            && matches!(
                event.event,
                PluginEventKind::Completed(_) | PluginEventKind::Failed(_)
            );
        let native_uncertain = ack.is_err();
        let result = (|| {
            ack?;
            let payload = match &event.event {
                PluginEventKind::Completed(payload) => payload,
                PluginEventKind::Failed(error) => {
                    return Err(crate::assembly::meari::viewer_callbacks::codec::sdk_error(
                        command, error,
                    )
                    .map(|sdk| format!("{:?}: {}", sdk.code, sdk.message))
                    .unwrap_or_else(|| error.message.clone()));
                }
                _ => return Err("Unexpected native nonterminal event".into()),
            };
            let manifest = crate::assembly::meari::viewer_callbacks::codec::manifest();
            let descriptor = manifest
                .capabilities
                .get(command)
                .ok_or("Unknown native capability")?;
            let mut native = self
                .native
                .try_borrow_mut()
                .map_err(|_| "Native playback projection borrowed")?;
            if command == "playback.engine" {
                let observation = payload
                    .decode::<EpisodePlaybackObservation>(&descriptor.reply_schema)
                    .map_err(|e| e.message)?;
                if let Ok(mut catalog) = self.catalog.try_borrow_mut()
                    && catalog.expected_engine_matches(&observation)
                {
                    catalog.engine_queries = 0;
                }
                native.observe(observation);
                self.native_admission.set(true);
            } else {
                let receipt = payload
                    .decode::<crate::application::playback_control::ControlReceipt>(
                        &descriptor.reply_schema,
                    )
                    .map_err(|e| e.message)?;
                native.receipt(&event.ticket, receipt)?;
            }
            Ok::<(), String>(())
        })();
        if let Err(error) = result {
            self.native_admission.set(false);
            if let Ok(mut native) = self.native.try_borrow_mut() {
                if command == "playback.engine" {
                    native.unavailable(error.clone());
                }
                native.control_failed(error, native_uncertain);
            }
        }
        if acked_terminal && is_native_control(command) {
            match self.native.try_borrow_mut() {
                Ok(mut native) => native.retire_accepted_terminal(&event.ticket),
                Err(_) => {
                    self.native_admission.set(false);
                    self.panel.projection_error = Some(
                        "Native terminal projection borrowed; original ticket retained".into(),
                    );
                }
            }
        }
    }
}
fn decode(event: &PluginEvent) -> Result<String, String> {
    let payload = match &event.event {
        PluginEventKind::Completed(payload) => payload,
        PluginEventKind::Failed(error) => return Err(error.message.clone()),
        _ => return Err("Unexpected nonterminal event".into()),
    };
    // Decode with the original schema; JSON here is display data, never executable UI source.
    let manifest = crate::assembly::meari::viewer_callbacks::codec::manifest();
    let descriptor = manifest
        .capabilities
        .get(event.ticket.capability())
        .ok_or("Unknown read capability")?;
    match event.ticket.capability() {
        "capabilities" => {
            let dto: CapabilitiesReply = payload
                .decode(&descriptor.reply_schema)
                .map_err(|e| e.message)?;
            serde_json::to_string(&dto).map_err(|e| e.to_string())
        }
        "playback.engine" => {
            let dto: EpisodePlaybackObservation = payload
                .decode(&descriptor.reply_schema)
                .map_err(|e| e.message)?;
            serde_json::to_string(&dto).map_err(|e| e.to_string())
        }
        "playback.status" => {
            let dto: PlaybackStatus = payload
                .decode(&descriptor.reply_schema)
                .map_err(|e| e.message)?;
            serde_json::to_string(&dto).map_err(|e| e.to_string())
        }
        _ => Err("Unknown read capability".into()),
    }
}
#[cfg(test)]
mod tests;
