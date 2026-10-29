//! Moved from `kinekino-player` local_host::peer_token tests (ADR 0007 batch 6).
use super::*;
use crate::ipc::{WindowsIpc, permits};
use kinekino_core::platform::ipc::IpcTransport;
use std::{sync::mpsc, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use windows::{
    Win32::Security::{
        Authorization::ConvertStringSidToSidW, DuplicateTokenEx, ImpersonateLoggedOnUser, PSID,
        SecurityImpersonation, SetTokenInformation, TOKEN_ADJUST_DEFAULT, TOKEN_DUPLICATE,
        TOKEN_IMPERSONATE, TokenImpersonation,
    },
    core::w,
};

fn duplicate() -> Token {
    let mut original = HANDLE::default();
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY | TOKEN_DUPLICATE,
            &mut original,
        )
        .unwrap();
    }
    let original = Token(original);
    let mut duplicate = HANDLE::default();
    unsafe {
        DuplicateTokenEx(
            original.0,
            TOKEN_QUERY | TOKEN_IMPERSONATE | TOKEN_ADJUST_DEFAULT,
            None,
            SecurityImpersonation,
            TokenImpersonation,
            &mut duplicate,
        )
        .unwrap();
    }
    Token(duplicate)
}
fn restore(guard: Impersonation) {
    unsafe {
        RevertToSelf().unwrap();
    }
    std::mem::forget(guard);
}

#[test]
fn unreadable_process_or_message_token_is_denied_and_does_not_change_thread_context() {
    let before = current().unwrap();
    assert!(process(HANDLE::default()).is_err());
    let failure = message(HANDLE::default()).unwrap_err();
    assert!(!failure.fatal);
    assert!(require_unimpersonated().is_ok());
    assert_eq!(current().unwrap(), before);
}

#[test]
fn current_real_token_is_observed_and_impersonating_callers_cannot_gain_a_fresh_thread() {
    let observed = current().unwrap();
    assert!(permits(&observed, &observed));
    let token = duplicate();
    unsafe {
        ImpersonateLoggedOnUser(token.0).unwrap();
    }
    let guard = Impersonation;
    assert!(require_unimpersonated().is_err());
    assert!(WindowsIpc.ensure_thread_identity().is_err());
    restore(guard);
    assert!(require_unimpersonated().is_ok());
}

#[test]
fn real_lower_integrity_pipe_context_is_denied_without_changing_process_or_account_privileges() {
    let local = current().unwrap();
    assert!(
        local.tier > 0x1000,
        "fixture requires a medium-or-higher process"
    );
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let ipc = WindowsIpc;
    let endpoint = ipc.fresh_endpoint("kinekino").unwrap();
    let mut listener = {
        let _entered = rt.enter();
        ipc.listen(&endpoint).unwrap()
    };
    let path = ipc.address(&endpoint).unwrap();
    let (result, received) = mpsc::sync_channel(1);
    let (release, wait) = mpsc::sync_channel(1);
    let lower = if local.tier >= 0x3000 { 0x2000 } else { 0x1000 };
    let worker = std::thread::spawn(move || {
        let token = duplicate();
        let mut sid = PSID::default();
        unsafe {
            ConvertStringSidToSidW(
                if lower == 0x2000 {
                    w!("S-1-16-8192")
                } else {
                    w!("S-1-16-4096")
                },
                &mut sid,
            )
            .unwrap();
        }
        let label = TOKEN_MANDATORY_LABEL {
            Label: windows::Win32::Security::SID_AND_ATTRIBUTES {
                Sid: sid,
                Attributes: 0x20,
            },
        };
        let set = unsafe {
            SetTokenInformation(
                token.0,
                TokenIntegrityLevel,
                (&label as *const TOKEN_MANDATORY_LABEL).cast(),
                size_of::<TOKEN_MANDATORY_LABEL>() as u32,
            )
        };
        unsafe {
            let _ = LocalFree(Some(HLOCAL(sid.0)));
        }
        set.unwrap();
        let lowered = profile(token.0).unwrap();
        assert_eq!(lowered.tier, lower);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        unsafe {
            ImpersonateLoggedOnUser(token.0).unwrap();
        }
        let guard = Impersonation;
        let connection = {
            let _entered = rt.enter();
            WindowsIpc.connect_address(&path)
        };
        match connection {
            Ok(mut client) => {
                rt.block_on(client.write_all(b"x")).unwrap();
                restore(guard);
                result.send((lowered, true)).unwrap();
                let _ = wait.recv_timeout(Duration::from_secs(2));
            }
            Err(error) => {
                restore(guard);
                assert_eq!(error.source.kind(), std::io::ErrorKind::PermissionDenied);
                result.send((lowered, false)).unwrap();
            }
        }
        assert!(require_unimpersonated().is_ok());
    });
    let (lowered, connected) = received.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(local.principal, lowered.principal);
    assert!(!permits(&local, &lowered));
    if connected {
        rt.block_on(async {
            let mut stream = tokio::time::timeout(Duration::from_secs(1), listener.accept())
                .await
                .unwrap()
                .unwrap();
            let process_peer = stream.peer().unwrap().identity;
            let mut byte = [0];
            tokio::time::timeout(Duration::from_secs(1), stream.read_exact(&mut byte))
                .await
                .unwrap()
                .unwrap();
            let message = stream.message_peer().unwrap();
            assert!(!permits(&process_peer, &message));
        });
        release.send(()).unwrap();
    }
    worker.join().unwrap();
    listener.cancel();
    assert_eq!(current().unwrap(), local);
    assert!(require_unimpersonated().is_ok());
}
