//! The ARM7TDMI supports seven modes of operation.
//! This file describes these modes.

use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// THe normal ARM program execution state.
    Usr,
    /// Designed to support a data transfer or channel process.
    Fiq,
    /// Used for general-purpose interrupt handling.
    Irq,
    /// Protected mode for the operating system.
    Supervisor,
    /// Entered after a data or instruction prefetch abort.
    Abort,
    /// A privileged user mode for the operating system.
    System,
    /// Entered when an undefined instruction is executed.
    Undefined,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Usr => write!(f, "usr"),
            Mode::Fiq => write!(f, "fiq"),
            Mode::Irq => write!(f, "irq"),
            Mode::Supervisor => write!(f, "svc"),
            Mode::Abort => write!(f, "abt"),
            Mode::System => write!(f, "sys"),
            Mode::Undefined => write!(f, "und"),
        }
    }
}
