pub mod instr;
pub mod memory;
pub mod mode;
pub mod processor;
pub mod registers;

#[cfg(test)]
mod tests {
    use crate::{
        instr::{Instr, Register},
        processor::{Processor, test::TestProcessorListener},
    };

    /// A division routine from the ARM7TDMI data sheet.
    const DIVIDE: [u32; 15] = [
        0xE3A01025, // mov r1,#37
        0xE3A02006, // mov r2,#6
        0xE3A00001, // mov r0,#1
        0xE3520102, // div1 cmp r2,#0x80000000
        0x31520001, // cmpcc r2,r1
        0x31A02082, // movcc r2,r2,asl#1
        0x31A00080, // movcc r0,r0,asl#1
        0x3AFFFFFA, // bcc div1
        0xE3A03000, // mov r3,#0
        0xE1510002, // div2 cmp r1,r2
        0x20411002, // subcs r1,r1,r2
        0x20833000, // addcs r3,r3,r0
        0xE1B000A0, // movs r0,r0,lsr#1
        0x11A020A2, // movne r2,r2,lsr#1
        0x1AFFFFF9, // bne div2
    ];

    #[test]
    fn test() {
        let mut proc = Processor::default();
        let mut listener = TestProcessorListener::default();
        proc.memory_mut().set_words_aligned(0x0, &DIVIDE);
        for _ in 0..48 {
            let pc = proc.registers().get(Register::R15);
            println!();
            println!("{}", proc.registers());
            println!(
                "About to execute {}",
                Instr::decode(proc.memory().get_word_aligned(pc))
                    .map_or_else(|| "???".to_owned(), |(cond, i)| Instr::display(&i, cond))
            );
            proc.try_execute(&mut listener).unwrap();
            // Advance the program counter.
            *proc.registers_mut().get_mut(Register::R15) += 4;
        }
        println!("{listener:#?}");
        // Assert that the routine is finished.
        assert_eq!(proc.registers().get(Register::R15), DIVIDE.len() as u32 * 4);
        // Assert that we got the right result.
        assert_eq!(proc.registers().get(Register::R0), 0);
        assert_eq!(proc.registers().get(Register::R1), 1);
        assert_eq!(proc.registers().get(Register::R2), 6);
        assert_eq!(proc.registers().get(Register::R3), 6);
    }
}
