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

#[derive(Debug)]
pub struct Child {
    pub(crate) process_information: PROCESS_INFORMATION,
}

impl Child {
    pub(crate) fn new(
        command: &OsStr,
        inherit_handles: bool,
        current_directory: Option<&Path>,
        env_clear: bool,
        env_vars: Vec<(OsString, Option<OsString>)>,
    ) -> Result<Self, Error> {
        let mut startup_information = STARTUPINFOW::default();
        let mut process_information = PROCESS_INFORMATION::default();

        startup_information.cb = size_of::<STARTUPINFOW>() as u32;

        let env_block = build_env_block(env_clear, env_vars);
        let lp_env_ptr = env_block
            .as_ref()
            .map(|b| b.as_ptr() as *mut c_void)
            .unwrap_or(null_mut());

        let process_creation_flags = if lp_env_ptr.is_null() {
            0
        } else {
            CREATE_UNICODE_ENVIRONMENT
        };

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

        // NOTE: This mirrors your original code; it’s safe for the call
        // because we pass command.as_ptr() directly into CreateProcessW.
        let command_wide = command.encode_wide().chain(std::iter::once(0)).collect::<Vec<_>>();

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

    pub fn kill(&self) -> Result<(), Error> {
        let res = unsafe { TerminateProcess(self.process_information.hProcess, 0 as UINT) };

        if res != 0 {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    }

    pub fn wait(&self) -> Result<ExitStatus, Error> {
        let mut exit_code = 0;

        let wait =
            unsafe { WaitForSingleObject(self.process_information.hProcess, INFINITE) == WAIT_OBJECT_0 };

        if wait {
            let res = unsafe {
                GetExitCodeProcess(self.process_information.hProcess, &mut exit_code as PDWORD)
            };

            if res != 0 {
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

    pub fn try_wait(&self) -> Result<Option<ExitStatus>, Error> {
        let mut exit_code = 0;

        let res =
            unsafe { GetExitCodeProcess(self.process_information.hProcess, &mut exit_code as PDWORD) };

        if res != 0 {
            if exit_code == STATUS_PENDING {
                Ok(None)
            } else {
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

    pub fn id(&self) -> u32 {
        self.process_information.dwProcessId
    }
}
