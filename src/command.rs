use std::{
    env,
    ffi::{OsStr, OsString},
    io::Error,
    path::{Path, PathBuf},
};

use crate::{child::Child, exit_status::ExitStatus};

#[derive(Debug)]
pub struct Command {
    command: OsString,
    inherit_handles: bool,
    current_directory: Option<PathBuf>,
    env_clear: bool,
    env_vars: Vec<(OsString, Option<OsString>)>,
}

impl Command {
    pub fn new(command: impl Into<OsString>) -> Self {
        Self {
            command: command.into(),
            inherit_handles: false,
            current_directory: None,
            env_clear: false,
            env_vars: Vec::new(),
        }
    }

    pub fn inherit_handles(&mut self, inherit: bool) -> &mut Self {
        self.inherit_handles = inherit;
        self
    }

    pub fn current_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.current_directory = Some(dir.into());
        self
    }

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

    pub fn env_remove<K>(&mut self, key: K) -> &mut Self
    where
        K: AsRef<OsStr>,
    {
        self.env_vars.push((key.as_ref().to_os_string(), None));
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.env_clear = true;
        self.env_vars.clear();
        self
    }

    pub fn spawn(&mut self) -> Result<Child, Error> {
        Child::new(
            &self.command,
            self.inherit_handles,
            self.current_directory.as_deref(),
            self.env_clear,
            std::mem::take(&mut self.env_vars),
        )
    }

    pub fn status(&mut self) -> Result<ExitStatus, Error> {
        self.spawn()?.wait()
    }
}
