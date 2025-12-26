//! Describes the physical registers in the processor's hardware.

use std::fmt::Display;

use num_derive::FromPrimitive;

use crate::{
    instr::{Cond, Psr, Register},
    mode::Mode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive)]
#[repr(u8)]
pub enum PhysicalRegister {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    R8Fiq,
    R9Fiq,
    R10Fiq,
    R11Fiq,
    R12Fiq,
    R13Fiq,
    R14Fiq,
    R13Svc,
    R14Svc,
    R13Abt,
    R14Abt,
    R13Irq,
    R14Irq,
    R13Und,
    R14Und,
    Cpsr,
    SpsrFiq,
    SpsrSvc,
    SpsrAbt,
    SpsrIrq,
    SpsrUnd,
}

impl Register {
    pub fn physical(self, mode: Mode) -> PhysicalRegister {
        match (self, mode) {
            (Register::R0, _) => PhysicalRegister::R0,
            (Register::R1, _) => PhysicalRegister::R1,
            (Register::R2, _) => PhysicalRegister::R2,
            (Register::R3, _) => PhysicalRegister::R3,
            (Register::R4, _) => PhysicalRegister::R4,
            (Register::R5, _) => PhysicalRegister::R5,
            (Register::R6, _) => PhysicalRegister::R6,
            (Register::R7, _) => PhysicalRegister::R7,
            (Register::R8, Mode::Fiq) => PhysicalRegister::R8Fiq,
            (Register::R9, Mode::Fiq) => PhysicalRegister::R9Fiq,
            (Register::R10, Mode::Fiq) => PhysicalRegister::R10Fiq,
            (Register::R11, Mode::Fiq) => PhysicalRegister::R11Fiq,
            (Register::R12, Mode::Fiq) => PhysicalRegister::R12Fiq,
            (Register::R8, _) => PhysicalRegister::R8,
            (Register::R9, _) => PhysicalRegister::R9,
            (Register::R10, _) => PhysicalRegister::R10,
            (Register::R11, _) => PhysicalRegister::R11,
            (Register::R12, _) => PhysicalRegister::R12,
            (Register::R13, Mode::Usr | Mode::System) => PhysicalRegister::R13,
            (Register::R13, Mode::Fiq) => PhysicalRegister::R13Fiq,
            (Register::R13, Mode::Supervisor) => PhysicalRegister::R13Svc,
            (Register::R13, Mode::Abort) => PhysicalRegister::R13Abt,
            (Register::R13, Mode::Irq) => PhysicalRegister::R13Irq,
            (Register::R13, Mode::Undefined) => PhysicalRegister::R13Und,
            (Register::R14, Mode::Usr | Mode::System) => PhysicalRegister::R14,
            (Register::R14, Mode::Fiq) => PhysicalRegister::R14Fiq,
            (Register::R14, Mode::Supervisor) => PhysicalRegister::R14Svc,
            (Register::R14, Mode::Abort) => PhysicalRegister::R14Abt,
            (Register::R14, Mode::Irq) => PhysicalRegister::R14Irq,
            (Register::R14, Mode::Undefined) => PhysicalRegister::R14Und,
            (Register::R15, _) => PhysicalRegister::R15,
        }
    }
}

impl Psr {
    /// Get the physical register corresponding to the given mode.
    /// Note that there is no SPSR in user or system mode.
    pub fn physical(self, mode: Mode) -> Option<PhysicalRegister> {
        match (self, mode) {
            (Psr::Cpsr, _) => Some(PhysicalRegister::Cpsr),
            (Psr::Spsr, Mode::Usr | Mode::System) => None,
            (Psr::Spsr, Mode::Fiq) => Some(PhysicalRegister::SpsrFiq),
            (Psr::Spsr, Mode::Irq) => Some(PhysicalRegister::SpsrIrq),
            (Psr::Spsr, Mode::Supervisor) => Some(PhysicalRegister::SpsrSvc),
            (Psr::Spsr, Mode::Abort) => Some(PhysicalRegister::SpsrAbt),
            (Psr::Spsr, Mode::Undefined) => Some(PhysicalRegister::SpsrUnd),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Registers {
    /// 31 general-purpose data registers and 6 status registers.
    regs: [u32; 37],
}

impl Default for Registers {
    fn default() -> Self {
        let mut this = Self { regs: [0; 37] };
        // Set supervisor mode with IRQ disabled.
        *this.cpsr_mut() = 0b10010011;
        this
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Mode {}  Flags ",
            self.mode()
                .map_or_else(|| "???".to_owned(), |x| x.to_string())
        )?;
        if self.negative() {
            write!(f, "N")?;
        }
        if self.zero() {
            write!(f, "Z")?;
        }
        if self.carry() {
            write!(f, "C")?;
        }
        if self.overflow() {
            write!(f, "V")?;
        }
        if self.irq_disable() {
            write!(f, "I")?;
        }
        if self.fiq_disable() {
            write!(f, "F")?;
        }
        if self.thumb_state() {
            write!(f, "T")?;
        }
        writeln!(f)?;
        write!(f, "R0  {:0>8X}  ", self.get(Register::R0))?;
        write!(f, "R1  {:0>8X}  ", self.get(Register::R1))?;
        write!(f, "R2  {:0>8X}  ", self.get(Register::R2))?;
        writeln!(f, "R3  {:0>8X}", self.get(Register::R3))?;
        write!(f, "R4  {:0>8X}  ", self.get(Register::R4))?;
        write!(f, "R5  {:0>8X}  ", self.get(Register::R5))?;
        write!(f, "R6  {:0>8X}  ", self.get(Register::R6))?;
        writeln!(f, "R7  {:0>8X}", self.get(Register::R7))?;
        write!(f, "R8  {:0>8X}  ", self.get(Register::R8))?;
        write!(f, "R9  {:0>8X}  ", self.get(Register::R9))?;
        write!(f, "R10 {:0>8X}  ", self.get(Register::R10))?;
        writeln!(f, "R11 {:0>8X}", self.get(Register::R11))?;
        write!(f, "R12 {:0>8X}  ", self.get(Register::R12))?;
        write!(f, "SP  {:0>8X}  ", self.get(Register::R13))?;
        write!(f, "LR  {:0>8X}  ", self.get(Register::R14))?;
        write!(f, "PC  {:0>8X}", self.get(Register::R15))?;
        Ok(())
    }
}

impl Registers {
    pub fn get_physical(&self, register: PhysicalRegister) -> u32 {
        self.regs[register as usize]
    }

    pub fn get_physical_mut(&mut self, register: PhysicalRegister) -> &mut u32 {
        &mut self.regs[register as usize]
    }

    /// Using the current mode of the processor, obtain the value of the given
    /// virtual register. In case of ill-defined mode, we default to the user mode.
    pub fn get(&self, register: Register) -> u32 {
        self.get_physical(register.physical(self.mode().unwrap_or(Mode::Usr)))
    }

    /// Get the value of the given register as in `Self::get`.
    /// But if `register` is `R15`, additionally add the given offset.
    pub fn get_pc_offset(&self, register: Register, pc_offset: u32) -> u32 {
        self.get(register)
            .wrapping_add(if register == Register::R15 {
                pc_offset
            } else {
                0
            })
    }

    /// Using the current mode of the processor, mutably borrow the given
    /// virtual register. In case of ill-defined mode, we default to the user mode.
    pub fn get_mut(&mut self, register: Register) -> &mut u32 {
        self.get_physical_mut(register.physical(self.mode().unwrap_or(Mode::Usr)))
    }

    /// Return the current program status register.
    pub fn cpsr(&self) -> u32 {
        self.get_physical(PhysicalRegister::Cpsr)
    }

    pub fn cpsr_mut(&mut self) -> &mut u32 {
        self.get_physical_mut(PhysicalRegister::Cpsr)
    }

    /// Get the current mode of the processor.
    /// If the CPSR had invalid mode bits, the processor has no definite mode.
    pub fn mode(&self) -> Option<Mode> {
        match self.cpsr() & 0b11111 {
            0b10000 => Some(Mode::Usr),
            0b10001 => Some(Mode::Fiq),
            0b10010 => Some(Mode::Irq),
            0b10011 => Some(Mode::Supervisor),
            0b10111 => Some(Mode::Abort),
            0b11011 => Some(Mode::Undefined),
            0b11111 => Some(Mode::System),
            _ => None,
        }
    }

    /// Test the N flag.
    pub fn negative(&self) -> bool {
        self.cpsr() & (1 << 31) != 0
    }

    pub fn set_negative(&mut self, set: bool) {
        set_bit(self.cpsr_mut(), 31, set);
    }

    /// Test the Z flag.
    pub fn zero(&self) -> bool {
        self.cpsr() & (1 << 30) != 0
    }

    pub fn set_zero(&mut self, set: bool) {
        set_bit(self.cpsr_mut(), 30, set);
    }

    /// Test the C flag.
    pub fn carry(&self) -> bool {
        self.cpsr() & (1 << 29) != 0
    }

    pub fn set_carry(&mut self, set: bool) {
        set_bit(self.cpsr_mut(), 29, set);
    }

    /// Test the V flag.
    pub fn overflow(&self) -> bool {
        self.cpsr() & (1 << 28) != 0
    }

    pub fn set_overflow(&mut self, set: bool) {
        set_bit(self.cpsr_mut(), 28, set);
    }

    /// Test the I bit.
    pub fn irq_disable(&self) -> bool {
        self.cpsr() & (1 << 7) != 0
    }

    pub fn set_irq_disable(&mut self, set: bool) {
        set_bit(self.cpsr_mut(), 7, set);
    }

    /// Test the F bit.
    pub fn fiq_disable(&self) -> bool {
        self.cpsr() & (1 << 6) != 0
    }

    pub fn set_fiq_disable(&mut self, set: bool) {
        set_bit(self.cpsr_mut(), 6, set);
    }

    /// Test the T bit.
    pub fn thumb_state(&self) -> bool {
        self.cpsr() & (1 << 5) != 0
    }

    pub fn set_thumb_state(&mut self, set: bool) {
        set_bit(self.cpsr_mut(), 5, set);
    }

    /// Returns true if the given condition holds.
    pub fn test_condition(&self, cond: Cond) -> bool {
        match cond {
            Cond::EQ => self.zero(),
            Cond::NE => !self.zero(),
            Cond::CS => self.carry(),
            Cond::CC => !self.carry(),
            Cond::MI => self.negative(),
            Cond::PL => !self.negative(),
            Cond::VS => self.overflow(),
            Cond::VC => !self.overflow(),
            Cond::HI => self.carry() && !self.zero(),
            Cond::LS => !self.carry() || self.zero(),
            Cond::GE => self.negative() == self.overflow(),
            Cond::LT => self.negative() != self.overflow(),
            Cond::GT => !self.zero() && (self.negative() == self.overflow()),
            Cond::LE => self.zero() || (self.negative() || self.overflow()),
            Cond::AL => true,
        }
    }
}

fn set_bit(value: &mut u32, bit: usize, set: bool) {
    if set {
        *value |= 1 << bit;
    } else {
        *value &= !(1 << bit);
    }
}
