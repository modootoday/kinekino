use super::{
    WindowsFamily,
    handles::{Handle, error},
    identity,
    job::FamilyJob,
    job::prepared::PreparedFamilyJob,
    pipe::ChildPipe,
};
use kinekino_core::platform::process::{
    CONTROL_ARGUMENT, DATA_ARGUMENT, Deadline, END_ARGUMENT, FamilyBudget, FaultPoint, LaunchState,
    OPERATION_ARGUMENT, OwnedFamily, PreparedFamily, ProcessIdentity, SourceChildren, SpawnFailure,
    SpawnRequest,
};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs::File,
    io,
    mem::{size_of, size_of_val},
    os::windows::ffi::OsStrExt,
    time::Instant,
};
use windows::{
    Win32::{
        Foundation::{HANDLE, HANDLE_FLAG_INHERIT, HANDLE_FLAGS, SetHandleInformation},
        System::{Pipes::CreatePipe, Threading::*},
    },
    core::{PCWSTR, PWSTR},
};

struct Attributes {
    storage: Vec<u128>,
    initialized: bool,
}
impl Attributes {
    fn ptr(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        LPPROC_THREAD_ATTRIBUTE_LIST(self.storage.as_mut_ptr().cast())
    }
    fn new(handles: &[HANDLE], fail_after_init: bool) -> io::Result<Self> {
        let mut bytes = 0;
        // SAFETY: sizing query passes no storage, as required by Win32.
        let sizing = unsafe { InitializeProcThreadAttributeList(None, 1, None, &mut bytes) };
        if sizing.is_ok()
            || sizing
                .as_ref()
                .err()
                .is_none_or(|failure| failure.code().0 & 0xffff != 122)
            || bytes == 0
        {
            return Err(io::Error::other("unexpected attribute sizing result"));
        }
        let count = bytes
            .checked_add(size_of::<u128>() - 1)
            .ok_or_else(|| io::Error::other("attribute allocation overflow"))?
            / size_of::<u128>();
        let mut storage = Vec::new();
        storage.try_reserve_exact(count).map_err(io::Error::other)?;
        storage.resize(count, 0);
        let mut result = Self {
            storage,
            initialized: false,
        };
        // SAFETY: 16-byte aligned initialized storage survives through CreateProcess; handle array
        // remains live in the caller through CreateProcess and contains inheritable duplicates only.
        unsafe {
            InitializeProcThreadAttributeList(Some(result.ptr()), 1, None, &mut bytes)
                .map_err(error)?;
            result.initialized = true;
            if fail_after_init {
                return Err(io::Error::other("injected initialized attribute failure"));
            }
            UpdateProcThreadAttribute(
                result.ptr(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                Some(handles.as_ptr().cast()),
                size_of_val(handles),
                None,
                None,
            )
            .map_err(error)?;
        }
        Ok(result)
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                DeleteProcThreadAttributeList(self.ptr());
            }
        }
    }
}
fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut result = value.encode_wide().take(32768).collect::<Vec<_>>();
    if result.contains(&0) || result.len() >= 32767 {
        return Err(io::Error::other("NUL or overlong UTF-16 input"));
    }
    result.push(0);
    Ok(result)
}
fn quoted(value: &OsStr) -> io::Result<Vec<u16>> {
    let value = wide(value)?;
    let mut output = vec![b'"' as u16];
    let mut slashes = 0usize;
    for unit in &value[..value.len() - 1] {
        if *unit == b'\\' as u16 {
            slashes += 1;
            continue;
        }
        let count = if *unit == b'"' as u16 {
            slashes * 2 + 1
        } else {
            slashes
        };
        output.extend(std::iter::repeat_n(b'\\' as u16, count));
        output.push(*unit);
        slashes = 0;
    }
    output.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
    output.push(b'"' as u16);
    Ok(output)
}
fn environment(values: &BTreeMap<String, OsString>) -> io::Result<Vec<u16>> {
    let mut ordered = BTreeMap::new();
    for (key, value) in values {
        if key.is_empty() || key.len() >= 32767 || !key.is_ascii() || key.contains(['=', '\0']) {
            return Err(io::Error::other(
                "explicit environment names must be nonempty ASCII without = or NUL",
            ));
        }
        if ordered
            .insert(key.to_ascii_uppercase(), (key, value))
            .is_some()
        {
            return Err(io::Error::other(
                "case-insensitive duplicate environment name",
            ));
        }
    }
    let mut block = Vec::new();
    for (_, (key, value)) in ordered {
        let mut entry = wide(OsStr::new(key))?;
        entry.pop();
        entry.push(b'=' as u16);
        entry.extend(wide(value)?);
        block.extend(entry);
        if block.len() >= 32767 {
            return Err(io::Error::other("environment block exceeds bound"));
        }
    }
    block.push(0);
    if block.len() == 1 {
        block.push(0);
    }
    Ok(block)
}
fn control_pipe() -> io::Result<(File, File)> {
    let mut read = HANDLE::default();
    let mut write = HANDLE::default();
    // SAFETY: default security attributes make both ends non-inheritable.
    unsafe {
        CreatePipe(&mut read, &mut write, None, 0).map_err(error)?;
    }
    // Wrap both outputs before any further fallible operation.
    let read = unsafe { Handle::from_raw(read) };
    let write = unsafe { Handle::from_raw(write) };
    unsafe {
        SetHandleInformation(write.raw(), HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)).map_err(error)?;
    }
    Ok((read.into_file(), write.into_file()))
}
fn remaining(deadline: Deadline) -> io::Result<()> {
    remaining_at(deadline, Instant::now())
}
fn remaining_at(deadline: Deadline, now: Instant) -> io::Result<()> {
    if deadline.operation_remaining(now).is_zero() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "spawn operation budget exhausted",
        ));
    }
    Ok(())
}
fn child_handle(
    endpoint: &dyn kinekino_core::platform::process::SourceChildEndpoint,
) -> io::Result<&Handle> {
    endpoint
        .as_any()
        .downcast_ref::<ChildPipe>()
        .map(|pipe| &pipe.0)
        .ok_or_else(|| io::Error::other("foreign source endpoint"))
}
/// Transfer the exact pre-reserved Job when given; no replacement Job is silently created.
pub(super) fn spawn(
    request: SpawnRequest<'_>,
    prepared: Option<Box<dyn PreparedFamily>>,
) -> Result<Box<dyn OwnedFamily>, Box<SpawnFailure>> {
    let before = |stage, error| {
        Box::new(SpawnFailure {
            stage,
            error,
            owner: None,
            termination_error: None,
        })
    };
    let prepared_job = match prepared {
        None => None,
        Some(prepared) => {
            if let Err(error) = prepared.check_current() {
                return Err(before("prepared_job", error));
            }
            match prepared.into_any().downcast::<PreparedFamilyJob>() {
                Ok(job) => Some((job.job, job.budget)),
                Err(_) => {
                    return Err(before(
                        "prepared_job",
                        io::Error::other("foreign prepared family"),
                    ));
                }
            }
        }
    };
    spawn_inner(request, prepared_job)
}

fn spawn_inner(
    request: SpawnRequest<'_>,
    prepared_job: Option<(FamilyJob, FamilyBudget)>,
) -> Result<Box<dyn OwnedFamily>, Box<SpawnFailure>> {
    let SpawnRequest {
        executable: spec_executable,
        cwd: spec_cwd,
        arguments,
        environment: spec_environment,
        stdin,
        stdout,
        stderr,
        deadline,
        source,
        fault,
    } = request;
    let before = |stage, error| {
        Box::new(SpawnFailure {
            stage,
            error,
            owner: None,
            termination_error: None,
        })
    };
    let expected_budget = prepared_job.as_ref().map(|(_, budget)| *budget);
    let setup = (|| -> io::Result<_> {
        remaining(deadline)?;
        if !spec_executable.is_absolute() || !spec_cwd.is_absolute() {
            return Err(io::Error::other("absolute executable and cwd required"));
        }
        let executable = spec_executable.canonicalize()?;
        if !executable.is_file() || !spec_cwd.is_dir() {
            return Err(io::Error::other("executable/cwd unavailable"));
        }
        let executable_wide = wide(executable.as_os_str())?;
        let cwd = wide(spec_cwd.as_os_str())?;
        let mut command = quoted(executable.as_os_str())?;
        for argument in &arguments {
            command.push(b' ' as u16);
            command.extend(quoted(argument)?);
            if command.len() >= 32767 {
                return Err(io::Error::other("command line exceeds UTF-16 bound"));
            }
        }
        if command.len() >= 32767 {
            return Err(io::Error::other("command line exceeds UTF-16 bound"));
        }
        command.push(0);
        let env = spec_environment.as_ref().map(environment).transpose()?;
        let job = match prepared_job {
            Some((job, _)) => job,
            None => FamilyJob::new()?,
        };
        let (input, writer) = match stdin {
            Some(file) => (file, None),
            None => {
                let (read, write) = control_pipe()?;
                (read, Some(write))
            }
        };
        let mut inherited = vec![
            Handle::inheritable_file(&input)?,
            Handle::inheritable_file(&stdout)?,
            Handle::inheritable_file(&stderr)?,
        ];
        if let Some(SourceChildren {
            control,
            data,
            cutoffs,
        }) = source
        {
            let control = child_handle(control)?.inheritable()?;
            let data = child_handle(data)?.inheritable()?;
            let extra = [
                format!("{CONTROL_ARGUMENT}{}", control.raw().0 as usize),
                format!("{DATA_ARGUMENT}{}", data.raw().0 as usize),
                format!("{OPERATION_ARGUMENT}{}", cutoffs.operation()),
                format!("{END_ARGUMENT}{}", cutoffs.end()),
            ];
            command.pop(); // Replace only the original terminator, never caller bytes.
            for argument in extra {
                command.push(b' ' as u16);
                command.extend(quoted(OsStr::new(&argument))?);
            }
            if command.len() >= 32767 {
                return Err(io::Error::other("source command exceeds UTF-16 bound"));
            }
            command.push(0);
            inherited.push(control);
            inherited.push(data);
        }
        let handles = inherited.iter().map(Handle::raw).collect::<Vec<_>>();
        if fault == Some(FaultPoint::AttributePreparation) {
            return Err(io::Error::other("injected attribute failure"));
        }
        let fail_after_init = fault == Some(FaultPoint::AttributeInitialized);
        let mut attributes = Attributes::new(&handles, fail_after_init)?;
        let startup = STARTUPINFOEXW {
            StartupInfo: STARTUPINFOW {
                cb: size_of::<STARTUPINFOEXW>() as u32,
                dwFlags: STARTF_USESTDHANDLES,
                hStdInput: handles[0],
                hStdOutput: handles[1],
                hStdError: handles[2],
                ..Default::default()
            },
            lpAttributeList: attributes.ptr(),
        };
        let mut created = PROCESS_INFORMATION::default();
        remaining(deadline)?;
        // SAFETY: explicit application, writable UTF-16 command, initialized startup/attribute
        // storage/allowlist/inheritable handles/environment remain live until this call returns.
        unsafe {
            CreateProcessW(
                PCWSTR(executable_wide.as_ptr()),
                Some(PWSTR(command.as_mut_ptr())),
                None,
                None,
                true,
                CREATE_SUSPENDED
                    | CREATE_NO_WINDOW
                    | EXTENDED_STARTUPINFO_PRESENT
                    | CREATE_UNICODE_ENVIRONMENT,
                env.as_ref().map(|block| block.as_ptr().cast()),
                PCWSTR(cwd.as_ptr()),
                &startup.StartupInfo,
                &mut created,
            )
            .map_err(error)?;
        }
        let process = unsafe { Handle::from_raw(created.hProcess) };
        let thread = unsafe { Handle::from_raw(created.hThread) };
        Ok(WindowsFamily {
            deadline,
            process,
            thread,
            job,
            input: writer,
            identity: ProcessIdentity {
                pid: created.dwProcessId,
                creation_filetime: None,
                executable,
            },
            state: LaunchState::UnassignedSuspended,
            outer_job_observed: false,
            fault,
        })
    })();
    let mut owner = setup.map_err(|error| before("prepare_or_create", error))?;
    let admitted = (|| -> io::Result<()> {
        owner.identity.creation_filetime = Some(identity::creation(owner.process.raw())?);
        if identity::image(owner.process.raw())?.canonicalize()? != owner.identity.executable {
            return Err(io::Error::other("process image readback differs"));
        }
        let mut outer = windows::core::BOOL::default();
        unsafe {
            windows::Win32::System::JobObjects::IsProcessInJob(
                owner.process.raw(),
                None,
                &mut outer,
            )
            .map_err(error)?;
        }
        owner.outer_job_observed = outer.as_bool();
        if matches!(
            fault,
            Some(
                FaultPoint::AfterCreate
                    | FaultPoint::Assignment
                    | FaultPoint::AssignmentAndTermination
            )
        ) {
            return Err(io::Error::other("injected unassigned suspended failure"));
        }
        owner.job.assign(owner.process.raw())?;
        owner.state = LaunchState::AssignedSuspended;
        if let Some(budget) = expected_budget {
            owner.job.check_budget(budget)?;
        }
        remaining(deadline)?;
        if fault == Some(FaultPoint::BeforeResume) {
            return Err(io::Error::other("injected pre-resume failure"));
        }
        // SAFETY: the original primary thread was created suspended, has never been resumed here.
        let previous = if fault == Some(FaultPoint::UnexpectedResume) {
            0
        } else {
            unsafe { ResumeThread(owner.thread.raw()) }
        };
        if previous != 1 {
            return Err(if previous == u32::MAX {
                io::Error::last_os_error()
            } else {
                io::Error::other("unexpected primary thread suspend count")
            });
        }
        owner.state = LaunchState::Running;
        let resumed_at = Instant::now();
        let resumed_at = if fault == Some(FaultPoint::LateResume) {
            resumed_at
                .checked_add(deadline.operation_remaining(resumed_at))
                .unwrap_or(resumed_at)
        } else {
            resumed_at
        };
        remaining_at(deadline, resumed_at)?;
        Ok(())
    })();
    if let Err(error) = admitted {
        let termination_error = owner.terminate().err();
        return Err(Box::new(SpawnFailure {
            stage: "assign_verify_resume",
            error,
            owner: Some(Box::new(owner)),
            termination_error,
        }));
    }
    Ok(Box::new(owner))
}
