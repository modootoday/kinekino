//! The pinned crate's FrameNavigationCompleted wrapper is FnOnce; navigation is recurring.
use std::cell::RefCell;
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use windows::core::{Ref, Result, implement};

type Completion = Box<
    dyn FnMut(
        Option<ICoreWebView2Frame>,
        Option<ICoreWebView2NavigationCompletedEventArgs>,
    ) -> Result<()>,
>;

#[implement(ICoreWebView2FrameNavigationCompletedEventHandler)]
pub(super) struct RepeatingFrameCompletion {
    callback: RefCell<Completion>,
}
impl RepeatingFrameCompletion {
    pub fn create(callback: Completion) -> ICoreWebView2FrameNavigationCompletedEventHandler {
        Self {
            callback: RefCell::new(callback),
        }
        .into()
    }
}
impl ICoreWebView2FrameNavigationCompletedEventHandler_Impl for RepeatingFrameCompletion_Impl {
    fn Invoke(
        &self,
        sender: Ref<ICoreWebView2Frame>,
        args: Ref<ICoreWebView2NavigationCompletedEventArgs>,
    ) -> Result<()> {
        let mut callback = self.callback.try_borrow_mut().map_err(|_| {
            windows::core::Error::from_hresult(windows::core::HRESULT(0x8000_ffff_u32 as i32))
        })?;
        callback(sender.cloned(), args.cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn one_registration_accepts_repeated_native_completion_callbacks() {
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let handler = RepeatingFrameCompletion::create(Box::new(move |_, _| {
            observed.set(observed.get() + 1);
            Ok(())
        }));
        // SAFETY: both calls and the callback remain on this test's thread; no browser is created.
        unsafe {
            handler
                .Invoke(
                    None::<&ICoreWebView2Frame>,
                    None::<&ICoreWebView2NavigationCompletedEventArgs>,
                )
                .unwrap();
            handler
                .Invoke(
                    None::<&ICoreWebView2Frame>,
                    None::<&ICoreWebView2NavigationCompletedEventArgs>,
                )
                .unwrap();
        }
        assert_eq!(calls.get(), 2);
    }
}
