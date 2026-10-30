//! Query-only Windows adapter. No process creation, jobs or affinity setters.
use crate::{cpu_selection::Core, topology_records};
use std::{ffi::c_void, io};
type Handle = *mut c_void;
const MAX_TOPOLOGY_BYTES: u32 = 1_048_576;
const INSUFFICIENT_BUFFER: i32 = 122;

#[derive(Debug)]
pub struct Failure {
    pub stage: &'static str,
    pub os_code: Option<i32>,
}
impl Failure {
    fn last(stage: &'static str) -> Self {
        Self {
            stage,
            os_code: io::Error::last_os_error().raw_os_error(),
        }
    }
    fn unsupported(stage: &'static str) -> Self {
        Self {
            stage,
            os_code: None,
        }
    }
}

#[derive(Debug)]
pub struct ProcessScope {
    pub group: u16,
    pub process_mask: usize,
    pub system_mask: usize,
    pub thread_mask: usize,
    pub allowed_mask: usize,
}

#[repr(C)]
#[derive(Default)]
struct GroupAffinity {
    mask: usize,
    group: u16,
    reserved: [u16; 3],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> Handle;
    fn GetCurrentThread() -> Handle;
    fn GetProcessGroupAffinity(process: Handle, count: *mut u16, groups: *mut u16) -> i32;
    fn GetThreadGroupAffinity(thread: Handle, affinity: *mut GroupAffinity) -> i32;
    fn GetProcessAffinityMask(
        process: Handle,
        process_mask: *mut usize,
        system_mask: *mut usize,
    ) -> i32;
    fn GetLogicalProcessorInformationEx(
        relationship: i32,
        buffer: *mut c_void,
        length: *mut u32,
    ) -> i32;
}

pub fn scope() -> Result<ProcessScope, Failure> {
    if usize::BITS != 64 {
        return Err(Failure::unsupported("unsupported_32_bit_affinity_view"));
    }
    let mut groups = [0u16; 64];
    let mut count = groups.len() as u16;
    let mut thread = GroupAffinity::default();
    let mut process_mask = 0;
    let mut system_mask = 0;
    // SAFETY: current-process/thread pseudo handles and initialized fixed-size output buffers.
    unsafe {
        if GetProcessGroupAffinity(GetCurrentProcess(), &mut count, groups.as_mut_ptr()) == 0 {
            return Err(Failure::last("process_group_query"));
        }
        // GetProcessAffinityMask cannot describe an unambiguous multi-group process here.
        if count != 1 {
            return Err(Failure::unsupported("multiple_or_unknown_process_groups"));
        }
        if GetThreadGroupAffinity(GetCurrentThread(), &mut thread) == 0 {
            return Err(Failure::last("thread_group_query"));
        }
        if groups[0] != thread.group {
            return Err(Failure::unsupported("process_thread_group_mismatch"));
        }
        if GetProcessAffinityMask(GetCurrentProcess(), &mut process_mask, &mut system_mask) == 0 {
            return Err(Failure::last("process_affinity_query"));
        }
    }
    Ok(ProcessScope {
        group: thread.group,
        process_mask,
        system_mask,
        thread_mask: thread.mask,
        allowed_mask: process_mask & system_mask & thread.mask,
    })
}

pub fn cores() -> Result<Vec<Core>, Failure> {
    if usize::BITS != 64 {
        return Err(Failure::unsupported("unsupported_32_bit_topology_view"));
    }
    let mut length = 0;
    // SAFETY: null buffer is the documented length probe; length is a valid output pointer.
    let result = unsafe { GetLogicalProcessorInformationEx(0, std::ptr::null_mut(), &mut length) };
    if result != 0 {
        return Err(Failure::unsupported("unexpected_topology_probe_success"));
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() != Some(INSUFFICIENT_BUFFER) {
        return Err(Failure {
            stage: "topology_length_query",
            os_code: error.raw_os_error(),
        });
    }
    // Hotplug can grow the required buffer. Bound both retries and allocation.
    for _ in 0..3 {
        if length == 0 || length > MAX_TOPOLOGY_BYTES {
            return Err(Failure::unsupported("topology_length_out_of_bounds"));
        }
        let capacity = length;
        let mut aligned = vec![0u64; (capacity as usize).div_ceil(8)];
        // SAFETY: aligned, initialized storage holds at least capacity bytes; API owns only that range.
        let result = unsafe {
            GetLogicalProcessorInformationEx(0, aligned.as_mut_ptr().cast(), &mut length)
        };
        if result != 0 {
            if length == 0 || length > capacity {
                return Err(Failure::unsupported("invalid_topology_returned_length"));
            }
            // SAFETY: returned length is checked against initialized allocated capacity above.
            let bytes = unsafe {
                std::slice::from_raw_parts(aligned.as_ptr().cast::<u8>(), length as usize)
            };
            return topology_records::decode(bytes)
                .map_err(|_| Failure::unsupported("malformed_topology_records"));
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(INSUFFICIENT_BUFFER) {
            return Err(Failure {
                stage: "topology_data_query",
                os_code: error.raw_os_error(),
            });
        }
    }
    Err(Failure::unsupported(
        "topology_changed_during_bounded_query",
    ))
}
