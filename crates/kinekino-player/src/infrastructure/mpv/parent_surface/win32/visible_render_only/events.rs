use super::*;
use std::sync::atomic::AtomicBool;
// Conservative process-wide quarantine: even TLS teardown must leave a sticky failure.
// No reset and no authority is granted by this bit. Cleanup still delegates to the original owner.
static OBSERVER_UNAVAILABLE: AtomicBool = AtomicBool::new(false);
thread_local! {
    // Weak callback observations only, never window/engine authority or a borrowed owner pointer.
    static INPUT_OBSERVERS: RefCell<BTreeMap<isize, (u64, Weak<AtomicU32>)>> = const { RefCell::new(BTreeMap::new()) };
}

fn unavailable() -> VisibleError {
    OBSERVER_UNAVAILABLE.store(true, Ordering::Release);
    VisibleError::ObserverUnavailable
}
pub(super) fn require_available() -> Result<()> {
    if OBSERVER_UNAVAILABLE.load(Ordering::Acquire) {
        Err(VisibleError::ObserverUnavailable)
    } else {
        Ok(())
    }
}
pub(super) fn register(child: HWND, generation: u64, observation: &Arc<AtomicU32>) -> Result<()> {
    require_available()?;
    INPUT_OBSERVERS
        .try_with(|observers| {
            let mut observers = observers.try_borrow_mut().map_err(|_| unavailable())?;
            observers.retain(|_, (_, value)| value.strong_count() != 0);
            if observers.len() >= 64 || observers.contains_key(&(child.0 as isize)) {
                return Err(unavailable());
            }
            observers.insert(child.0 as isize, (generation, Arc::downgrade(observation)));
            Ok(())
        })
        .map_err(|_| unavailable())?
}
pub(super) fn require_original(
    child: HWND,
    generation: u64,
    observation: &Arc<AtomicU32>,
) -> Result<()> {
    require_available()?;
    let original = INPUT_OBSERVERS
        .try_with(|observers| {
            let observers = observers.try_borrow().map_err(|_| unavailable())?;
            Ok::<bool, VisibleError>(observers.get(&(child.0 as isize)).is_some_and(
                |(expected, weak)| {
                    *expected == generation
                        && weak
                            .upgrade()
                            .is_some_and(|value| Arc::ptr_eq(&value, observation))
                },
            ))
        })
        .map_err(|_| unavailable())??;
    if original { Ok(()) } else { Err(unavailable()) }
}

pub(super) unsafe extern "system" fn visible_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if matches!(message, WM_STYLECHANGING | WM_STYLECHANGED)
        && wparam.0 as u32 == GWL_EXSTYLE.0 as u32
    {
        // These notification messages have no LastError return contract. Preserve the caller's
        // exact incoming value across our observation and original delegation; never force zero.
        let incoming = unsafe { GetLastError().0 };
        return preserve_callback_error(
            incoming,
            || unsafe { visible_proc_body(hwnd, message, wparam, lparam) },
            || unsafe { GetLastError().0 },
            |error| unsafe { SetLastError(WIN32_ERROR(error)) },
            |entry, delegated| {
                #[cfg(test)]
                super::style_trace::record_style_message(hwnd, message, entry, delegated);
                #[cfg(not(test))]
                let _ = (entry, delegated);
            },
        );
    }
    unsafe { visible_proc_body(hwnd, message, wparam, lparam) }
}

fn preserve_callback_error<T>(
    incoming: u32,
    delegate: impl FnOnce() -> T,
    read: impl FnOnce() -> u32,
    restore: impl FnOnce(u32),
    observe: impl FnOnce(u32, u32),
) -> T {
    let result = delegate();
    let delegated = read();
    observe(incoming, delegated);
    restore(incoming);
    result
}

unsafe fn visible_proc_body(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_NCHITTEST {
        return LRESULT(HTTRANSPARENT as isize);
    }
    if message == WM_MOUSEACTIVATE {
        return LRESULT(MA_NOACTIVATE as isize);
    }
    // SAFETY: Original private class owns the installed Events reference until downstream NCDESTROY.
    let generation = unsafe {
        let events = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const window_events::Events;
        if events.is_null() {
            None
        } else {
            Some((*events).generation)
        }
    };
    if unexpected_input(message, wparam) {
        let observation = INPUT_OBSERVERS
            .try_with(|observers| {
                let observers = observers.try_borrow().map_err(|_| unavailable())?;
                observers
                    .get(&(hwnd.0 as isize))
                    .and_then(|(expected, weak)| {
                        (Some(*expected) == generation)
                            .then(|| weak.upgrade())
                            .flatten()
                    })
                    .ok_or_else(unavailable)
            })
            .map_err(|_| unavailable())
            .and_then(|result| result);
        if let Ok(observation) = observation {
            let _ = observation.compare_exchange(0, message, Ordering::AcqRel, Ordering::Acquire);
        }
    }
    // SAFETY: Same private original class; original installation/detachment and ACK remain authoritative.
    let result = unsafe { window_events::window_proc(hwnd, message, wparam, lparam) };
    if message == WM_NCDESTROY {
        // Original NCDESTROY/detach already ran exactly once, regardless of observer availability.
        let _ = INPUT_OBSERVERS
            .try_with(|observers| {
                let mut observers = observers.try_borrow_mut().map_err(|_| unavailable())?;
                if observers
                    .get(&(hwnd.0 as isize))
                    .is_some_and(|(expected, _)| Some(*expected) == generation)
                {
                    observers.remove(&(hwnd.0 as isize));
                } else {
                    return Err(unavailable());
                }
                Ok::<(), VisibleError>(())
            })
            .map_err(|_| unavailable());
    }
    result
}

#[cfg(test)]
pub(super) fn trace_unavailable() -> VisibleError {
    unavailable()
}
#[cfg(test)]
mod tests;
pub(super) fn unexpected_input(message: u32, wparam: WPARAM) -> bool {
    matches!(
        message,
        WM_SETFOCUS
            | WM_KEYDOWN
            | WM_SYSKEYDOWN
            | WM_CHAR
            | WM_LBUTTONDOWN
            | WM_RBUTTONDOWN
            | WM_MBUTTONDOWN
    ) || (message == WM_ACTIVATE && wparam.0 & 0xffff != WA_INACTIVE as usize)
}
