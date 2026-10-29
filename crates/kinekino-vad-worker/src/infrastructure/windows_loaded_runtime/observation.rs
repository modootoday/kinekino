use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{ffi::c_void, path::PathBuf};
#[repr(C)]
struct OsVersion {
    size: u32,
    major: u32,
    minor: u32,
    build: u32,
    platform: u32,
    service_pack: [u16; 128],
}
#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "GetCurrentProcess"]
    fn current_process() -> *mut c_void;
    #[link_name = "K32EnumProcessModules"]
    fn enum_modules(
        process: *mut c_void,
        modules: *mut *mut c_void,
        bytes: u32,
        needed: *mut u32,
    ) -> i32;
    #[link_name = "GetModuleHandleW"]
    fn main_module(name: *const u16) -> *mut c_void;
    #[link_name = "GetModuleFileNameW"]
    fn module_path(module: *mut c_void, buffer: *mut u16, size: u32) -> u32;
    #[link_name = "GetModuleHandleExW"]
    fn retain_module(flags: u32, name: *const u16, module: *mut *mut c_void) -> i32;
    #[link_name = "FreeLibrary"]
    fn free_library(module: *mut c_void) -> i32;
    #[link_name = "GetSystemDirectoryW"]
    fn system_directory(buffer: *mut u16, size: u32) -> u32;
}
#[link(name = "ntdll")]
unsafe extern "system" {
    #[link_name = "RtlGetVersion"]
    fn os_version(version: *mut OsVersion) -> i32;
}
pub(super) fn io() -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        format!(
            "Native module observation: {}",
            std::io::Error::last_os_error()
        ),
    )
}
fn changed(message: &str) -> ApiError {
    ApiError::new(ErrorCode::SourceExpired, message)
}
pub(super) struct ModuleReference {
    handle: *mut c_void,
    owned: bool,
}
impl Drop for ModuleReference {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                free_library(self.handle);
            }
        }
    }
}
pub(super) struct ObservedModule {
    pub path: PathBuf,
    pub is_main: bool,
    _reference: ModuleReference,
}
pub(super) fn modules() -> Result<Vec<ObservedModule>, ApiError> {
    let mut raw = vec![std::ptr::null_mut(); 512];
    let mut needed = 0u32;
    let bytes = (raw.len() * std::mem::size_of::<*mut c_void>()) as u32;
    if unsafe { enum_modules(current_process(), raw.as_mut_ptr(), bytes, &mut needed) } == 0 {
        return Err(io());
    }
    if needed > bytes || !(needed as usize).is_multiple_of(std::mem::size_of::<*mut c_void>()) {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Actual module snapshot exceeds bound",
        ));
    }
    raw.truncate(needed as usize / std::mem::size_of::<*mut c_void>());
    let main = unsafe { main_module(std::ptr::null()) };
    if main.is_null() {
        return Err(io());
    }
    let mut observed = Vec::new();
    for module in raw {
        let mut retained = module;
        if module != main {
            // Retain the currently mapped module by address BEFORE reading its
            // path; never interpret a stale snapshot path as a held identity.
            if unsafe { retain_module(4, module.cast::<u16>(), &mut retained) } == 0 {
                return Err(io());
            }
            if retained != module {
                let _release = ModuleReference {
                    handle: retained,
                    owned: true,
                };
                return Err(changed("Module identity changed during snapshot"));
            }
        }
        let reference = ModuleReference {
            handle: retained,
            owned: module != main,
        };
        let mut buffer = vec![0u16; 32768];
        let count = unsafe { module_path(retained, buffer.as_mut_ptr(), buffer.len() as u32) };
        if count == 0 {
            return Err(io());
        }
        if count as usize >= buffer.len() {
            return Err(changed("Actual module path truncated"));
        }
        let path = String::from_utf16(&buffer[..count as usize])
            .map_err(|_| changed("Actual module path is not Unicode"))?;
        observed.push(ObservedModule {
            path: PathBuf::from(path),
            is_main: module == main,
            _reference: reference,
        });
    }
    Ok(observed)
}
pub(super) fn system_root() -> Result<PathBuf, ApiError> {
    let mut buffer = vec![0u16; 32768];
    let count = unsafe { system_directory(buffer.as_mut_ptr(), buffer.len() as u32) };
    if count == 0 {
        return Err(io());
    }
    if count as usize >= buffer.len() {
        return Err(changed("System directory path truncated"));
    }
    let path = String::from_utf16(&buffer[..count as usize])
        .map_err(|_| changed("System directory is not Unicode"))?;
    std::fs::canonicalize(path).map_err(|e| ApiError::new(ErrorCode::EngineFailed, e.to_string()))
}
pub(super) fn check_windows(minimum_build: u32) -> Result<(), ApiError> {
    let mut version = OsVersion {
        size: std::mem::size_of::<OsVersion>() as u32,
        major: 0,
        minor: 0,
        build: 0,
        platform: 0,
        service_pack: [0; 128],
    };
    if unsafe { os_version(&mut version) } != 0 {
        return Err(ApiError::new(
            ErrorCode::EngineFailed,
            "Original Windows version observation failed",
        ));
    }
    if version.major != 10 || version.build < minimum_build {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Windows build is outside the independently configured native baseline",
        ));
    }
    Ok(())
}
