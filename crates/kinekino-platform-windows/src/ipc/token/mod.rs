//! Token identity queries. Moved from `kinekino-player` local_host::peer_token.
use super::{fail, invalid};
use kinekino_core::platform::ipc::{IpcFailure, PeerIdentity};
use std::{ffi::c_void, mem::size_of};
use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_NO_TOKEN, HANDLE, HLOCAL, LocalFree},
        Security::{
            Authorization::ConvertSidToStringSidW, GetSidSubAuthority, GetSidSubAuthorityCount,
            GetTokenInformation, IsValidSid, RevertToSelf, TOKEN_ELEVATION, TOKEN_ELEVATION_TYPE,
            TOKEN_INFORMATION_CLASS, TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TOKEN_USER,
            TokenElevation, TokenElevationType, TokenIntegrityLevel, TokenUser,
        },
        System::{
            Pipes::ImpersonateNamedPipeClient,
            Threading::{GetCurrentProcess, GetCurrentThread, OpenProcessToken, OpenThreadToken},
        },
    },
    core::{HRESULT, PWSTR},
};

fn denied() -> IpcFailure {
    invalid("token_query")
}
pub(crate) struct Token(pub(crate) HANDLE);
impl Drop for Token {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
fn information(
    token: HANDLE,
    class: TOKEN_INFORMATION_CLASS,
    minimum: usize,
) -> Result<Vec<usize>, IpcFailure> {
    let mut length = 0;
    unsafe {
        let _ = GetTokenInformation(token, class, None, 0, &mut length);
    }
    if (length as usize) < minimum || length > 65536 {
        return Err(denied());
    }
    let capacity = length;
    let mut buffer = vec![0usize; (length as usize).div_ceil(size_of::<usize>())];
    unsafe {
        GetTokenInformation(
            token,
            class,
            Some(buffer.as_mut_ptr().cast()),
            capacity,
            &mut length,
        )
        .map_err(|e| fail("GetTokenInformation", e))?;
    }
    if length > capacity || (length as usize) < minimum {
        return Err(denied());
    }
    Ok(buffer)
}
pub(crate) fn profile(token: HANDLE) -> Result<PeerIdentity, IpcFailure> {
    let user = information(token, TokenUser, size_of::<TOKEN_USER>())?;
    let user_sid = unsafe {
        let user = &*user.as_ptr().cast::<TOKEN_USER>();
        let mut text = PWSTR::null();
        ConvertSidToStringSidW(user.User.Sid, &mut text)
            .map_err(|e| fail("ConvertSidToStringSidW", e))?;
        let result = text.to_string().map_err(|_| denied());
        let _ = LocalFree(Some(HLOCAL(text.0.cast::<c_void>())));
        result?
    };
    let integrity = information(
        token,
        TokenIntegrityLevel,
        size_of::<TOKEN_MANDATORY_LABEL>(),
    )?;
    let integrity_rid = unsafe {
        let label = &*integrity.as_ptr().cast::<TOKEN_MANDATORY_LABEL>();
        if !IsValidSid(label.Label.Sid).as_bool() {
            return Err(denied());
        }
        let count = *GetSidSubAuthorityCount(label.Label.Sid);
        if count == 0 {
            return Err(denied());
        }
        *GetSidSubAuthority(label.Label.Sid, u32::from(count - 1))
    };
    let elevation = information(token, TokenElevation, size_of::<TOKEN_ELEVATION>())?;
    let kind = information(token, TokenElevationType, size_of::<TOKEN_ELEVATION_TYPE>())?;
    let elevated = unsafe { (*elevation.as_ptr().cast::<TOKEN_ELEVATION>()).TokenIsElevated != 0 };
    let elevation_type = unsafe { (*kind.as_ptr().cast::<TOKEN_ELEVATION_TYPE>()).0 };
    Ok(PeerIdentity {
        principal: user_sid,
        tier: integrity_rid,
        elevated,
        elevation_kind: elevation_type,
    })
}
pub(crate) fn process(process: HANDLE) -> Result<PeerIdentity, IpcFailure> {
    let mut token = HANDLE::default();
    unsafe {
        OpenProcessToken(process, TOKEN_QUERY, &mut token)
            .map_err(|e| fail("OpenProcessToken", e))?;
    }
    profile(Token(token).0)
}
pub(crate) fn current() -> Result<PeerIdentity, IpcFailure> {
    process(unsafe { GetCurrentProcess() })
}
/// A new OS thread must not silently discard a caller's impersonation context.
pub(crate) fn require_unimpersonated() -> Result<(), IpcFailure> {
    let mut token = HANDLE::default();
    match unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, true, &mut token) } {
        Ok(()) => {
            drop(Token(token));
            Err(invalid("thread_impersonating"))
        }
        Err(error) if error.code() == HRESULT::from_win32(ERROR_NO_TOKEN.0) => Ok(()),
        Err(e) => Err(fail("OpenThreadToken", e)),
    }
}
pub(crate) struct Impersonation;
impl Drop for Impersonation {
    fn drop(&mut self) {
        unsafe {
            let _ = RevertToSelf();
        }
    }
}
/// Synchronous token query only: no await, SDK dispatch or external operation while impersonating.
pub(crate) fn message(pipe: HANDLE) -> Result<PeerIdentity, IpcFailure> {
    unsafe {
        ImpersonateNamedPipeClient(pipe).map_err(|e| fail("ImpersonateNamedPipeClient", e))?;
    }
    let guard = Impersonation;
    let result = (|| {
        let mut token = HANDLE::default();
        unsafe {
            OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, true, &mut token)
                .map_err(|_| denied())?;
        }
        profile(Token(token).0)
    })();
    if unsafe { RevertToSelf() }.is_err() {
        drop(guard);
        return Err(IpcFailure::fatal(
            "RevertToSelf",
            std::io::Error::other("impersonation not restored"),
        ));
    }
    std::mem::forget(guard);
    result
}

/// String SID of the current process user.
pub(crate) fn current_sid() -> Result<String, IpcFailure> {
    current().map(|p| p.principal)
}

#[cfg(test)]
mod tests;
