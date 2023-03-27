use Condition::*;
use Instruction::*;
use Register::*;
use RegisterPair::*;

/// Instructions of the Cpu in the order of Chapter 4 of the manual.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction {
    /// Move register - MOV r1, r2
    MoveRegister(Register, Register),
    /// Move from memory - MOV r, M
    MoveFromMemory(Register),
    /// Move to memory - MOV M, r
    MoveToMemory(Register),
    /// Move to register immediate - MVI r, data
    MoveImmediate(Register, u8),
    /// Move to memory immediate - MVI M, data
    MoveToMemoryImmediate(u8),
    /// Load register pair immediate - LXI rp, data16
    LoadRegisterPairImmediate(RegisterPair, u16),
    /// Load accumulator direct - LDA addr
    LoadAccumulatorDirect(usize),
    /// Store accumulator direct - STA addr
    StoreAccumulatorDirect(usize),
    /// Load H and L direct - LHLD addr
    LoadHLDirect(usize),
    /// Store H and L direct - SHLD addr
    StoreHLDirect(usize),
    /// Load accumulator indirect - LDAX rp
    LoadAccumulatorIndirect(RegisterPair),
    /// Store accumulator indirect - STAX rp
    StoreAccumulatorIndirect(RegisterPair),
    /// Exchange H and L with D and E - XCHG
    ExchangeHLWithDE,

    /// Add register - ADD r
    AddRegister(Register),
    /// Add memory - ADD M
    AddMemory,
    /// Add immediate - ADI data
    AddImmediate(u8),
    /// Add register with carry - ADC r
    AddRegisterWithCarry(Register),
    /// Add memory with carry - ADC M
    AddMemoryWithCarry,
    /// Add immediate with carry - ACI data
    AddImmediateWithCarry(u8),
    /// Subtract register - SUB r
    SubtractRegister(Register),
    /// Subtract memory - SUB M
    SubtractMemory,
    /// Subtract immediate - SUI data
    SubtractImmediate(u8),
    /// Subtract register with borrow - SBB r
    SubtractRegisterWithBorrow(Register),
    /// Subtract memory with borrow - SBB M
    SubtractMemoryWithBorrow,
    /// Subtract immediate with borrow - SBI data
    SubtractImmediateWithBorrow(u8),
    /// Increment register - INR r
    IncrementRegister(Register),
    /// Increment memory - INR M
    IncrementMemory,
    /// Decrement register - DCR r
    DecrementRegister(Register),
    /// Decrement memory - DCR M
    DecrementMemory,
    /// Increment register pair - INX rp
    IncrementRegisterPair(RegisterPair),
    /// Decrement register pair - DCX rp
    DecrementRegisterPair(RegisterPair),
    /// Add register pair to HL - DAD rp
    AddRegisterPairToHL(RegisterPair),
    /// Decimal adjust accumulator - DAA
    DecimalAdjustAccumulator,

    /// AND register - ANA r
    AndRegister(Register),
    /// AND memory - ANA M
    AndMemory,
    /// AND immediate - ANI data
    AndImmediate(u8),
    /// Exclusive OR register - XRA r
    XorRegister(Register),
    /// Exclusive OR memory - XRA M
    XorMemory,
    /// Exclusive OR immediate - XRI data
    XorImmediate(u8),
    /// OR register - ORA r
    OrRegister(Register),
    /// OR memory - ORA M
    OrMemory,
    /// OR immediate - ORI data
    OrImmediate(u8),
    /// Compare register - CMP r
    CompareRegister(Register),
    /// Compare memory - CMP M
    CompareMemory,
    /// Compare immediate - CPI data
    CompareImmediate(u8),
    /// Rotate left - RLC
    RotateLeft,
    /// Rotate right - RRC
    RotateRight,
    /// Rotate left through carry - RAL
    RotateLeftThroughCarry,
    /// Rotate right through carry - RAR
    RotateRightThroughCarry,
    /// Complement accumulator - CMA
    ComplementAccumulator,
    /// Complement carry - CMC
    ComplementCarry,
    /// Set carry
    SetCarry,

    /// Jump to address - JMP addr
    Jump(usize),
    /// Conditional jump - Jcondition addr
    ConditionalJump(Condition, usize),
    /// Call - CALL addr
    Call(usize),
    /// Conditional call - Ccondition addr
    ConditionalCall(Condition, usize),
    /// Return - RET
    Return,
    /// Conditional return - Rcondition addr
    ConditionalReturn(Condition),
    /// Restart - RST n
    Restart(u8),
    /// Jump H and L indirect, move H and L to PC - PCHL
    JumpHLIndirect,
    /// Push - PUSH rp
    Push(RegisterPair),
    /// Push processor status word - PUSH PSW
    PushProcessorStatusWord,
    /// Pop - POP rp
    Pop(RegisterPair),
    /// Pop processor status word - POP PSW
    PopProcessorStatusWord,
    /// Exchange stack top with H and L - XHTL
    ExchangeSPWithHL,
    /// Move HL to SP - SPHL
    MoveHLToSP,
    /// Input - IN port
    Input(u8),
    /// Output - OUT port
    Output(u8),
    /// Enable interrupts - EI
    EnableInterrupts,
    /// Disable interrupts - DI
    DisableInterrupts,
    /// Halt - HLT
    Halt,
    /// No operation - NOP
    NoOperation,
    /// Error in decoding opcode (something is wrong)
    Err(u8),
}

/// Register pairs
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RegisterPair {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11,
}

/// Register
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Register {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    A = 0b111,
}

/// Condition
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Condition {
    NotZero = 0b000,
    Zero = 0b001,
    NoCarry = 0b010,
    Carry = 0b011,
    ParityOdd = 0b100,
    ParityEven = 0b101,
    Plus = 0b110,
    Minus = 0b111,
}

/// Flags
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Flag {
    Z = 0,
    S = 1,
    P = 2,
    CY = 3,
    AC = 4,
}

pub const MEMORY_SIZE: usize = 0x4000;
pub const NREGS: usize = 8;
pub const NFLAGS: usize = 5;

/// The CPU-model including memory etc.
pub struct Cpu {
    /// ROM/RAM all writable for now
    pub memory: [u8; MEMORY_SIZE],
    /// Program counter
    pub pc: usize,
    /// Registers B,C,D,E,H,L and A (accumulator). Register pairs BC, DE, HL.
    pub registers: [u8; NREGS],
    /// Stack pointer/register pair SP
    pub sp: usize,
    /// Flags
    pub flags: [bool; NFLAGS],
}

impl Cpu {
    pub fn new(program: Vec<u8>) -> Self {
        let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        memory[..program.len()].copy_from_slice(&program);

        Cpu {
            memory,
            pc: 0,
            registers: [0; NREGS],
            sp: 0,
            flags: [false; NFLAGS],
        }
    }

    /// Fetch, decode and execute one instruction
    pub fn step(&mut self) {
        let instr = self.fetch_and_decode();
        self.execute(instr);
    }

    #[allow(clippy::unusual_byte_groupings)]
    fn fetch_and_decode(&mut self) -> Instruction {
        let op = self.memory[self.pc];

        #[cfg(debug_assertions)]
        eprint!("{:04X} {:02X} {:08b} ", self.pc, op, op);

        self.pc += 1;

        // Decoding in the order from the manual
        let instr = match op {
            // Data Transfer Group
            0b01_000_000 => MoveRegister(B, B),
            0b01_000_001 => MoveRegister(B, C),
            0b01_000_010 => MoveRegister(B, D),
            0b01_000_011 => MoveRegister(B, E),
            0b01_000_100 => MoveRegister(B, H),
            0b01_000_101 => MoveRegister(B, L),
            0b01_000_111 => MoveRegister(B, A),
            0b01_001_000 => MoveRegister(C, B),
            0b01_001_001 => MoveRegister(C, C),
            0b01_001_010 => MoveRegister(C, D),
            0b01_001_011 => MoveRegister(C, E),
            0b01_001_100 => MoveRegister(C, H),
            0b01_001_101 => MoveRegister(C, L),
            0b01_001_111 => MoveRegister(C, A),
            0b01_010_000 => MoveRegister(D, B),
            0b01_010_001 => MoveRegister(D, C),
            0b01_010_010 => MoveRegister(D, D),
            0b01_010_011 => MoveRegister(D, E),
            0b01_010_100 => MoveRegister(D, H),
            0b01_010_101 => MoveRegister(D, L),
            0b01_010_111 => MoveRegister(D, A),
            0b01_011_000 => MoveRegister(E, B),
            0b01_011_001 => MoveRegister(E, C),
            0b01_011_010 => MoveRegister(E, D),
            0b01_011_011 => MoveRegister(E, E),
            0b01_011_100 => MoveRegister(E, H),
            0b01_011_101 => MoveRegister(E, L),
            0b01_011_111 => MoveRegister(E, A),
            0b01_100_000 => MoveRegister(H, B),
            0b01_100_001 => MoveRegister(H, C),
            0b01_100_010 => MoveRegister(H, D),
            0b01_100_011 => MoveRegister(H, E),
            0b01_100_100 => MoveRegister(H, H),
            0b01_100_101 => MoveRegister(H, L),
            0b01_100_111 => MoveRegister(H, A),
            0b01_101_000 => MoveRegister(L, B),
            0b01_101_001 => MoveRegister(L, C),
            0b01_101_010 => MoveRegister(L, D),
            0b01_101_011 => MoveRegister(L, E),
            0b01_101_100 => MoveRegister(L, H),
            0b01_101_101 => MoveRegister(L, L),
            0b01_101_111 => MoveRegister(L, A),
            0b01_111_000 => MoveRegister(A, B),
            0b01_111_001 => MoveRegister(A, C),
            0b01_111_010 => MoveRegister(A, D),
            0b01_111_011 => MoveRegister(A, E),
            0b01_111_100 => MoveRegister(A, H),
            0b01_111_101 => MoveRegister(A, L),
            0b01_111_111 => MoveRegister(A, A),

            0b01_000_110 => MoveFromMemory(B),
            0b01_001_110 => MoveFromMemory(C),
            0b01_010_110 => MoveFromMemory(D),
            0b01_011_110 => MoveFromMemory(E),
            0b01_100_110 => MoveFromMemory(H),
            0b01_101_110 => MoveFromMemory(L),
            0b01_111_110 => MoveFromMemory(A),

            0b01110_000 => MoveToMemory(B),
            0b01110_001 => MoveToMemory(C),
            0b01110_010 => MoveToMemory(D),
            0b01110_011 => MoveToMemory(E),
            0b01110_100 => MoveToMemory(H),
            0b01110_101 => MoveToMemory(L),
            0b01110_111 => MoveToMemory(A),

            0b00_000_110 => MoveImmediate(B, self.fetch_data()),
            0b00_001_110 => MoveImmediate(C, self.fetch_data()),
            0b00_010_110 => MoveImmediate(D, self.fetch_data()),
            0b00_011_110 => MoveImmediate(E, self.fetch_data()),
            0b00_100_110 => MoveImmediate(H, self.fetch_data()),
            0b00_101_110 => MoveImmediate(L, self.fetch_data()),
            0b00_111_110 => MoveImmediate(A, self.fetch_data()),

            0b00110110 => MoveToMemoryImmediate(self.fetch_data()),

            0b00_00_0001 => LoadRegisterPairImmediate(BC, self.fetch_data16()),
            0b00_01_0001 => LoadRegisterPairImmediate(DE, self.fetch_data16()),
            0b00_10_0001 => LoadRegisterPairImmediate(HL, self.fetch_data16()),
            0b00_11_0001 => LoadRegisterPairImmediate(SP, self.fetch_data16()),

            0b00111010 => LoadAccumulatorDirect(self.fetch_address()),

            0b00110010 => StoreAccumulatorDirect(self.fetch_address()),

            0b00101010 => LoadHLDirect(self.fetch_address()),

            0b00100010 => StoreHLDirect(self.fetch_address()),

            0b00_00_1010 => LoadAccumulatorIndirect(BC),
            0b00_01_1010 => LoadAccumulatorIndirect(DE),

            0b00_00_0010 => StoreAccumulatorIndirect(BC),
            0b00_01_0010 => StoreAccumulatorIndirect(DE),

            0b11101011 => ExchangeHLWithDE,

            // Arithmetic Group
            0b10000_000 => AddRegister(B),
            0b10000_001 => AddRegister(C),
            0b10000_010 => AddRegister(D),
            0b10000_011 => AddRegister(E),
            0b10000_100 => AddRegister(H),
            0b10000_101 => AddRegister(L),
            0b10000_111 => AddRegister(A),

            0b10000110 => AddMemory,

            0b11000110 => AddImmediate(self.fetch_data()),

            0b10001_000 => AddRegisterWithCarry(B),
            0b10001_001 => AddRegisterWithCarry(C),
            0b10001_010 => AddRegisterWithCarry(D),
            0b10001_011 => AddRegisterWithCarry(E),
            0b10001_100 => AddRegisterWithCarry(H),
            0b10001_101 => AddRegisterWithCarry(L),
            0b10001_111 => AddRegisterWithCarry(A),

            0b10001110 => AddMemoryWithCarry,

            0b11001110 => AddImmediateWithCarry(self.fetch_data()),

            0b10010_000 => SubtractRegister(B),
            0b10010_001 => SubtractRegister(C),
            0b10010_010 => SubtractRegister(D),
            0b10010_011 => SubtractRegister(E),
            0b10010_100 => SubtractRegister(H),
            0b10010_101 => SubtractRegister(L),
            0b10010_111 => SubtractRegister(A),

            0b10010110 => SubtractMemory,

            0b11010110 => SubtractImmediate(self.fetch_data()),

            0b10011_000 => SubtractRegisterWithBorrow(B),
            0b10011_001 => SubtractRegisterWithBorrow(C),
            0b10011_010 => SubtractRegisterWithBorrow(D),
            0b10011_011 => SubtractRegisterWithBorrow(E),
            0b10011_100 => SubtractRegisterWithBorrow(H),
            0b10011_101 => SubtractRegisterWithBorrow(L),
            0b10011_111 => SubtractRegisterWithBorrow(A),

            0b10011110 => SubtractMemoryWithBorrow,

            0b11011110 => SubtractImmediateWithBorrow(self.fetch_data()),

            0b00_000_100 => IncrementRegister(B),
            0b00_001_100 => IncrementRegister(C),
            0b00_010_100 => IncrementRegister(D),
            0b00_011_100 => IncrementRegister(E),
            0b00_100_100 => IncrementRegister(H),
            0b00_101_100 => IncrementRegister(L),
            0b00_111_100 => IncrementRegister(A),

            0b00110100 => IncrementMemory,

            0b00_000_101 => DecrementRegister(B),
            0b00_001_101 => DecrementRegister(C),
            0b00_010_101 => DecrementRegister(D),
            0b00_011_101 => DecrementRegister(E),
            0b00_100_101 => DecrementRegister(H),
            0b00_101_101 => DecrementRegister(L),
            0b00_111_101 => DecrementRegister(A),

            0b00110101 => DecrementMemory,

            0b00_00_0011 => IncrementRegisterPair(BC),
            0b00_01_0011 => IncrementRegisterPair(DE),
            0b00_10_0011 => IncrementRegisterPair(HL),
            0b00_11_0011 => IncrementRegisterPair(SP),

            0b00_00_1011 => DecrementRegisterPair(BC),
            0b00_01_1011 => DecrementRegisterPair(DE),
            0b00_10_1011 => DecrementRegisterPair(HL),
            0b00_11_1011 => DecrementRegisterPair(SP),

            0b00_00_1001 => AddRegisterPairToHL(BC),
            0b00_01_1001 => AddRegisterPairToHL(DE),
            0b00_10_1001 => AddRegisterPairToHL(HL),
            0b00_11_1001 => AddRegisterPairToHL(SP),

            0b00100111 => DecimalAdjustAccumulator,

            // Logical Group
            0b10100_000 => AndRegister(B),
            0b10100_001 => AndRegister(C),
            0b10100_010 => AndRegister(D),
            0b10100_011 => AndRegister(E),
            0b10100_100 => AndRegister(H),
            0b10100_101 => AndRegister(L),
            0b10100_111 => AndRegister(A),

            0b10100110 => AndMemory,

            0b11100110 => AndImmediate(self.fetch_data()),

            0b10101_000 => XorRegister(B),
            0b10101_001 => XorRegister(C),
            0b10101_010 => XorRegister(D),
            0b10101_011 => XorRegister(E),
            0b10101_100 => XorRegister(H),
            0b10101_101 => XorRegister(L),
            0b10101_111 => XorRegister(A),

            0b10101110 => XorMemory,

            0b11101110 => XorImmediate(self.fetch_data()),

            0b10110_000 => OrRegister(B),
            0b10110_001 => OrRegister(C),
            0b10110_010 => OrRegister(D),
            0b10110_011 => OrRegister(E),
            0b10110_100 => OrRegister(H),
            0b10110_101 => OrRegister(L),
            0b10110_111 => OrRegister(A),

            0b10110110 => OrMemory,

            0b11110110 => OrImmediate(self.fetch_data()),

            0b10111_000 => CompareRegister(B),
            0b10111_001 => CompareRegister(C),
            0b10111_010 => CompareRegister(D),
            0b10111_011 => CompareRegister(E),
            0b10111_100 => CompareRegister(H),
            0b10111_101 => CompareRegister(L),
            0b10111_111 => CompareRegister(A),

            0b10111110 => CompareMemory,

            0b11111110 => CompareImmediate(self.fetch_data()),

            0b00000111 => RotateLeft,

            0b00001111 => RotateRight,

            0b00010111 => RotateLeftThroughCarry,

            0b00011111 => RotateRightThroughCarry,

            0b00101111 => ComplementAccumulator,

            0b00111111 => ComplementCarry,

            0b00110111 => SetCarry,

            // Branch Group
            0b11000011 => Jump(self.fetch_address()),

            0b11_000_010 => ConditionalJump(NotZero, self.fetch_address()),
            0b11_001_010 => ConditionalJump(Zero, self.fetch_address()),
            0b11_010_010 => ConditionalJump(NoCarry, self.fetch_address()),
            0b11_011_010 => ConditionalJump(Carry, self.fetch_address()),
            0b11_100_010 => ConditionalJump(ParityOdd, self.fetch_address()),
            0b11_101_010 => ConditionalJump(ParityEven, self.fetch_address()),
            0b11_110_010 => ConditionalJump(Plus, self.fetch_address()),
            0b11_111_010 => ConditionalJump(Minus, self.fetch_address()),

            0b11001101 => Call(self.fetch_address()),

            0b11_000_100 => ConditionalCall(NotZero, self.fetch_address()),
            0b11_001_100 => ConditionalCall(Zero, self.fetch_address()),
            0b11_010_100 => ConditionalCall(NoCarry, self.fetch_address()),
            0b11_011_100 => ConditionalCall(Carry, self.fetch_address()),
            0b11_100_100 => ConditionalCall(ParityOdd, self.fetch_address()),
            0b11_101_100 => ConditionalCall(ParityEven, self.fetch_address()),
            0b11_110_100 => ConditionalCall(Plus, self.fetch_address()),
            0b11_111_100 => ConditionalCall(Minus, self.fetch_address()),

            0b11001001 => Return,

            0b11_000_000 => ConditionalReturn(NotZero),
            0b11_001_000 => ConditionalReturn(Zero),
            0b11_010_000 => ConditionalReturn(NoCarry),
            0b11_011_000 => ConditionalReturn(Carry),
            0b11_100_000 => ConditionalReturn(ParityOdd),
            0b11_101_000 => ConditionalReturn(ParityEven),
            0b11_110_000 => ConditionalReturn(Plus),
            0b11_111_000 => ConditionalReturn(Minus),

            0b11_000_111 => Restart(0b000),
            0b11_001_111 => Restart(0b001),
            0b11_010_111 => Restart(0b010),
            0b11_011_111 => Restart(0b011),
            0b11_100_111 => Restart(0b100),
            0b11_101_111 => Restart(0b101),
            0b11_110_111 => Restart(0b110),
            0b11_111_111 => Restart(0b111),

            0b11101001 => JumpHLIndirect,

            // Stack, I/O and Machine Control Group
            0b11_00_0101 => Push(BC),
            0b11_01_0101 => Push(DE),
            0b11_10_0101 => Push(HL),

            0b11110101 => PushProcessorStatusWord,

            0b11_00_0001 => Pop(BC),
            0b11_01_0001 => Pop(DE),
            0b11_10_0001 => Pop(HL),

            0b11110001 => PopProcessorStatusWord,

            0b11100011 => ExchangeSPWithHL,

            0b11111001 => MoveHLToSP,

            0b11011011 => Input(self.fetch_data()),

            0b11010011 => Output(self.fetch_data()),

            0b11111011 => EnableInterrupts,

            0b11110011 => DisableInterrupts,

            0b01110110 => Halt,

            0b00000000 => NoOperation,
            _ => Err(op), // 12 values unused
        };

        #[cfg(debug_assertions)]
        eprintln!("{:04X?}", instr);

        instr
    }

    /// Fetch one byte from memory and advance program counter
    fn fetch_data(&mut self) -> u8 {
        let ret = self.memory[self.pc];
        self.pc += 1;

        ret
    }

    /// Fetch two bytes from memory and advance program counter
    fn fetch_data16(&mut self) -> u16 {
        let low = self.memory[self.pc] as u16;
        self.pc += 1;
        let high = self.memory[self.pc] as u16;
        self.pc += 1;

        (high << 8) | low
    }

    /// Fetch a two-byte address from memory and advance program counter
    fn fetch_address(&mut self) -> usize {
        self.fetch_data16() as usize
    }

    /// Execute one instruction and return number of cycles taken
    fn execute(&mut self, instr: Instruction) -> u8 {
        let cycles = match instr {
            NoOperation => 1,
            Jump(addr) => {
                self.pc = addr;
                3
            }
            LoadRegisterPairImmediate(rp, data) => {
                self.set_register_pair(rp, data);
                3
            }
            MoveImmediate(r, data) => {
                self.set_register(r, data);
                2
            }
            Call(addr) => {
                self.push(self.pc);
                self.pc = addr;
                5
            }
            Return => {
                self.pc = self.pop();
                3
            }
            LoadAccumulatorIndirect(rp) => {
                match rp {
                    BC | DE => {
                        self.set_register(A, self.memory[self.get_register_pair(rp) as usize]);
                    }
                    _ => panic!("Invalid instruction {:04X?}", instr),
                }
                2
            }
            MoveToMemory(r) => {
                self.memory[self.get_register_pair(HL) as usize] = self.get_register(r);
                2
            }
            IncrementRegisterPair(rp) => {
                self.set_register_pair(rp, 1 + self.get_register_pair(rp));
                1
            }
            DecrementRegister(r) => {
                let before = self.get_register(r);
                let (after, carry) = before.overflowing_sub(1);
                self.set_register(r, after);
                self.set_flags_for_aritmethic(before, after, carry);
                1
            }
            ConditionalJump(c, addr) => {
                if self.is_condition(c) {
                    self.pc = addr;
                }
                3
            }
            MoveToMemoryImmediate(data) => {
                self.memory[self.get_register_pair(HL) as usize] = data;
                3
            }
            MoveRegister(to, from) => {
                self.set_register(to, self.get_register(from));
                1
            }
            CompareImmediate(data) => {
                self.set_flags_for_comparison(data);
                2
            }
            Push(rp) => {
                match rp {
                    BC | DE | HL => {
                        self.push(self.get_register_pair(rp) as usize);
                    }
                    SP => {
                        panic!("Can't push SP");
                    }
                }
                3
            }
            Pop(rp) => {
                match rp {
                    BC | DE | HL => {
                        let data = self.pop() as u16;
                        self.set_register_pair(rp, data);
                    }
                    SP => {
                        panic!("Can't pop SP");
                    }
                }
                3
            }
            AddRegisterPairToHL(rp) => {
                let (value, carry) = self
                    .get_register_pair(HL)
                    .overflowing_add(self.get_register_pair(rp));
                self.set_register_pair(HL, value);
                self.set_flag(Flag::CY, carry);
                3
            }
            ExchangeHLWithDE => {
                self.set_register_pair(HL, self.get_register_pair(DE));
                1
            }
            _ => panic!("Unimplemented {:04X?}", instr),
        };

        #[cfg(debug_assertions)]
        eprintln!(
            "     pc: {:04X}, sp: {:04X}, reg: {:02X?}, flg: {:?}",
            self.pc, self.sp, self.registers, self.flags
        );

        cycles
    }

    // CPU "micro-code" below

    /// Set flag
    fn set_flag(&mut self, flag: Flag, value: bool) {
        self.flags[flag as usize] = value;
    }

    /// Get flag
    fn get_flag(&self, flag: Flag) -> bool {
        self.flags[flag as usize]
    }

    /// Set the flags for aritmethic operations taking into account carry using the before and after values
    fn set_flags_for_aritmethic(&mut self, before: u8, after: u8, carry: bool) {
        self.flags[Flag::Z as usize] = after == 0;
        self.flags[Flag::S as usize] = ((after & 0b1000_0000) >> 7) == 1;
        self.flags[Flag::P as usize] = (((after & 0b1000_0000) >> 7)
            + ((after & 0b0100_0000) >> 6)
            + ((after & 0b0010_0000) >> 5)
            + ((after & 0b0001_0000) >> 4)
            + ((after & 0b0000_1000) >> 3)
            + ((after & 0b0000_0100) >> 2)
            + ((after & 0b0000_0010) >> 1)
            + (after & 0b0000_0001))
            % 2
            == 0;
        self.flags[Flag::CY as usize] = carry;
        self.flags[Flag::AC as usize] =
            (before & 0b0000_1000 >> 3) == 1 && (after & 0b0001_0000 >> 4) == 1;
    }

    /// Set flags for comparisons
    fn set_flags_for_comparison(&mut self, value: u8) {
        let acc = self.get_register(A);
        self.flags[Flag::Z as usize] = acc == value;
        self.flags[Flag::CY as usize] = acc < value;
    }

    /// Set register pair
    fn set_register_pair(&mut self, rp: RegisterPair, data: u16) {
        match rp {
            BC | DE | HL => {
                let i = (rp as usize) * 2;
                self.registers[i] = ((data & 0xFF00) >> 8) as u8;
                self.registers[i + 1] = (data & 0x00FF) as u8;
            }
            SP => {
                self.sp = data as usize;
            }
        }
    }

    /// Get register pair
    fn get_register_pair(&self, rp: RegisterPair) -> u16 {
        match rp {
            BC | DE | HL => {
                let i = (rp as usize) * 2;
                (self.registers[i] as u16) << 8 | self.registers[i + 1] as u16
            }
            SP => self.sp as u16,
        }
    }

    /// Set register
    fn set_register(&mut self, r: Register, data: u8) {
        self.registers[r as usize] = data;
    }

    /// Get register
    fn get_register(&self, r: Register) -> u8 {
        self.registers[r as usize]
    }

    /// Check condition
    fn is_condition(&self, c: Condition) -> bool {
        match c {
            NotZero => !self.get_flag(Flag::Z),
            Zero => self.get_flag(Flag::Z),
            NoCarry => !self.get_flag(Flag::CY),
            Carry => self.get_flag(Flag::CY),
            ParityOdd => !self.get_flag(Flag::P),
            ParityEven => self.get_flag(Flag::P),
            Plus => !self.get_flag(Flag::S),
            Minus => self.get_flag(Flag::S),
        }
    }

    /// Push
    fn push(&mut self, data: usize) {
        self.memory[self.sp - 1] = ((data & 0xFF00) >> 8) as u8;
        self.memory[self.sp - 2] = (data & 0x00FF) as u8;
        self.sp -= 2;
    }

    /// Pop
    fn pop(&mut self) -> usize {
        let ret = self.peek();
        self.sp += 2;
        ret
    }

    /// Peek
    fn peek(&self) -> usize {
        (self.memory[self.sp] as usize) | ((self.memory[self.sp + 1] as usize) << 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Return a Cpu in a default state (zero/unset)
    fn setup() -> Cpu {
        Cpu::new(vec![])
    }

    #[test]
    fn no_operation() {
        let mut cpu = setup();
        assert_eq!(1, cpu.execute(NoOperation));
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.registers, [0; NREGS]);
        assert_eq!(cpu.flags, [false; NFLAGS]);
    }

    #[test]
    fn jump() {
        let mut cpu = setup();
        assert_eq!(3, cpu.execute(Jump(0xABCD)));
        assert_eq!(cpu.pc, 0xABCD);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.registers, [0; NREGS]);
        assert_eq!(cpu.flags, [false; NFLAGS]);
    }

    #[test]
    fn load_register_pair_immediate() {
        let mut cpu = setup();
        assert_eq!(3, cpu.execute(LoadRegisterPairImmediate(BC, 0xABCD)));
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.registers, [0xAB, 0xCD, 0, 0, 0, 0, 0, 0]);
        assert_eq!(cpu.flags, [false; NFLAGS]);

        cpu = setup();
        assert_eq!(3, cpu.execute(LoadRegisterPairImmediate(DE, 0xABCD)));
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.registers, [0, 0, 0xAB, 0xCD, 0, 0, 0, 0]);
        assert_eq!(cpu.flags, [false; NFLAGS]);

        cpu = setup();
        assert_eq!(3, cpu.execute(LoadRegisterPairImmediate(HL, 0xABCD)));
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.registers, [0, 0, 0, 0, 0xAB, 0xCD, 0, 0]);
        assert_eq!(cpu.flags, [false; NFLAGS]);

        cpu = setup();
        assert_eq!(3, cpu.execute(LoadRegisterPairImmediate(SP, 0xABCD)));
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0xABCD);
        assert_eq!(cpu.registers, [0; NREGS]);
        assert_eq!(cpu.flags, [false; NFLAGS]);
    }

    #[test]
    fn move_immediate() {
        let mut cpu = setup();
        let mut v = 42u8;
        for r in [B, C, D, E, H, L, A] {
            assert_eq!(2, cpu.execute(MoveImmediate(r, v)));
            assert_eq!(cpu.pc, 0);
            assert_eq!(cpu.sp, 0);
            assert_eq!(cpu.registers[r as usize], v);
            assert_eq!(cpu.flags, [false; NFLAGS]);
            v += 1;
        }
    }

    #[test]
    fn call() {
        let mut cpu = setup();
        cpu.sp = 2;
        cpu.pc = 0x1234;
        assert_eq!(5, cpu.execute(Call(0x2345)));
        assert_eq!(cpu.pc, 0x2345);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.memory[cpu.sp + 1], 0x12);
        assert_eq!(cpu.memory[cpu.sp], 0x34);
        assert_eq!(cpu.registers, [0; NREGS]);
        assert_eq!(cpu.flags, [false; NFLAGS]);
    }

    #[test]
    fn ret() {
        let mut cpu = setup();
        cpu.memory[cpu.sp] = 0xAB;
        cpu.memory[cpu.sp + 1] = 0xCD;
        assert_eq!(3, cpu.execute(Return));
        assert_eq!(cpu.pc, 0xCDAB);
        assert_eq!(cpu.sp, 2);
    }

    #[test]
    fn load_accumulator_indirect() {
        let mut cpu = setup();
        cpu.memory[0x1234] = 0x56;
        cpu.memory[0x2345] = 0x67;
        cpu.registers[B as usize] = 0x12;
        cpu.registers[C as usize] = 0x34;
        assert_eq!(2, cpu.execute(LoadAccumulatorIndirect(BC)));
        assert_eq!(0x56, cpu.registers[A as usize]);
        cpu.registers[D as usize] = 0x23;
        cpu.registers[E as usize] = 0x45;
        assert_eq!(2, cpu.execute(LoadAccumulatorIndirect(DE)));
        assert_eq!(0x67, cpu.registers[A as usize]);

        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.flags, [false; NFLAGS]);
    }

    #[test]
    #[should_panic]
    fn load_accumulator_indirect_hl() {
        let mut cpu = setup();
        cpu.execute(LoadAccumulatorIndirect(HL));
    }

    #[test]
    #[should_panic]
    fn load_accumulator_indirect_sp() {
        let mut cpu = setup();
        cpu.execute(LoadAccumulatorIndirect(SP));
    }

    #[test]
    fn move_to_memory() {
        let mut cpu = setup();
        let mut v = 1u8;
        for r in [B, C, D, E, A] {
            cpu.registers[H as usize] = 1;
            cpu.registers[L as usize] = v;
            cpu.registers[r as usize] = v + 1;
            assert_eq!(2, cpu.execute(MoveToMemory(r)));
            assert_eq!(cpu.pc, 0);
            assert_eq!(cpu.sp, 0);
            assert_eq!(cpu.memory[(0x100usize | v as usize)], v + 1);
            assert_eq!(cpu.flags, [false; NFLAGS]);
            v += 1;
        }
    }

    #[test]
    fn increment_register_pair() {
        let mut cpu = setup();
        for rp in [BC, DE, HL, SP] {
            cpu.set_register_pair(rp, 0xFF);
            assert_eq!(1, cpu.execute(IncrementRegisterPair(rp)));
            assert_eq!(0x100, cpu.get_register_pair(rp));
        }
    }

    #[test]
    fn decrement_register() {
        let mut cpu = setup();
        for r in [B, C, D, E, H, L, A] {
            cpu.set_register(r, 1);
            assert_eq!(1, cpu.execute(DecrementRegister(r)));
            assert_eq!(0, cpu.get_register(r));
            assert_eq!(cpu.flags, [true, false, true, false, false]);
            assert_eq!(1, cpu.execute(DecrementRegister(r)));
            assert_eq!(-1, cpu.get_register(r) as i8);
            assert_eq!(cpu.flags, [false, true, true, true, false]);
            assert_eq!(1, cpu.execute(DecrementRegister(r)));
            assert_eq!(-2, cpu.get_register(r) as i8);
            assert_eq!(cpu.flags, [false, true, false, false, false]);
        }
    }

    #[test]
    fn conditional_jump() {
        let mut cpu = setup();
        assert_eq!(3, cpu.execute(ConditionalJump(Condition::NotZero, 0x0001)));
        assert_eq!(cpu.pc, 0x0001);
        assert_eq!(3, cpu.execute(ConditionalJump(Condition::Zero, 0x0002)));
        assert_eq!(cpu.pc, 0x0001);
        assert_eq!(3, cpu.execute(ConditionalJump(Condition::NoCarry, 0x0002)));
        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(3, cpu.execute(ConditionalJump(Condition::Carry, 0x0003)));
        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(
            3,
            cpu.execute(ConditionalJump(Condition::ParityOdd, 0x0003))
        );
        assert_eq!(cpu.pc, 0x0003);
        assert_eq!(
            3,
            cpu.execute(ConditionalJump(Condition::ParityEven, 0x0004))
        );
        assert_eq!(cpu.pc, 0x0003);
        assert_eq!(3, cpu.execute(ConditionalJump(Condition::Plus, 0x0004)));
        assert_eq!(cpu.pc, 0x0004);
        assert_eq!(3, cpu.execute(ConditionalJump(Condition::Minus, 0x0005)));
        assert_eq!(cpu.pc, 0x0004);
    }

    #[test]
    fn move_to_memory_immediate() {
        let mut cpu = setup();
        assert_eq!(3, cpu.execute(MoveToMemoryImmediate(0xFE)));
        assert_eq!(cpu.memory[0], 0xFE);
    }

    #[test]
    fn move_register() {
        let mut cpu = setup();
        let mut v = 1;
        for f in [B, C, D, E, H, L, A] {
            for t in [B, C, D, E, H, L, A] {
                cpu.set_register(f, v);
                if f != t {
                    assert_ne!(cpu.get_register(t), v);
                }
                assert_eq!(1, cpu.execute(MoveRegister(t, f)));
                assert_eq!(cpu.get_register(t), v);
            }
            v += 1;
        }
    }

    #[test]
    fn compare_immediate() {
        let mut cpu = setup();

        cpu.set_register(A, 0xFE);
        assert_eq!(2, cpu.execute(CompareImmediate(0xFB)));
        assert_eq!(cpu.flags, [false; NFLAGS]);
        assert_eq!(2, cpu.execute(CompareImmediate(0xFE)));
        assert!(cpu.get_flag(Flag::Z));
        assert_eq!(2, cpu.execute(CompareImmediate(0xFF)));
        assert!(cpu.get_flag(Flag::CY));
    }

    #[test]
    fn push() {
        let mut cpu = setup();
        cpu.sp = 0xF;
        let mut v = 0xA1;
        for rp in [BC, DE, HL] {
            cpu.set_register_pair(rp, v);
            let sp = cpu.sp;
            assert_eq!(3, cpu.execute(Push(rp)));
            assert_eq!(cpu.peek() as u16, v);
            v += 1;
            assert_eq!(cpu.sp, sp - 2);
        }
    }

    #[test]
    #[should_panic]
    fn push_sp() {
        let mut cpu = setup();
        cpu.sp = 0xF;
        assert_eq!(3, cpu.execute(Push(SP)));
    }

    #[test]
    fn pop() {
        let mut cpu = setup();
        cpu.sp = 0xF;
        for rp in [BC, DE, HL] {
            cpu.set_register_pair(rp, 42);
            let sp = cpu.sp;
            assert_eq!(3, cpu.execute(Pop(rp)));
            assert_eq!(cpu.get_register_pair(rp) as u16, 0);
            assert_eq!(cpu.sp, sp + 2);
        }
    }

    #[test]
    #[should_panic]
    fn pop_sp() {
        let mut cpu = setup();
        cpu.sp = 0xF;
        assert_eq!(3, cpu.execute(Pop(SP)));
    }

    #[test]
    fn add_register_pair_to_hl() {
        let mut cpu = setup();
        cpu.set_register_pair(BC, 1);
        cpu.set_register_pair(DE, 2);
        cpu.set_register_pair(SP, 4);
        cpu.set_register_pair(HL, 0xFFFD);
        assert_eq!(3, cpu.execute(AddRegisterPairToHL(BC)));
        assert!(!cpu.get_flag(Flag::CY));
        assert_eq!(0xFFFE, cpu.get_register_pair(HL));
        assert_eq!(3, cpu.execute(AddRegisterPairToHL(DE)));
        assert!(cpu.get_flag(Flag::CY));
        assert_eq!(0, cpu.get_register_pair(HL));
        assert_eq!(3, cpu.execute(AddRegisterPairToHL(SP)));
        assert!(!cpu.get_flag(Flag::CY));
        assert_eq!(4, cpu.get_register_pair(HL));
        assert_eq!(3, cpu.execute(AddRegisterPairToHL(HL)));
        assert!(!cpu.get_flag(Flag::CY));
        assert_eq!(8, cpu.get_register_pair(HL));
    }

    #[test]
    fn exchange_hl_with_de() {
        let mut cpu = setup();
        cpu.set_register_pair(DE, 0xABCD);
        assert_eq!(1, cpu.execute(ExchangeHLWithDE));
        assert_eq!(0xABCD, cpu.get_register_pair(HL));
    }

    // Test helper functions/"micro-code" below

    #[test]
    fn get_and_set_reg_pair() {
        let mut cpu = setup();

        for rp in [BC, DE, HL, SP] {
            cpu.set_register_pair(rp, 0xCAFE);
            assert_eq!(0xCAFE, cpu.get_register_pair(rp));
        }
    }

    #[test]
    fn get_and_set_reg() {
        let mut cpu = setup();

        for r in [B, C, D, E, H, L, A] {
            cpu.set_register(r, 0xFE);
            assert_eq!(0xFE, cpu.get_register(r));
        }
    }

    #[test]
    fn cond() {
        let mut cpu = setup();
        cpu.flags = [false; NFLAGS];
        assert!(cpu.is_condition(Condition::NotZero));
        assert!(!cpu.is_condition(Condition::Zero));
        assert!(cpu.is_condition(Condition::NoCarry));
        assert!(!cpu.is_condition(Condition::Carry));
        assert!(cpu.is_condition(Condition::ParityOdd));
        assert!(!cpu.is_condition(Condition::ParityEven));
        assert!(cpu.is_condition(Condition::Plus));
        assert!(!cpu.is_condition(Condition::Minus));
    }
}
