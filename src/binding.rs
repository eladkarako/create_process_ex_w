#![allow(
    clippy::upper_case_acronyms,
    non_snake_case,
    non_camel_case_types,
    unused_doc_comments
)]
use std::{ffi::c_void, mem::size_of, ptr::null_mut};

/// Common Windows API constant used with `WaitForSingleObject`.
///
/// This value instructs the function to wait indefinitely for the provided handle
/// to become signaled.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject#parameters
pub(crate) const INFINITE: u32 = 0xFFFFFFFF;

/// Constant indicating the wait completed because the handle was signaled.
///
/// When `WaitForSingleObject` returns, `WAIT_OBJECT_0` represents a successful
/// wait on the specified handle.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject#return-value
pub(crate) const WAIT_OBJECT_0: u32 = 0x00000000;

/// Status code indicating that a process is still pending (i.e., not terminated yet).
///
/// This is commonly returned from `GetExitCodeProcess` until the process finishes.
/// When the exit code equals `STATUS_PENDING`, the process has not exited.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
pub(crate) const STATUS_PENDING: u32 = 0x00000103;

/// Process creation flag that tells Windows to use the environment block from
/// the current Unicode environment.
///
/// This is a bitmask used with `CreateProcessW`'s `dwCreationFlags`.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getexitcodeprocess#remarks
/// and:
/// https://learn.microsoft.com/en-us/windows/win32/api/procthread/process-creation-flags
pub(crate) const CREATE_UNICODE_ENVIRONMENT: DWORD = 0x00000400;

/// Handle inheritance flag used in tests.
///
/// When enabled, Windows will allow child processes created by the current
/// process to inherit handles marked as inheritable.
///
/// Note: this constant is only compiled for tests.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-gethandleinformation#parameters
#[cfg(test)]
pub(crate) const HANDLE_FLAG_INHERIT: u32 = 0x00000001;

/// Windows `BOOL` type.
///
/// In the Windows API, `BOOL` is a 32-bit signed integer used as a boolean:
/// `0` represents `FALSE`, and non-zero represents `TRUE`.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#BOOL
pub(crate) type BOOL = i32;

/// Windows `DWORD` type.
///
/// `DWORD` is a 32-bit unsigned integer commonly used for counts, timeouts,
/// flags, and process/thread identifiers.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#DWORD
pub(crate) type DWORD = u32;

/// Windows `PCWSTR` type (pointer to constant wide string).
///
/// `PCWSTR` is a raw pointer to UTF-16 code units (wide string) that the
/// called Windows function does not modify.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PCWSTR
pub(crate) type PCWSTR = *const u16;

/// Windows `PDWORD` type (pointer to DWORD).
///
/// `PDWORD` points to a `DWORD` value that the called Windows function writes to.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PDWORD
pub(crate) type PDWORD = *mut u32;

/// Windows `PVOID` type (generic pointer).
///
/// This is an opaque pointer (`void*` in C terms).
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PVOID
type PVOID = *mut c_void;

/// Windows `PWSTR` type (pointer to writable wide string).
///
/// `PWSTR` is a raw pointer to UTF-16 code units (wide string) that the called
/// Windows function may modify.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PWSTR
pub(crate) type PWSTR = *mut u16;

/// Windows `UINT` type.
///
/// `UINT` is a 32-bit unsigned integer commonly used for exit codes, flags, and other parameters.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#UINT
pub(crate) type UINT = u32;

/// Windows `HANDLE` type.
///
/// `HANDLE` is an opaque pointer-like value used by the Windows API to refer to kernel objects
/// (processes, threads, events, mutexes, files, etc.).
///
/// In this crate it is represented as a raw pointer.
type HANDLE = *mut c_void;

/// Windows `PBYTE` type (pointer to BYTE).
///
/// `PBYTE` is a pointer to a sequence of bytes (unsigned 8-bit values).
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PBYTE
type PBYTE = *mut u8;

/// Windows `WORD` type.
///
/// `WORD` is a 16-bit unsigned integer used for smaller fields within Windows structs.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#WORD
pub(crate) type WORD = u16;

/// Bitmask type for `STARTUPINFOW.dwFlags`.
pub(crate) type STARTUPINFOW_FLAGS = DWORD;

/// When set in `STARTUPINFOW.dwFlags`, Windows uses `STARTUPINFOW.wShowWindow`
/// to determine how the created process's primary window should be shown.
pub(crate) const STARTF_USESHOWWINDOW: STARTUPINFOW_FLAGS = 0x00000001;

/// Value to place into `STARTUPINFOW.wShowWindow` when creating a new process.
///
/// Windows uses this to decide how the primary window of the created process
/// should be shown (minimized, maximized, restored, hidden, etc.).
///
/// This is represented as a raw `u16` because `wShowWindow` in `STARTUPINFOW` is
/// defined as an integer value, and multiple named constants may map to the same
/// underlying numeric value (e.g. `SW_NORMAL` and `SW_SHOWNORMAL` are both `1`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShowWindowCommand(pub(crate) u16);

impl ShowWindowCommand {
    /// Hides the window and activates another window.
    pub const SW_HIDE: Self = Self(0);

    /// Activates and displays a window.
    ///
    /// If the window is minimized, maximized, or arranged, the system restores it to its
    /// original size and position. An application should specify this flag when displaying
    /// the window for the first time.
    pub const SW_SHOWNORMAL: Self = Self(1);

    /// Alias for `SW_SHOWNORMAL`.
    ///
    /// Same numeric value: 1.
    pub const SW_NORMAL: Self = Self(1);

    /// Activates the window and displays it as a minimized window.
    pub const SW_SHOWMINIMIZED: Self = Self(2);

    /// Activates the window and displays it as a maximized window.
    pub const SW_SHOWMAXIMIZED: Self = Self(3);

    /// Alias for `SW_SHOWMAXIMIZED`.
    ///
    /// Same numeric value: 3.
    pub const SW_MAXIMIZE: Self = Self(3);

    /// Displays a window in its most recent size and position.
    ///
    /// This value is similar to `SW_SHOWNORMAL`, except that the window is not activated.
    pub const SW_SHOWNOACTIVATE: Self = Self(4);

    /// Activates and displays the window in its current size and position.
    pub const SW_SHOW: Self = Self(5);

    /// Minimizes the specified window and activates the next top-level window in the Z order.
    pub const SW_MINIMIZE: Self = Self(6);

    /// Displays the window as a minimized window.
    ///
    /// This value is similar to `SW_SHOWMINIMIZED`, except the window is not activated.
    pub const SW_SHOWMINNOACTIVE: Self = Self(7);

    /// Displays the window in its current size and position.
    ///
    /// This value is similar to `SW_SHOW`, except that the window is not activated.
    pub const SW_SHOWNA: Self = Self(8);

    /// Activates and displays the window.
    ///
    /// If the window is minimized, maximized, or arranged, the system restores it to its
    /// original size and position. An application should specify this flag when restoring a
    /// minimized window.
    pub const SW_RESTORE: Self = Self(9);

    /// Sets the show state based on the `SW_` value specified in the `STARTUPINFO` structure
    /// passed to `CreateProcess` by the program that started the application.
    pub const SW_SHOWDEFAULT: Self = Self(10);

    /// Minimizes a window even if the thread that owns the window is not responding.
    ///
    /// This flag should only be used when minimizing windows from a different thread.
    pub const SW_FORCEMINIMIZE: Self = Self(11);

    #[inline]
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Windows `PROCESS_INFORMATION` structure.
///
/// This structure is filled in by `CreateProcessW` and provides information
/// about the newly created process and its primary thread.
///
/// Fields:
/// - `hProcess`: Handle to the newly created process.
/// - `hThread`: Handle to the primary thread of the newly created process.
/// - `dwProcessId`: Process identifier (PID).
/// - `dwThreadId`: Thread identifier (TID).
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-process_information
#[repr(C)]

pub(crate) struct PROCESS_INFORMATION {
    /// Handle to the newly created process.
    pub hProcess: HANDLE,
    /// Handle to the primary thread of the newly created process.
    pub hThread: HANDLE,
    /// Process identifier (PID).
    pub dwProcessId: DWORD,
    /// Thread identifier (TID).
    dwThreadId: DWORD,
}

impl Default for PROCESS_INFORMATION {
    /// Creates a zero/NULL-initialized `PROCESS_INFORMATION`.
    ///
    /// This is useful as a starting point before calling `CreateProcessW`.
    /// After `CreateProcessW` returns successfully, the relevant fields will be
    /// overwritten by Windows.
    fn default() -> Self {
        Self {
            hProcess: null_mut(),
            hThread: null_mut(),
            dwProcessId: 0,
            dwThreadId: 0,
        }
    }
}

/// Windows `SECURITY_ATTRIBUTES` structure.
///
/// This structure controls security-related attributes for objects created
/// by functions that accept security attribute parameters (e.g., `CreateProcessW`).
///
/// Fields:
/// - `nLength`: Size of the structure in bytes.
/// - `lpSecurityDescriptor`: Optional pointer to a security descriptor; `NULL` usually means
///   "use default security".
/// - `bInheritHandle`: Whether created handles should be inheritable by child processes.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/api/wtypesbase/ns-wtypesbase-security_attributes
#[repr(C)]
pub(crate) struct SECURITY_ATTRIBUTES {
    nLength: DWORD,
    lpSecurityDescriptor: PVOID,
    bInheritHandle: BOOL,
}

impl SECURITY_ATTRIBUTES {
    /// Creates a `SECURITY_ATTRIBUTES` with configurable handle inheritance.
    ///
    /// # Parameters
    /// - `inherit_handles`: If `true`, sets `bInheritHandle` to a non-zero `BOOL` value,
    ///   allowing the created handles to be inheritable by child processes.
    ///   If `false`, sets it to `0`.
    ///
    /// # Notes
    /// - `lpSecurityDescriptor` is set to `NULL`.
    /// - `nLength` is set to the size of `SECURITY_ATTRIBUTES` as required by Windows.
    pub(crate) fn new(inherit_handles: bool) -> Self {
        Self {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as DWORD,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: inherit_handles as BOOL,
        }
    }
}

/// Windows `STARTUPINFOW` structure.
///
/// This structure provides information about how to start a new process,
/// including window settings and standard handle redirection.
///
/// Only some fields are typically needed; the rest can be left as zero/NULL.
/// This crate provides a `Default` implementation to initialize all fields
/// to safe "not set" values.
///
/// See:
/// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfow
#[repr(C)]
pub(crate) struct STARTUPINFOW {
    /// Size of the structure, in bytes.
    pub cb: DWORD,
    /// Reserved; should typically be `NULL`.
    lpReserved: PWSTR,
    /// Optional name of the desktop for the new process; `NULL` uses default.
    lpDesktop: PWSTR,
    /// Optional title for the new process; `NULL` leaves it unset.
    lpTitle: PWSTR,
    /// X coordinate of the window position.
    dwX: DWORD,
    /// Y coordinate of the window position.
    dwY: DWORD,
    /// X size of the window.
    dwXSize: DWORD,
    /// Y size of the window.
    dwYSize: DWORD,
    /// Number of character columns in the window buffer.
    dwXCountChars: DWORD,
    /// Number of character rows in the window buffer.
    dwYCountChars: DWORD,
    /// Fill attribute for the window (e.g., text/background attributes).
    dwFillAttribute: DWORD,
    /// Flags specifying which members contain valid data.
    pub(crate) dwFlags: DWORD,
    /// How the window is shown (e.g., normal/minimized).
    pub(crate) wShowWindow: WORD,
    /// Reserved; should typically be `0` or `NULL`-like values.
    cbReserved2: WORD,
    /// Reserved; should typically be `NULL`.
    lpReserved2: PBYTE,
    /// Handle for standard input redirection.
    hStdInput: HANDLE,
    /// Handle for standard output redirection.
    hStdOutput: HANDLE,
    /// Handle for standard error redirection.
    hStdError: HANDLE,
}

impl Default for STARTUPINFOW {
    /// Creates a `STARTUPINFOW` with all fields zero/NULL-initialized, except `cb`.
    ///
    /// # Why `cb` is set
    /// Many Windows structs require their `cb` field to be set to `size_of::<Self>()`
    /// before calling Windows API functions.
    fn default() -> Self {
        Self {
            cb: size_of::<STARTUPINFOW>() as DWORD,
            lpReserved: null_mut(),
            lpDesktop: null_mut(),
            lpTitle: null_mut(),
            dwX: 0,
            dwY: 0,
            dwXSize: 0,
            dwYSize: 0,
            dwXCountChars: 0,
            dwYCountChars: 0,
            dwFillAttribute: 0,
            dwFlags: 0,
            wShowWindow: 0,
            cbReserved2: 0,
            lpReserved2: null_mut(),
            hStdInput: null_mut(),
            hStdOutput: null_mut(),
            hStdError: null_mut(),
        }
    }
}

/// Windows FFI bindings to process/thread management functions.
///
/// These functions are declared with `extern "system"` to match the Windows calling convention.
/// They are intended to be used by higher-level wrapper code in the crate.
///
/// # Safety
/// Each function here is `unsafe` to call from Rust because they interface with FFI:
/// - arguments must be valid pointers where required,
/// - structures must have correct layout (`#[repr(C)]`),
/// - buffers/strings must live long enough and be valid for the duration of the call.
/// The wrapper code in the crate should ensure these invariants.
///
/// See:
/// Individual function documentation links are embedded inline below.
extern "system" {
    /// Closes an open handle to a Windows object.
    ///
    /// This decrements the reference count for the underlying handle-managed resource.
    ///
    /// # Parameters
    /// - `hObject`: Handle to close.
    ///
    /// # Returns
    /// Returns non-zero (`TRUE`) on success, and zero (`FALSE`) on failure.
    ///
    /// See:
    /// https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle
    pub(crate) fn CloseHandle(hObject: HANDLE) -> BOOL;

    /// Creates a new process.
    ///
    /// This is the core API used to start a child process given a command line and startup info.
    ///
    /// # Parameters (high-level meaning)
    /// - `lpApplicationName`: Optional executable path; can be `NULL` if `lpCommandLine` contains it.
    /// - `lpCommandLine`: The command line to execute (mutable pointer in Windows API).
    /// - `lpProcessAttributes` / `lpThreadAttributes`: Optional security attributes.
    /// - `bInheritHandles`: Whether handles are inheritable in the child process.
    /// - `dwCreationFlags`: Process creation options.
    /// - `lpEnvironment`: Optional environment block; can be `NULL` to inherit the parent's environment.
    /// - `lpCurrentDirectory`: Optional working directory; can be `NULL`.
    /// - `lpStartupInfo`: Startup configuration (e.g., stdio handles, window settings).
    /// - `lpProcessInformation`: Receives process/thread handles and IDs.
    ///
    /// # Returns
    /// Returns non-zero (`TRUE`) on success, and zero (`FALSE`) on failure.
    ///
    /// See:
    /// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw
    pub(crate) fn CreateProcessW(
        lpApplicationName: PCWSTR,
        lpCommandLine: PWSTR,
        lpProcessAttributes: *mut SECURITY_ATTRIBUTES,
        lpThreadAttributes: *mut SECURITY_ATTRIBUTES,
        bInheritHandles: BOOL,
        dwCreationFlags: DWORD,
        lpEnvironment: PVOID,
        lpCurrentDirectory: PCWSTR,
        lpStartupInfo: *const STARTUPINFOW,
        lpProcessInformation: *mut PROCESS_INFORMATION,
    ) -> BOOL;

    /// Retrieves the exit code of a process.
    ///
    /// If the process is still running, the exit code is `STATUS_PENDING`.
    /// Otherwise, the exit code returned is the process' termination code.
    ///
    /// # Parameters
    /// - `hProcess`: Handle to the process.
    /// - `lpExitCode`: Output pointer to receive the exit code.
    ///
    /// # Returns
    /// Returns non-zero (`TRUE`) on success, and zero (`FALSE`) on failure.
    ///
    /// See:
    /// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getexitcodeprocess
    pub(crate) fn GetExitCodeProcess(hProcess: HANDLE, lpExitCode: PDWORD) -> BOOL;

    /// Terminates (kills) a process.
    ///
    /// # Parameters
    /// - `hProcess`: Handle to the process to terminate.
    /// - `uExitCode`: Exit code to report to the process' parent/observers.
    ///
    /// # Returns
    /// Returns non-zero (`TRUE`) on success, and zero (`FALSE`) on failure.
    ///
    /// See:
    /// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess
    pub(crate) fn TerminateProcess(hProcess: HANDLE, uExitCode: UINT) -> BOOL;

    /// Waits until the specified object is in a signaled state or a timeout occurs.
    ///
    /// This is commonly used to wait for a process handle to become signaled
    /// (i.e., for the process to exit).
    ///
    /// # Parameters
    /// - `hHandle`: Handle to wait on (commonly a process handle).
    /// - `dwMilliseconds`: Timeout in milliseconds. Use `INFINITE` to wait forever.
    ///
    /// # Returns
    /// Returns one of the documented wait result constants (e.g., `WAIT_OBJECT_0`).
    ///
    /// See:
    /// https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject
    pub(crate) fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;

    /// Retrieves information about a handle.
    ///
    /// This is used in tests (compiled only under `cfg(test)` in this crate).
    ///
    /// # Parameters
    /// - `hObject`: Handle to query.
    /// - `lpdwFlags`: Output pointer receiving flags describing the handle.
    ///
    /// # Returns
    /// Returns non-zero (`TRUE`) on success, and zero (`FALSE`) on failure.
    ///
    /// See:
    /// https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-gethandleinformation
    #[cfg(test)]
    pub(crate) fn GetHandleInformation(hObject: HANDLE, lpdwFlags: PDWORD) -> BOOL;
}
