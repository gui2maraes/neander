pub mod instr {
    pub const NOP: u8 = 0b0000_0000;
    pub const STA: u8 = 0b0001_0000;
    pub const LDA: u8 = 0b0010_0000;
    pub const ADD: u8 = 0b0011_0000;
    pub const OR: u8 = 0b0100_0000;
    pub const AND: u8 = 0b0101_0000;
    pub const NOT: u8 = 0b0110_0000;
    pub const JMP: u8 = 0b1000_0000;
    pub const JN: u8 = 0b1001_0000;
    pub const JZ: u8 = 0b1010_0000;
    pub const HLT: u8 = 0b1111_0000;
    pub fn print_instr_table() {
        let instrs = [NOP, STA, LDA, ADD, OR, AND, NOT, JMP, JN, JZ, HLT];
        let names = [
            "NOP", "STA", "LDA", "ADD", "OR", "AND", "NOT", "JMP", "JN", "JZ", "HLT",
        ];
        println!("INSTR | DEC | HEX");
        for (i, name) in instrs.iter().zip(names) {
            println!("{name:5} | {i:3} | {i:X}");
        }
    }
}
use instr::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecResult {
    Normal,
    Halted,
    MemWrite { addr: u8, value: i8 },
    Exception(NeanderException),
}
impl ExecResult {
    pub fn unwrap(self) {
        if let ExecResult::Exception(e) = self {
            panic!("ExecResult was an exception: {e}")
        }
    }
}
impl<T> From<Result<T, NeanderException>> for ExecResult {
    fn from(value: Result<T, NeanderException>) -> Self {
        if let Err(e) = value {
            Self::Exception(e)
        } else {
            Self::Normal
        }
    }
}

/// The Neander CPU. 8-bit based,
/// with a program counter, accumulator
/// and 256 bytes of RAM. 2-complement
/// integer representation.
#[derive(Debug, Clone)]
pub struct Neander {
    /// The Program Counter
    pc: u8,
    /// The Accumulator
    acc: i8,
    /// The status register.
    /// bit 0 is the zero bit and
    /// bit 1 is the negative bit.
    /// bit 2 is set if the end of program is reached,
    /// and used only by this implementation
    status: u8,
    /// RAM
    mem: Box<[u8; 256]>,
}

/// An Error that occurred during execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeanderException {
    EndOfProgram,
    InvalidInstruction(u8),
    MissingArgument,
}
impl std::fmt::Display for NeanderException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::EndOfProgram => write!(f, "reached end of program"),
            Self::InvalidInstruction(i) => write!(f, "invalid instruction: {i:x}"),
            Self::MissingArgument => write!(f, "missing argument to instruction"),
        }
    }
}

macro_rules! or_bail {
    ($x:expr) => {
        match $x {
            Ok(x) => x,
            e => return e.into(),
        }
    };
}
impl Neander {
    pub fn new() -> Self {
        Self {
            pc: 0,
            acc: 0,
            status: 0,
            mem: vec![0; 256].into_boxed_slice().try_into().unwrap(),
        }
    }
    pub fn pc(&self) -> u8 {
        self.pc
    }
    pub fn acc(&self) -> i8 {
        self.acc
    }
    pub fn status(&self) -> u8 {
        self.status
    }
    pub fn run(&mut self) -> Result<(), NeanderException> {
        loop {
            match self.step() {
                ExecResult::Halted => break Ok(()),
                ExecResult::Exception(e) => return Err(e),
                _ => (),
            }
        }
    }
    /// Executes the next instruction and updates the program counter.
    /// Returns Ok(true) if reached a HLT instruction, Err(exception)
    /// if an error occurred, or Ok(false) otherwise.
    pub fn step(&mut self) -> ExecResult {
        let instr = or_bail!(self.next_instr());
        match instr {
            // NOP
            NOP => {}
            // STA addr
            STA => {
                let arg = or_bail!(self.arg());
                self.set_ram(arg, self.acc as u8);
                return ExecResult::MemWrite {
                    addr: arg,
                    value: self.acc,
                };
            }
            // LDA addr
            LDA => {
                let arg = or_bail!(self.arg());
                self.acc = self.ram(arg) as i8;
                self.set_status(self.acc);
            }
            // ADD addr
            ADD => {
                let arg = or_bail!(self.arg());
                self.acc = self.acc.wrapping_add(self.ram(arg) as i8);
                self.set_status(self.acc);
            }
            // OR addr
            OR => {
                let arg = or_bail!(self.arg());
                self.acc |= self.ram(arg) as i8;
                self.set_status(self.acc);
            }
            // AND addr
            AND => {
                let arg = or_bail!(self.arg());
                self.acc &= self.ram(arg) as i8;
                self.set_status(self.acc);
            }
            // NOT
            NOT => {
                self.acc = !self.acc;
                self.set_status(self.acc);
            }
            // JMP addr
            JMP => {
                self.pc = or_bail!(self.arg());
            }
            // JN addr
            JN => {
                let addr = or_bail!(self.arg());
                if self.status_negative() {
                    self.pc = addr;
                }
            }
            // JZ addr
            JZ => {
                let addr = or_bail!(self.arg());
                if self.status_zero() {
                    self.pc = addr;
                }
            }
            // HLT
            HLT => return ExecResult::Halted,
            i => return ExecResult::Exception(NeanderException::InvalidInstruction(i)),
        }
        ExecResult::Normal
    }
    pub fn memory(&self) -> &[u8] {
        self.mem.as_ref()
    }
    pub fn memory_mut(&mut self) -> &mut [u8] {
        self.mem.as_mut()
    }

    /// Gets the byte at position `idx` in RAM.
    pub fn ram(&self, idx: u8) -> u8 {
        self.mem[idx as usize]
    }

    /// Sets byte at `idx` to `val` in RAM.
    pub fn set_ram(&mut self, idx: u8, val: u8) {
        self.mem[idx as usize] = val;
    }
    /// Copies the content of `slice` into memory, starting at `idx`.
    pub fn set_ram_slice(&mut self, idx: u8, slice: &[u8]) {
        let start = idx as usize;
        let end = slice.len() + start;
        self.mem[start..end].copy_from_slice(slice);
    }

    /// Returns the argument of an instruction.
    fn arg(&mut self) -> Result<u8, NeanderException> {
        match self.next_instr() {
            Ok(arg) => Ok(arg),
            Err(NeanderException::EndOfProgram) => Err(NeanderException::MissingArgument),
            _ => unreachable!(),
        }
    }
    fn set_status(&mut self, val: i8) {
        // clear bits 0 and 1, but leave bit 2 alone.
        self.status &= 4;
        if val == 0 {
            self.status |= 1;
        } else if val < 0 {
            self.status |= 2;
        }
    }
    fn set_end_of_program(&mut self) {
        self.status |= 4;
    }
    pub fn status_zero(&self) -> bool {
        self.status & 1 != 0
    }
    pub fn status_negative(&self) -> bool {
        self.status & 2 != 0
    }
    pub fn status_end_of_prog(&self) -> bool {
        self.status & 4 != 0
    }
    pub fn print_mem_range(&self, start: u8, end: u8) {
        // start at an address divisible by 4
        let s = start - start % 4;
        for (i, line) in self.memory()[(s as usize)..=(end as usize)]
            .chunks_exact(4)
            .enumerate()
        {
            println!(
                "{0:02X} ({0:03}): {1:02X} {2:02X} {3:02X} {4:02X}",
                i * 4,
                line[0],
                line[1],
                line[2],
                line[3]
            );
        }
    }
    pub fn print_mem(&self) {
        for (i, line) in self.memory().chunks_exact(4).enumerate() {
            println!(
                "{0:02X} ({0:03}): {1:02X} {2:02X} {3:02X} {4:02X}",
                i * 4,
                line[0],
                line[1],
                line[2],
                line[3]
            );
        }
    }

    /// Returns the next instruction, or none
    /// if PC is 255. In this case, CPU should halt.
    fn next_instr(&mut self) -> Result<u8, NeanderException> {
        if self.status_end_of_prog() {
            return Err(NeanderException::EndOfProgram);
        }
        let instr = self.mem[self.pc as usize];
        if self.pc == 255 {
            self.set_end_of_program();
        }
        self.pc = self.pc.wrapping_add(1);
        Ok(instr)
    }
}

impl std::fmt::Display for Neander {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "STATE:
AC: {0} | 0x{0:X} | 0b{0:b}
PC: {1} | 0x{1:X} | 0b{1:b}
N: {2}, Z: {3}",
            self.acc(),
            self.pc(),
            self.status_negative() as u8,
            self.status_zero() as u8
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn assert_pc_acc_stt(cpu: &Neander, pc: u8, acc: i8, stt: u8) {
        assert_eq!(cpu.pc, pc);
        assert_eq!(cpu.acc, acc);
        assert_eq!(cpu.status, stt);
    }

    #[test]
    fn test_nop() {
        let mut cpu = Neander::new();
        cpu.set_ram(0, 0);
        assert_eq!(cpu.step(), ExecResult::Normal);
        assert_pc_acc_stt(&cpu, 1, 0, 0);
    }
    #[test]
    fn test_lda() {
        let mut cpu = Neander::new();
        cpu.set_ram_slice(0, &[LDA, 128, LDA, 128, LDA, 128]);

        cpu.set_ram(128, 10);
        assert_eq!(cpu.step(), ExecResult::Normal);
        assert_pc_acc_stt(&cpu, 2, 10, 0);

        cpu.set_ram(128, 0);
        assert_eq!(cpu.step(), ExecResult::Normal);
        assert_pc_acc_stt(&cpu, 4, 0, 1);

        cpu.set_ram(128, 240);
        assert_eq!(cpu.step(), ExecResult::Normal);
        assert_pc_acc_stt(&cpu, 6, -16, 2);
    }
    #[test]
    fn test_sta() {
        let mut cpu = Neander::new();
        cpu.acc = 10;
        cpu.set_ram_slice(0, &[STA, 128]);
        assert_eq!(
            cpu.step(),
            ExecResult::MemWrite {
                addr: 128,
                value: 10
            }
        );
        assert_eq!(cpu.ram(128), 10);
        assert_pc_acc_stt(&cpu, 2, 10, 0);
    }
    #[test]
    fn test_add() {
        let mut cpu = Neander::new();
        cpu.set_ram_slice(
            0,
            &[LDA, 128, ADD, 129, LDA, 130, ADD, 131, LDA, 132, ADD, 133],
        );
        cpu.set_ram_slice(128, &[0, 5, 251, 5, 10, 245]);

        cpu.step().unwrap();
        cpu.step().unwrap();
        assert_pc_acc_stt(&cpu, 4, 5, 0);

        cpu.step().unwrap();
        cpu.step().unwrap();
        assert_pc_acc_stt(&cpu, 8, 0, 1);

        cpu.step().unwrap();
        cpu.step().unwrap();
        assert_pc_acc_stt(&cpu, 12, -1, 2);
    }
}
