//! A working stdin pipe for spawned children on Windows.
//!
//! Rust's `Stdio::piped()` builds its anonymous pipe through a named-pipe path
//! (`CreateNamedPipeW` + `CreateFileW`) in the standard library; on this
//! machine that connect step fails with ERROR_PIPE_BUSY (os error 231), so any
//! `Command::spawn()` that pipes stdin dies before the child even starts.
//!
//! `CreatePipe` itself works fine (the shared Python tooling relies on it), so
//! we build the pipe here and hand only its read end to the child.

use anyhow::{anyhow, Result};
use std::fs::File;
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::process::Stdio;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Pipes::CreatePipe;

/// Create an anonymous pipe and return (child's stdin config, our write end).
///
/// The read end is moved into the returned `Stdio`, so the std library owns it
/// and closes it once the child has been spawned. The write end is *not*
/// inheritable, so the child cannot hold it open and `drop(file)` reliably
/// signals EOF.
pub fn stdin_pipe() -> Result<(Stdio, File)> {
    let mut read: HANDLE = std::ptr::null_mut();
    let mut write: HANDLE = std::ptr::null_mut();
    // SAFETY: both out-pointers are valid; default security (non-inheritable).
    let ok = unsafe { CreatePipe(&mut read, &mut write, std::ptr::null(), 0) };
    if ok == 0 {
        return Err(anyhow!(
            "CreatePipe failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    if read.is_null() || write.is_null() {
        return Err(anyhow!("CreatePipe returned null handles"));
    }
    // SAFETY: CreatePipe just produced these two owned handles.
    let read = unsafe { OwnedHandle::from_raw_handle(read as _) };
    let write = unsafe { OwnedHandle::from_raw_handle(write as _) };
    Ok((Stdio::from(read), File::from(write)))
}
