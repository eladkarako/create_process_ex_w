use std::{
    ffi::{c_void, OsStr, OsString},
    io::Error,
    mem::size_of,
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr::{null, null_mut},
};

use crate::binding::{
    CloseHandle, CreateProcessW, GetExitCodeProcess, TerminateProcess, WaitForSingleObject,
    BOOL, CREATE_UNICODE_ENVIRONMENT, DWORD, INFINITE, PCWSTR, PDWORD, PROCESS_INFORMATION, PWSTR,
    SECURITY_ATTRIBUTES, STARTUPINFOW, STATUS_PENDING, UINT, WAIT_OBJECT_0,
};
use crate::env_block::build_env_block;
use crate::exit_status::ExitStatus;

/// A handle-like wrapper around a newly created Windows process.
///
/// `Child` owns the Windows `PROCESS_INFORMATION` returned from `CreateProcessW` and
/// provides higher-level operations such as:
///
/// - `kill`: terminate the process
/// - `wait`: wait until the process exits and return its `ExitStatus`
/// - `try_wait`: poll the process exit code without blocking
/// - `id`: return the process ID (PID)
///
/// This type is specific to Windows because it relies on Windows APIs and FFI bindings.
pub struct Child {
    /// Raw `PROCESS_INFORMATION` filled by `CreateProcessW`.
    ///
    /// This includes:
    /// - handles to the created process and its primary thread
    /// - numeric PID/TID values
    pub(crate) process_information: PROCESS_INFORMATION,
}

impl Child {
    /// Internal constructor used by `Command` to create a child process.
    ///
    /// # Parameters
    /// - `command`: The program/command line to execute. It is passed to `CreateProcessW`
    ///   as a UTF-16 (wide) command line buffer.
    /// - `inherit_handles`: If `true`, enables handle inheritance for the child process.
    ///   This is implemented by passing `SECURITY_ATTRIBUTES { bInheritHandle: TRUE }`.
    /// - `current_directory`: If provided, becomes the child process working directory
    ///   (passed as `lpCurrentDirectory`).
    /// - `env_clear`: If `true`, indicates the environment should be cleared before applying
    ///   `env_vars` (behavior implemented in `build_env_block`).
    /// - `env_vars`: Environment variable overrides/additions. Each item is:
    ///   - key/value pair where
    ///   - the value may be `None` to indicate removal/unsetting (again implemented
    ///   in `build_env_block`).
    ///
    /// # Returns
    /// - `Ok(Child)` if `CreateProcessW` succeeds.
    /// - `Err(std::io::Error)` if the underlying Windows call fails.
    ///
    /// # How environment affects creation flags
    /// This code sets `process_creation_flags` to `CREATE_UNICODE_ENVIRONMENT` only when
    /// an environment block pointer is non-null. Otherwise the process is created with a
    /// null environment pointer, which lets Windows use inherited/default behavior.
    pub(crate) fn new(
        command: &OsStr,
        inherit_handles: bool,
        current_directory: Option<&Path>,
        env_clear: bool,
        env_vars: Vec<(OsString, Option<OsString>)>,
    ) -> Result<Self, Error> {
        // Initialize the Windows structs with default "not set" values.
        let mut startup_information = STARTUPINFOW::default();
        let mut process_information = PROCESS_INFORMATION::default();

        // Windows expects the caller to fill `cb` with the size of the struct.
        startup_information.cb = size_of::<STARTUPINFOW>() as u32;

        // Build an environment block (or decide to use `NULL` for default behavior).
        let env_block = build_env_block(env_clear, env_vars);
        let lp_env_ptr = env_block
            .as_ref()
            .map(|b| b.as_ptr() as *mut c_void)
            .unwrap_or(null_mut());

        // `CREATE_UNICODE_ENVIRONMENT` is required for a Unicode environment block.
        let process_creation_flags = if lp_env_ptr.is_null() {
            0
        } else {
            CREATE_UNICODE_ENVIRONMENT
        };

        // If handle inheritance is requested, create security attributes and pass them
        // for both process/thread attributes.
        let mut security_attributes;
        let (lp_process_attributes, lp_thread_attributes) = if inherit_handles {
            security_attributes = SECURITY_ATTRIBUTES::new(true);
            (
                &mut security_attributes as *mut SECURITY_ATTRIBUTES,
                &mut security_attributes as *mut SECURITY_ATTRIBUTES,
            )
        } else {
            (null_mut(), null_mut())
        };

        // Convert optional working directory to a wide string buffer ending with `0`.
        let current_directory_ptr = current_directory
            .map(|path| {
                let wide_path: Vec<u16> = path
                    .as_os_str()
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                wide_path.as_ptr()
            })
            .unwrap_or(std::ptr::null_mut());

        // Convert the command to a wide, null-terminated command line buffer.
        // NOTE: The created Vec must live until after the `CreateProcessW` call returns.
        let command_wide = command.encode_wide().chain(std::iter::once(0)).collect::<Vec<_>>();

        // Call into the Windows API.
        let res = unsafe {
            CreateProcessW(
                null(),
                command_wide.as_ptr() as PWSTR,
                lp_process_attributes,
                lp_thread_attributes,
                inherit_handles as BOOL,
                process_creation_flags as DWORD,
                lp_env_ptr,
                current_directory_ptr as PCWSTR,
                &startup_information,
                &mut process_information,
            )
        };

        if res != 0 {
            Ok(Self {
                process_information,
            })
        } else {
            Err(Error::last_os_error())
        }
    }

    /// Terminates the child process.
    ///
    /// This is implemented via `TerminateProcess`.
    ///
    /// # Parameters
    /// - None.
    ///
    /// # Returns
    /// - `Ok(())` if termination succeeds.
    /// - `Err(std::io::Error)` if `TerminateProcess` fails.
    ///
    /// # Notes
    /// - The exit code passed to `TerminateProcess` is always `0` in this implementation.
    pub fn kill(&self) -> Result<(), Error> {
        // TerminateProcess returns non-zero on success.
        let res = unsafe { TerminateProcess(self.process_information.hProcess, 0 as UINT) };

        if res != 0 {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    }

    /// Waits for the child process to exit, then returns its `ExitStatus`.
    ///
    /// # Returns
    /// - `Ok(ExitStatus)` once the process has exited.
    /// - `Err(std::io::Error)` if waiting fails or retrieving the exit code fails.
    ///
    /// # How it works
    /// 1. Calls `WaitForSingleObject(hProcess, INFINITE)` to block until the process handle
    ///    is signaled.
    /// 2. If signaled, calls `GetExitCodeProcess` to retrieve the process exit code.
    /// 3. Closes both the process and primary thread handles using `CloseHandle`.
    ///
    /// # Resource handling
    /// This method closes `hProcess` and `hThread` once it successfully obtains the exit
    /// code. If you plan to call other methods afterward, prefer `try_wait` patterns
    /// or avoid further use after a successful `wait`.
    pub fn wait(&self) -> Result<ExitStatus, Error> {
        let mut exit_code = 0;

        let wait =
            unsafe { WaitForSingleObject(self.process_information.hProcess, INFINITE) == WAIT_OBJECT_0 };

        if wait {
            let res = unsafe {
                GetExitCodeProcess(self.process_information.hProcess, &mut exit_code as PDWORD)
            };

            if res != 0 {
                // Handles are no longer needed once the process is known to have exited.
                unsafe {
                    CloseHandle(self.process_information.hProcess);
                    CloseHandle(self.process_information.hThread);
                }

                Ok(ExitStatus(exit_code))
            } else {
                Err(Error::last_os_error())
            }
        } else {
            Err(Error::last_os_error())
        }
    }

    /// Attempts to retrieve the child process exit code without blocking.
    ///
    /// # Returns
    /// - `Ok(None)` if the process is still running.
    /// - `Ok(Some(ExitStatus))` if the process has exited.
    /// - `Err(std::io::Error)` if retrieving the exit code fails.
    ///
    /// # How it works
    /// This method calls `GetExitCodeProcess`:
    /// - If the exit code equals `STATUS_PENDING`, the process has not exited yet.
    /// - Otherwise, it treats the returned value as the final exit code and closes the
    ///   process and thread handles.
    pub fn try_wait(&self) -> Result<Option<ExitStatus>, Error> {
        let mut exit_code = 0;

        let res =
            unsafe { GetExitCodeProcess(self.process_information.hProcess, &mut exit_code as PDWORD) };

        if res != 0 {
            if exit_code == STATUS_PENDING {
                Ok(None)
            } else {
                // Once we have a final exit code, close handles.
                unsafe {
                    CloseHandle(self.process_information.hProcess);
                    CloseHandle(self.process_information.hThread);
                }

                Ok(Some(ExitStatus(exit_code)))
            }
        } else {
            Err(Error::last_os_error())
        }
    }

    /// Returns the process identifier (PID) of the child.
    ///
    /// # Returns
    /// The PID as a `u32`.
    pub fn id(&self) -> u32 {
        self.process_information.dwProcessId
    }
}
