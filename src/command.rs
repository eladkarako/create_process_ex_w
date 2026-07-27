use std::{
    ffi::{OsStr, OsString},
    io::Error,
    path::PathBuf,
};

use crate::{child::Child, exit_status::ExitStatus};

/// Builder for launching a Windows process.
///
/// `Command` stores the configuration required to create a new process (program/command
/// line, working directory, handle inheritance, and environment handling), and then
/// spawns a [`Child`].
///
/// This crate uses Windows' `CreateProcessW` under the hood, so command execution is
/// inherently platform-specific.
///
/// # Examples
/// ```rust
/// use std::ffi::OsString;
/// use your_crate::{Command, ExitStatus};
///
/// let status: ExitStatus = Command::new("C:\\Windows\\System32\\cmd.exe")
///     .env("MY_VAR", OsString::from("hello"))
///     .status()
///     .unwrap();
/// println!("Process exited: {}", status.0);
/// ```
#[derive(Debug)]
pub struct Command {
    /// The wide command line / executable invocation passed to Windows.
    ///
    /// Despite the name, this is stored as an opaque `OsString` and later encoded
    /// to UTF-16 for `CreateProcessW`.
    command: OsString,

    /// Whether to allow handles to be inherited into the child process.
    inherit_handles: bool,

    /// Optional working directory for the child.
    current_directory: Option<PathBuf>,

    /// Whether to clear the child's environment before applying `env_vars`.
    env_clear: bool,

    /// Environment variable modifications to apply to the child.
    ///
    /// Each entry is `(key, value)` where:
    /// - `Some(value)` means set/override this variable
    /// - `None` means remove/unset this variable (behavior implemented in `env_block`)
    env_vars: Vec<(OsString, Option<OsString>)>,
}

impl Command {
    /// Creates a new process command builder.
    ///
    /// # Parameters
    /// - `command`: An executable path or command line (stored as an `OsString`).
    ///
    /// # Default configuration
    /// - `inherit_handles`: `false`
    /// - `current_directory`: unset (`None`)
    /// - environment: inherited/default, unless you call `env_clear`, `env`, `env_remove`, etc.
    pub fn new(command: impl Into<OsString>) -> Self {
        Self {
            command: command.into(),
            inherit_handles: false,
            current_directory: None,
            env_clear: false,
            env_vars: Vec::new(),
        }
    }

    /// Enables or disables handle inheritance for the child process.
    ///
    /// When inheritance is enabled, Windows may allow handles marked as inheritable
    /// in the parent to be used by the child.
    ///
    /// # Parameters
    /// - `inherit`: If `true`, sets the child process to inherit handles.
    pub fn inherit_handles(&mut self, inherit: bool) -> &mut Self {
        self.inherit_handles = inherit;
        self
    }

    /// Sets the child's current working directory.
    ///
    /// If `dir` is `Some`, the directory will be passed to Windows as the
    /// `lpCurrentDirectory` argument.
    ///
    /// # Parameters
    /// - `dir`: Working directory for the child process.
    pub fn current_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.current_directory = Some(dir.into());
        self
    }

    /// Sets or overrides a single environment variable for the child.
    ///
    /// # Parameters
    /// - `key`: Environment variable name.
    /// - `val`: Environment variable value.
    ///
    /// # Behavior
    /// - The pair is stored in `env_vars` and applied when spawning.
    /// - Call [`env_clear`](Self::env_clear) to request clearing the environment first.
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.env_vars.push((
            key.as_ref().to_os_string(),
            Some(val.as_ref().to_os_string()),
        ));
        self
    }

    /// Sets/overrides multiple environment variables for the child.
    ///
    /// # Parameters
    /// - `vars`: An iterator of `(key, value)` pairs.
    ///
    /// # Notes
    /// This method simply calls [`env`](Self::env) for each pair.
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (key, val) in vars {
            self.env(key, val);
        }
        self
    }

    /// Removes (unsets) an environment variable from the child.
    ///
    /// # Parameters
    /// - `key`: Environment variable name to remove.
    ///
    /// # Behavior
    /// Internally, this stores `(key, None)` and the actual unset behavior is implemented
    /// when the crate builds the environment block used by process creation.
    pub fn env_remove<K>(&mut self, key: K) -> &mut Self
    where
        K: AsRef<OsStr>,
    {
        self.env_vars.push((key.as_ref().to_os_string(), None));
        self
    }

    /// Clears the child's environment before applying the configured environment variables.
    ///
    /// # Behavior
    /// - Sets `env_clear = true`
    /// - Clears any previously stored environment modifications (`env_vars`)
    /// - After calling this, you can add new overrides/removals using [`env`](Self::env),
    ///   [`env_remove`](Self::env_remove), etc.
    pub fn env_clear(&mut self) -> &mut Self {
        self.env_clear = true;
        self.env_vars.clear();
        self
    }

    /// Spawns the child process using the current builder configuration.
    ///
    /// # Returns
    /// - `Ok(Child)` if process creation succeeds.
    /// - `Err(std::io::Error)` if `CreateProcessW` fails.
    ///
    /// # How configuration is applied
    /// - The stored `command` is passed to the Windows `CreateProcessW` call.
    /// - `inherit_handles` controls whether security attributes are passed with
    ///   handle inheritance enabled.
    /// - `current_directory` is passed as `lpCurrentDirectory` when present.
    /// - `env_clear` and `env_vars` are used to build an environment block during spawn.
    pub fn spawn(&mut self) -> Result<Child, Error> {
        Child::new(
            &self.command,
            self.inherit_handles,
            self.current_directory.as_deref(),
            self.env_clear,
            std::mem::take(&mut self.env_vars),
        )
    }

    /// Spawns the child and waits for it to exit, returning its [`ExitStatus`].
    ///
    /// # Returns
    /// - `Ok(ExitStatus)` once the process has exited.
    /// - `Err(std::io::Error)` if spawning or waiting fails.
    ///
    /// # Implementation detail
    /// This is a convenience wrapper around:
    /// - `self.spawn()?`
    /// - `.wait()`
    pub fn status(&mut self) -> Result<ExitStatus, Error> {
        self.spawn()?.wait()
    }
}
