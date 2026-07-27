use std::fmt;

/// Exit status returned by a finished child process.
///
/// This is a thin wrapper around the `u32` exit code reported by Windows
/// `GetExitCodeProcess`.
///
/// By convention in many ecosystems:
/// - `0` typically means success
/// - any non-zero value indicates some kind of failure or termination code
///
/// # Examples
/// ```rust
/// use your_crate::ExitStatus;
///
/// let status = ExitStatus(0);
/// assert!(status.success());
/// assert_eq!(status.code(), 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitStatus(pub u32);

impl ExitStatus {
    /// Returns whether the process exit code indicates success.
    ///
    /// # Returns
    /// - `true` if the underlying exit code is `0`
    /// - `false` otherwise
    pub fn success(&self) -> bool {
        self.0 == 0
    }

    /// Returns the raw numeric exit code.
    ///
    /// # Returns
    /// The wrapped `u32` exit code.
    pub fn code(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for ExitStatus {
    /// Formats the exit status as its numeric exit code.
    ///
    /// This prints the underlying `u32` value (e.g. `0`, `1`, `42`, ...).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
