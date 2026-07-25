use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitStatus(u32);

impl ExitStatus {
    pub fn success(&self) -> bool {
        self.0 == 0
    }

    pub fn code(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
