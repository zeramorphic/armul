pub mod instr;
pub mod memory;
pub mod mode;
pub mod processor;
pub mod registers;

#[cfg(test)]
mod tests {
    use crate::{
        instr::Instr,
        processor::{Processor, test::TestProcessorListener},
    };

    #[test]
    fn test() {
        let mut proc = Processor::default();
        let mut listener = TestProcessorListener::default();
        let (cond, instr) = Instr::decode(0xE3A0DA01).unwrap();
        proc.try_execute(&mut listener).unwrap();
        panic!("{listener:#?}\n{proc:?}");
    }
}
