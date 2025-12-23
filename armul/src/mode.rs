//! The ARM7TDMI supports seven modes of operation.
//! This file describes these modes.

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
