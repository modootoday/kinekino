use super::*;
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(super) struct CreationStyleTrace {
    pub(super) hwnd: isize,
    pub(super) parent: isize,
    pub(super) generation: u64,
    pub(super) owner_thread: u32,
    pub(super) current_thread: u32,
    pub(super) requested: u32,
    pub(super) observed: u32,
    pub(super) installed: bool,
    pub(super) destroyed: bool,
    pub(super) detached: bool,
    pub(super) callback_error: u32,
}
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct StyleTrace {
    changing: Option<(isize, u32, u32)>,
    changed: Option<(isize, u32, u32)>,
    messages: u32,
    creation: Option<CreationStyleTrace>,
    fresh_extended_readback: Option<u32>,
}
#[cfg(test)]
thread_local! { static STYLE_TRACE: std::cell::Cell<StyleTrace> = const { std::cell::Cell::new(StyleTrace {
changing:None,changed:None,messages:0,creation:None,fresh_extended_readback:None }) }; }
#[cfg(test)]
fn update_trace(update: impl FnOnce(&mut StyleTrace)) {
    if STYLE_TRACE
        .try_with(|cell| {
            let mut trace = cell.get();
            update(&mut trace);
            cell.set(trace);
        })
        .is_err()
    {
        let _ = events::trace_unavailable();
    }
}
#[cfg(test)]
pub(super) fn record_style_message(hwnd: HWND, message: u32, entry: u32, delegated: u32) {
    update_trace(|trace| {
        trace.messages = trace.messages.saturating_add(1);
        let target = if message == WM_STYLECHANGING {
            &mut trace.changing
        } else {
            &mut trace.changed
        };
        target.get_or_insert((hwnd.0 as isize, entry, delegated));
    });
}
#[cfg(test)]
pub(super) fn record_creation(creation: CreationStyleTrace) {
    update_trace(|trace| {
        trace.creation.get_or_insert(creation);
    });
}
#[cfg(test)]
pub(super) fn record_style_readback(extended: u32) {
    update_trace(|trace| {
        trace.fresh_extended_readback = Some(extended);
    });
}
#[cfg(test)]
pub(super) fn style_trace() -> Result<StyleTrace> {
    STYLE_TRACE
        .try_with(|cell| cell.get())
        .map_err(|_| events::trace_unavailable())
}
