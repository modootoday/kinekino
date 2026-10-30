//! Original-window effects only. Media policy stays in the common SDK.
use super::{ViewerCallbacks, ViewerHost};
use meari::domain::Value;
use meari_native::{
    application::window_actions::*,
    infrastructure::{
        native_host::{HostTurn, HostTurnReason},
        window::{NativeHostTimer, NativeHostTimerToken, NativeWindowActions, NativeWindowWake},
    },
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    num::NonZeroU64,
    rc::Rc,
    time::{Duration, Instant},
};

pub(crate) struct WindowUiProjection {
    pub(crate) caption_requested: Option<CaptionIntent>,
    caption_minimize_available: bool,
    caption_maximize_available: bool,
    caption_close_available: bool,
    caption_pending: bool,
    caption_status: String,
    maximized: Option<bool>,
    pub(crate) episodes_open: bool,
    pub(crate) toggle_requested: bool,
    fullscreen: bool,
    available: bool,
    pending: bool,
    controls_hidden: bool,
    error: String,
}
impl Default for WindowUiProjection {
    fn default() -> Self {
        Self {
            caption_requested: None,
            caption_minimize_available: false,
            caption_maximize_available: false,
            caption_close_available: false,
            caption_pending: false,
            caption_status: String::new(),
            maximized: None,
            episodes_open: true,
            toggle_requested: false,
            fullscreen: false,
            available: false,
            pending: false,
            controls_hidden: false,
            error: String::new(),
        }
    }
}
impl WindowUiProjection {
    pub(super) fn caption_view(&self) -> CaptionView {
        CaptionView {
            minimize_disabled: !self.caption_minimize_available || self.caption_pending,
            maximize_disabled: !self.caption_maximize_available || self.caption_pending,
            close_disabled: !self.caption_close_available,
            maximize_label: match self.maximized {
                Some(true) => "복원",
                Some(false) => "최대화",
                None => "최대화 상태 미확인",
            }
            .into(),
            maximize_icon: if self.maximized == Some(true) {
                "aw-icon aw-icon-restore"
            } else {
                "aw-icon aw-icon-maximize"
            }
            .into(),
            status: if self.error.is_empty() {
                self.caption_status.clone()
            } else {
                format!("창 제어 오류 · {}", self.error)
            },
            error: !self.error.is_empty(),
        }
    }
    pub(crate) fn user_close_vetoed(&mut self) {
        self.caption_status = "원 작업의 사용자 닫기 보호 조건으로 창을 유지했습니다.".into();
    }
    fn observe_fullscreen(&mut self, observed: Option<bool>) {
        self.fullscreen = observed == Some(true);
    }
    pub(crate) fn props(&self) -> BTreeMap<String, Value> {
        let mut props = BTreeMap::from([
            ("viewingFullscreen".into(), Value::Bool(self.fullscreen)),
            (
                "viewingFrameClass".into(),
                Value::String(
                    if self.fullscreen {
                        "aw-player-shell aw-player-fullscreen"
                    } else {
                        "aw-player-shell"
                    }
                    .into(),
                ),
            ),
            (
                "viewingRootClass".into(),
                Value::String(
                    if self.episodes_open {
                        "aw-viewing-read-root"
                    } else {
                        "aw-viewing-read-root aw-viewing-panel-closed"
                    }
                    .into(),
                ),
            ),
            (
                "episodePanelClosed".into(),
                Value::Bool(!self.episodes_open),
            ),
            (
                "transportClass".into(),
                Value::String(
                    if self.controls_hidden {
                        "aw-transport aw-controls-hidden"
                    } else {
                        "aw-transport"
                    }
                    .into(),
                ),
            ),
            (
                "episodePanelClass".into(),
                Value::String(
                    if self.episodes_open {
                        "aw-episode-panel"
                    } else {
                        "aw-episode-panel aw-episodes-closed"
                    }
                    .into(),
                ),
            ),
            (
                "fullscreenDisabled".into(),
                Value::Bool(!self.available || self.pending),
            ),
            (
                "fullscreenLabel".into(),
                Value::String(
                    if self.fullscreen {
                        "전체화면 해제"
                    } else {
                        "전체화면"
                    }
                    .into(),
                ),
            ),
            ("windowError".into(), Value::String(self.error.clone())),
        ]);
        props.extend(self.caption_view().props());
        props
    }
}
type ActionsSource = Rc<RefCell<Option<NativeWindowActions>>>;
type WakeSource = Rc<RefCell<Option<NativeWindowWake>>>;
pub(crate) use crate::presentation::web_ui_adapter::player_shell::window_caption::CaptionIntent;
use crate::presentation::web_ui_adapter::player_shell::window_caption::{
    CaptionState, CaptionView,
};
#[derive(Default)]
pub(super) struct WindowUiHost {
    pub(super) captions: Rc<RefCell<CaptionState>>,
    document: Option<meari_native::infrastructure::native_host::NativeDocumentHandle>,
    source: Option<ActionsSource>,
    wake: Option<WakeSource>,
    scope: Option<ActionScope>,
    next_id: u64,
    fullscreen_request: Option<NonZeroU64>,
    caption_request: Option<(NonZeroU64, WindowAction)>,
    cursor_request: Option<(NonZeroU64, bool)>,
    cursor_applied: Option<bool>,
    desired_cursor: bool,
    activity_sequence: u64,
    keyboard_mode: bool,
    focused: bool,
    inside: bool,
    last_activity: Option<Instant>,
    timer: Option<NativeHostTimer>,
    armed: Option<(NativeHostTimerToken, Instant)>,
    stopped: bool,
    pending_since: Option<Instant>,
}
impl WindowUiHost {
    pub(super) fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        if let Ok(mut captions) = self.captions.try_borrow_mut() {
            captions.close();
        }
        self.cancel_timer();
        self.desired_cursor = true;
        // Close may already reject requests; the library's revoke cleanup is the final restore.
        let port = self
            .source
            .as_ref()
            .and_then(|source| source.try_borrow().ok().and_then(|source| source.clone()));
        if let Some(port) = port
            && let Ok(snapshot) = port.snapshot()
        {
            let _restore =
                self.request(&port, snapshot.scope, WindowAction::SetCursorVisible(true));
        }
    }
    fn request(
        &mut self,
        port: &NativeWindowActions,
        scope: ActionScope,
        action: WindowAction,
    ) -> Result<NonZeroU64, String> {
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or("Window request counter exhausted")?;
        let id = NonZeroU64::new(self.next_id).ok_or("Invalid window request counter")?;
        port.request(scope, id, action).map_err(|e| e.to_string())?;
        Ok(id)
    }
    fn cancel_timer(&mut self) {
        self.armed = None;
        self.timer = None; // Original timer Drop cancels only this owner.
    }
    pub(super) fn advance(
        &mut self,
        owner: &ViewerCallbacks,
        viewing: bool,
        turn: &HostTurn,
        document: Option<meari_native::infrastructure::native_host::NativeDocumentHandle>,
    ) {
        self.document = document;
        self.desired_cursor = true;
        let result = self.drive(owner, viewing, turn);
        if let Err(error) = result {
            self.stop();
            if let Ok(mut projection) = owner.projection.try_borrow_mut() {
                projection.window_ui.error = error;
                projection.window_ui.available = false;
                projection.window_ui.caption_minimize_available = false;
                projection.window_ui.caption_maximize_available = false;
                projection.window_ui.caption_close_available = false;
                projection.window_ui.controls_hidden = false;
                if let Ok(mut captions) = self.captions.try_borrow_mut() {
                    captions.view = projection.window_ui.caption_view();
                }
                projection.dirty = true;
            }
        }
    }
    fn drive(
        &mut self,
        owner: &ViewerCallbacks,
        viewing: bool,
        turn: &HostTurn,
    ) -> Result<(), String> {
        let Some(source) = &self.source else {
            return Ok(());
        };
        let Some(port) = source
            .try_borrow()
            .map_err(|_| "Window actions source borrowed")?
            .clone()
        else {
            return Ok(());
        };
        let snapshot = match port.snapshot() {
            Ok(snapshot) => snapshot,
            Err(ActionError::NotReady) => return Ok(()),
            Err(error) => return Err(error.to_string()),
        };
        if snapshot.scope.host != turn.stamp.host_id {
            return Err("Foreign window action scope".into());
        }
        if self.scope.is_some_and(|old| old != snapshot.scope) {
            self.cancel_timer();
            return Err("Original window action scope replaced".into());
        }
        self.scope = Some(snapshot.scope);
        let mut projection = owner
            .projection
            .try_borrow_mut()
            .map_err(|_| "Window projection borrowed")?;
        let playing = projection
            .native
            .try_borrow()
            .is_ok_and(|native| native.permits_controls_idle());
        let blocking_notice = projection
            .catalog
            .try_borrow()
            .map(|catalog| catalog.has_blocking_notice())
            .unwrap_or(true)
            || projection.panel.projection_error.is_some();
        let menu_open = projection.diagnostics_expanded;
        let interaction = self
            .document
            .as_ref()
            .filter(|document| {
                document.lifecycle()
                    == meari_native::infrastructure::native_host::HostLifecycle::Open
                    && document.stamp().ok().as_ref() == Some(&turn.stamp)
            })
            .and_then(|document| {
                document
                    .try_with(|document| {
                        (
                            document.has_active_range_gesture(),
                            document.renderer.document.get_focussed_node_id().is_some(),
                        )
                    })
                    .ok()
            });
        let ui = &mut projection.window_ui;
        let before = ui.props();
        ui.observe_fullscreen(snapshot.observed.borderless_monitor_coverage);
        ui.maximized = snapshot.observed.maximized;
        ui.caption_minimize_available = !self.stopped
            && snapshot.capabilities.minimize
            && snapshot.observed.minimized.is_some();
        ui.caption_maximize_available = !self.stopped
            && snapshot.capabilities.maximize
            && ui.maximized.is_some()
            && !ui.fullscreen;
        ui.caption_close_available = !self.stopped
            && self
                .wake
                .as_ref()
                .is_some_and(|wake| wake.try_borrow().is_ok_and(|wake| wake.is_some()));
        if let Some((id, action)) = self.caption_request
            && let Some(record) = port.take_result(id).map_err(|error| error.to_string())?
        {
            if record.ticket.scope != snapshot.scope || record.action != action {
                return Err("Original caption request scope/action differs".into());
            }
            self.caption_request = None;
            match record.status {
                ActionStatus::Observed
                    if record
                        .observation
                        .is_some_and(|observation| observation.matches(action)) =>
                {
                    ui.caption_status = "원 창 상태 변경을 관측했습니다.".into()
                }
                ActionStatus::Failed(error) => {
                    ui.error = error.to_string();
                    ui.caption_status = "창 상태 변경에 실패했습니다.".into();
                }
                _ => ui.caption_status = "창 요청 결과가 아직 확인되지 않았습니다.".into(),
            }
        }
        let caption_intent = ui.caption_requested.take().or_else(|| {
            self.captions
                .try_borrow_mut()
                .ok()
                .and_then(|mut captions| captions.take())
        });
        if let Some(intent) = caption_intent {
            match intent {
                CaptionIntent::Minimize
                    if ui.caption_minimize_available && self.caption_request.is_none() =>
                {
                    let action = WindowAction::SetMinimized(true);
                    self.caption_request =
                        Some((self.request(&port, snapshot.scope, action)?, action));
                    ui.caption_status =
                        "최소화를 요청했습니다. 실제 창 상태를 확인 중입니다.".into();
                }
                CaptionIntent::ToggleMaximize
                    if ui.caption_maximize_available && self.caption_request.is_none() =>
                {
                    let action = WindowAction::SetMaximized(ui.maximized != Some(true));
                    self.caption_request =
                        Some((self.request(&port, snapshot.scope, action)?, action));
                    ui.caption_status =
                        "최대화 또는 복원을 요청했습니다. 실제 창 상태를 확인 중입니다.".into();
                }
                CaptionIntent::UserClose if ui.caption_close_available => {
                    let wake = self
                        .wake
                        .as_ref()
                        .ok_or("Original close wake unavailable")?
                        .try_borrow()
                        .map_err(|_| "Original close wake borrowed")?
                        .clone()
                        .ok_or("Original close wake unavailable")?;
                    match wake.request_user_close(snapshot.scope) {
                        Ok(_) => ui.caption_status = "사용자 닫기 요청을 전달했습니다. 종료/보호 조건 판정은 아직 확인되지 않았습니다.".into(),
                        Err(error) => { ui.error = error.to_string(); ui.caption_status = "사용자 닫기 요청을 전달하지 못했습니다.".into(); },
                    }
                }
                _ => {
                    ui.error =
                        "원 창 기능이 준비되지 않았거나 이전 창 요청이 처리 중입니다.".into();
                }
            }
        }
        ui.caption_pending = self.caption_request.is_some();
        ui.available = viewing
            && !self.stopped
            && snapshot.capabilities.borderless_fullscreen
            && snapshot.observed.borderless_monitor_coverage.is_some();
        if let Some(id) = self.fullscreen_request
            && let Some(record) = port.take_result(id).map_err(|e| e.to_string())?
        {
            self.fullscreen_request = None;
            if let ActionStatus::Failed(error) = record.status {
                ui.error = error.to_string();
            }
        }
        if let Some((id, value)) = self.cursor_request
            && let Some(record) = port.take_result(id).map_err(|e| e.to_string())?
        {
            self.cursor_request = None;
            match record.status {
                ActionStatus::Applied => self.cursor_applied = Some(value),
                ActionStatus::Failed(error) => {
                    ui.error = error.to_string();
                    ui.controls_hidden = false;
                }
                _ => (),
            }
        }
        let now = Instant::now();
        if let Some(activity) = port.input_activity().map_err(|e| e.to_string())?
            && activity.scope == snapshot.scope
            && activity.sequence > self.activity_sequence
        {
            self.activity_sequence = activity.sequence;
            match activity.kind {
                WindowInputKind::Key => self.keyboard_mode = true,
                WindowInputKind::PointerButton | WindowInputKind::FocusLost => {
                    self.keyboard_mode = false
                }
                _ => (),
            }
            self.focused = activity.focused == Some(true);
            self.inside = activity.pointer_inside == Some(true);
            self.last_activity = Some(now);
            ui.controls_hidden = false;
            self.desired_cursor = true;
        }
        if ui.toggle_requested {
            ui.toggle_requested = false;
            if viewing && !self.stopped && self.fullscreen_request.is_none() {
                self.fullscreen_request = Some(self.request(
                    &port,
                    snapshot.scope,
                    WindowAction::SetFullscreen(!ui.fullscreen),
                )?);
                self.last_activity = Some(now);
                ui.error.clear();
            }
        }
        if !viewing && ui.fullscreen && self.fullscreen_request.is_none() && !self.stopped {
            self.fullscreen_request =
                Some(self.request(&port, snapshot.scope, WindowAction::SetFullscreen(false))?);
        }
        let idle_allowed = can_idle(
            viewing,
            self.stopped,
            ui.fullscreen,
            self.focused,
            self.inside,
            ui.episodes_open,
            IdlePins {
                playing,
                error: blocking_notice || !ui.error.is_empty(),
                menu_open,
                interaction_known: interaction.is_some(),
                dragging: interaction.is_some_and(|state| state.0),
                keyboard_focus: self.keyboard_mode && interaction.is_some_and(|state| state.1),
            },
        );
        if !idle_allowed {
            ui.controls_hidden = false;
            self.desired_cursor = true;
        } else if idle_due(self.last_activity, now) {
            ui.controls_hidden = true;
            self.desired_cursor = false;
        }
        if self.cursor_request.is_none() && self.cursor_applied != Some(self.desired_cursor) {
            let value = self.desired_cursor;
            self.cursor_request = Some((
                self.request(&port, snapshot.scope, WindowAction::SetCursorVisible(value))?,
                value,
            ));
        }
        ui.pending = self.fullscreen_request.is_some();
        if let Ok(mut captions) = self.captions.try_borrow_mut() {
            captions.view = ui.caption_view();
        }
        if before != ui.props() {
            projection.dirty = true;
        }
        let pending = self.fullscreen_request.is_some()
            || self.cursor_request.is_some()
            || self.caption_request.is_some();
        if pending {
            let since = self.pending_since.get_or_insert(now);
            if now.saturating_duration_since(*since) >= Duration::from_secs(5) {
                return Err("Window request remains unconfirmed; original request retained, no automatic retry".into());
            }
        } else {
            self.pending_since = None;
        }
        let deadline = next_deadline(
            pending,
            idle_allowed,
            projection.window_ui.controls_hidden,
            self.last_activity,
            now,
        );
        drop(projection);
        self.reconcile_timer(turn, deadline, now)
    }
    fn reconcile_timer(
        &mut self,
        turn: &HostTurn,
        deadline: Option<Instant>,
        now: Instant,
    ) -> Result<(), String> {
        if turn.reason == HostTurnReason::Completion
            && self.armed.as_ref().is_some_and(|(_, due)| now >= *due)
            && let Some((token, _)) = self.armed.take()
            && let Some(timer) = &self.timer
        {
            timer.cancel(&token).map_err(|e| e.to_string())?;
        }
        let Some(deadline) = deadline else {
            self.cancel_timer();
            return Ok(());
        };
        // Keep an earlier arm; unrelated SDK turns cannot postpone an existing poll/deadline.
        if self
            .armed
            .as_ref()
            .is_some_and(|(_, old)| keep_earlier_deadline(*old, deadline, now))
        {
            return Ok(());
        }
        if self.timer.is_none() {
            let document = self.document(turn)?;
            let wake = self
                .wake
                .as_ref()
                .ok_or("Window wake unavailable")?
                .try_borrow()
                .map_err(|_| "Window wake borrowed")?
                .clone()
                .ok_or("Window not attached")?;
            self.timer = Some(wake.timer(&document).map_err(|e| e.to_string())?);
        }
        let token = self
            .timer
            .as_ref()
            .ok_or("Window timer unavailable")?
            .schedule_after(
                deadline
                    .saturating_duration_since(now)
                    .max(Duration::from_millis(1)),
            )
            .map_err(|e| e.to_string())?;
        self.armed = Some((token, deadline));
        Ok(())
    }
    fn document(
        &self,
        _turn: &HostTurn,
    ) -> Result<meari_native::infrastructure::native_host::NativeDocumentHandle, String> {
        self.document
            .clone()
            .ok_or("Original window document unavailable".into())
    }
}
fn idle_due(last: Option<Instant>, now: Instant) -> bool {
    last.is_some_and(|last| now.saturating_duration_since(last) >= Duration::from_secs(3))
}
fn can_idle(
    viewing: bool,
    stopped: bool,
    fullscreen: bool,
    focused: bool,
    inside: bool,
    panel_open: bool,
    pins: IdlePins,
) -> bool {
    viewing
        && !stopped
        && fullscreen
        && focused
        && inside
        && !panel_open
        && pins.playing
        && pins.interaction_known
        && !pins.error
        && !pins.dragging
        && !pins.menu_open
        && !pins.keyboard_focus
}
#[derive(Clone, Copy, Default)]
struct IdlePins {
    playing: bool,
    interaction_known: bool,
    error: bool,
    dragging: bool,
    menu_open: bool,
    keyboard_focus: bool,
}
fn next_deadline(
    pending: bool,
    idle_allowed: bool,
    hidden: bool,
    last: Option<Instant>,
    now: Instant,
) -> Option<Instant> {
    if pending {
        Some(now + Duration::from_millis(50))
    } else if idle_allowed && !hidden {
        last.map(|last| last + Duration::from_secs(3))
    } else {
        None
    }
}
fn keep_earlier_deadline(armed: Instant, requested: Instant, now: Instant) -> bool {
    armed <= requested && armed > now
}
impl ViewerHost {
    pub(crate) fn bind_viewing_window_actions(&mut self, source: ActionsSource, wake: WakeSource) {
        self.window_ui.source = Some(source);
        self.window_ui.wake = Some(wake);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn viewing_caption_props_use_the_same_observed_view_as_studio() {
        let mut projection = WindowUiProjection::default();
        for maximized in [None, Some(false), Some(true)] {
            projection.maximized = maximized;
            projection.caption_maximize_available = maximized.is_some();
            let view = projection.caption_view();
            for (key, value) in view.props() {
                assert_eq!(projection.props().get(&key), Some(&value));
            }
            assert_eq!(view.maximize_disabled, maximized.is_none());
            assert_eq!(
                view.maximize_icon.contains("restore"),
                maximized == Some(true)
            );
        }
    }
    #[test]
    fn fullscreen_frame_follows_only_original_observation_not_requested_intent() {
        let mut projection = WindowUiProjection {
            toggle_requested: true,
            ..Default::default()
        };
        assert_eq!(projection.props()["viewingFullscreen"], Value::Bool(false));
        projection.observe_fullscreen(Some(true));
        assert_eq!(projection.props()["viewingFullscreen"], Value::Bool(true));
        assert_eq!(
            projection.props()["viewingFrameClass"],
            Value::String("aw-player-shell aw-player-fullscreen".into())
        );
        projection.observe_fullscreen(Some(false));
        assert_eq!(projection.props()["viewingFullscreen"], Value::Bool(false));
        projection.observe_fullscreen(None);
        assert_eq!(
            projection.props()["viewingFrameClass"],
            Value::String("aw-player-shell".into())
        );
    }
    #[test]
    fn activity_rearms_idle_deadline_and_old_completion_cannot_hide_early() {
        let start = Instant::now();
        assert!(!idle_due(None, start + Duration::from_secs(10)));
        assert!(!idle_due(Some(start), start + Duration::from_millis(2999)));
        assert!(idle_due(Some(start), start + Duration::from_secs(3)));
        let new_input = start + Duration::from_millis(2900);
        assert!(!idle_due(Some(new_input), start + Duration::from_secs(3)));
        assert!(idle_due(
            Some(new_input),
            new_input + Duration::from_secs(3)
        ));
    }
    #[test]
    fn close_is_idempotent_cancels_owned_timer_and_prevents_late_intents() {
        let mut host = WindowUiHost::default();
        host.stop();
        host.stop();
        assert!(host.stopped);
        assert!(host.desired_cursor);
        assert!(host.timer.is_none());
        assert!(host.armed.is_none());
        // The host's production admission rejects callbacks after shutdown_started.
        // No native cursor/OS observation is fabricated by this lifecycle model case.
    }
    #[test]
    fn idle_is_blocked_after_focus_loss_route_leave_close_or_panel_open() {
        let pins = IdlePins {
            playing: true,
            interaction_known: true,
            ..Default::default()
        };
        assert!(can_idle(true, false, true, true, true, false, pins));
        assert!(!can_idle(false, false, true, true, true, false, pins));
        assert!(!can_idle(true, true, true, true, true, false, pins));
        assert!(!can_idle(true, false, false, true, true, false, pins));
        assert!(!can_idle(true, false, true, false, true, false, pins));
        assert!(!can_idle(true, false, true, true, false, false, pins));
        assert!(!can_idle(true, false, true, true, true, true, pins));
    }
    #[test]
    fn pause_error_drag_menu_keyboard_focus_or_unknown_interaction_keeps_controls_visible() {
        let allowed = IdlePins {
            playing: true,
            interaction_known: true,
            ..Default::default()
        };
        for pins in [
            IdlePins {
                playing: false,
                ..allowed
            },
            IdlePins {
                interaction_known: false,
                ..allowed
            },
            IdlePins {
                error: true,
                ..allowed
            },
            IdlePins {
                dragging: true,
                ..allowed
            },
            IdlePins {
                menu_open: true,
                ..allowed
            },
            IdlePins {
                keyboard_focus: true,
                ..allowed
            },
        ] {
            assert!(!can_idle(true, false, true, true, true, false, pins));
        }
    }
    #[test]
    fn idle_pin_preserves_pending_poll_and_never_postpones_an_original_earlier_deadline() {
        let now = Instant::now();
        assert_eq!(next_deadline(false, false, false, Some(now), now), None);
        assert_eq!(
            next_deadline(true, false, false, Some(now), now),
            Some(now + Duration::from_millis(50))
        );
        let original = now + Duration::from_millis(10);
        assert!(keep_earlier_deadline(
            original,
            now + Duration::from_millis(50),
            now
        ));
        assert!(keep_earlier_deadline(
            original,
            now + Duration::from_secs(3),
            now
        ));
        assert!(!keep_earlier_deadline(
            original,
            now + Duration::from_millis(5),
            now
        ));
        assert!(!keep_earlier_deadline(
            original,
            now + Duration::from_secs(3),
            original
        ));
    }
}
