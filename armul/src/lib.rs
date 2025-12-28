pub mod assemble;
pub mod instr;
pub mod memory;
pub mod mode;
pub mod processor;
pub mod registers;
#[cfg(test)]
pub mod test;

#[cfg(test)]
include!(concat!(env!("OUT_DIR"), "/tests.rs"));
