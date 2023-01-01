use Condition::*;
use Instruction::*;
use Register::*;
use RegisterPair::*;

/// Instructions of the Cpu in the order of Chapter 4 of the manual.
#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    /// Move register - MOV r1, r2
    Move(Register, Register),
    /// Move from memory - MOV r, M
    MoveFromMem(Register),
    /// Move to memory - MOV M, r
    MoveToMem(Register),
    /// Move to register immediate - MVI r, data
    MoveIm(Register, u8),
    /// Move to memory immediate - MVI M, data
    MoveToMemIm(u8),
    /// Load register pair immediate - LXI rp, data16
    LoadRegPairIm(RegisterPair, [u8; 2]),
    /// Load accumulator direct - LDA addr
    LoadAcc(usize),
    /// Store accumulator direct - STA addr
    StoreAcc(usize),
    /// Load H and L direct - LHLD addr
    LoadHL(usize),
    /// Store H and L direct - SHLD addr
    StoreHL(usize),
    /// Load accumulator indirect - LDAX rp
    LoadAccInd(RegisterPair),
    /// Store accumulator indirect - STAX rp
    StoreAccInd(RegisterPair),
    /// Exchange H and L with D and E - XCHG
    ExchangeHLDE,

    /// Add register - ADD r
    Add(Register),
    /// Add memory - ADD M
    AddMem,
    /// Add immediate - ADI data
    AddIm(u8),
    /// Add register with carry - ADC r
    AddCarry(Register),
    /// Add memory with carry - ADC M
    AddMemCarry,
    /// Add immediate with carry - ACI data
    AddImCarry(u8),
    /// Subtract register - SUB r
    Sub(Register),
    /// Subtract memory - SUB M
    SubMem,
    /// Subtract immediate - SUI data
    SubIm(u8),
    /// Subtract register with borrow - SBB r
    SubBorrow(Register),
    /// Subtract memory with borrow - SBB M
    SubMemBorrow,
    /// Subtract immediate with borrow - SBI data
    SubImBorrow(u8),
    /// Increment register - INR r
    Increment(Register),
    /// Increment memory - INR M
    IncrementMem,
    /// Decrement register - DCR r
    Decrement(Register),
    /// Decrement memory - DCR M
    DecrementMem,
    /// Increment register pair - INX rp
    IncrementRegPair(RegisterPair),
    /// Decrement register pair - DCX rp
    DecrementRegPair(RegisterPair),
    /// Add register pair to HL - DAD rp
    AddRegPairHL(RegisterPair),
    /// Decimal adjust accumulator - DAA
    Decimal,

    /// AND register - ANA r
    And(Register),
    /// AND memory - ANA M
    AndMem,
    /// AND immediate - ANI data
    AndIm(u8),
    /// Exclusive OR register - XRA r
    Xor(Register),
    /// Exclusive OR memory - XRA M
    XorMem,
    /// Exclusive OR immediate - XRI data
    XorIm(u8),
    /// OR register - ORA r
    Or(Register),
    /// OR memory - ORA M
    OrMem,
    /// OR immediate - ORI data
    OrIm(u8),
    /// Compare register - CMP r
    Cmp(Register),
    /// Compare memory - CMP M
    CmpMem,
    /// Compare immediate - CPI data
    CmpIm(u8),
    /// Rotate left - RLC
    RotateLeft,
    /// Rotate right - RRC
    RotateRight,
    /// Rotate left through carry - RAL
    RotateLeftCarry,
    /// Rotate right through carry - RAR
    RotateRightCarry,
    /// Complement accumulator - CMA
    Complement,
    /// Complement carry - CMC
    ComplementCarry,
    /// Set carry
    SetCarry,

    /// Jump to address - JMP addr
    Jump(usize),
    /// Conditional jump - Jcondition addr
    JumpCond(Condition, usize),
    /// Call - CALL addr
    Call(usize),
    /// Condition call - Ccondition addr
    CallCond(Condition, usize),
    /// Return - RET
    Return,
    /// Conditional return - Rcondition addr
    ReturnCond(Condition),
    /// Restart - RST n
    Restart(u8),
    /// Jump H and L indirect, move H and L to PC - PCHL
    PCfromHL,
    /// Push - PUSH rp
    Push(RegisterPair),
    /// Push processor status word - PUSH PSW
    PushPsw,
    /// Pop - POP rp
    Pop(RegisterPair),
    /// Pop processor status word - POP PSW
    PopPsw,
    /// Exchange stack top with H and L - XHTL
    StackTopHL,
    /// Move HL to SP - SPHL
    SPfromHL,
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
#[derive(Copy, Clone, Debug)]
#[repr(usize)]
pub enum RegisterPair {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11,
}

/// Register
#[derive(Copy, Clone, Debug)]
#[repr(usize)]
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
#[derive(Copy, Clone, Debug)]
#[repr(usize)]
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

pub const MEMORY_SIZE: usize = 8192;

/// The CPU-model including memory etc.
pub struct Cpu {
    /// RAM
    pub memory: [u8; MEMORY_SIZE],
    /// Program counter
    pub pc: usize,
}

impl Cpu {
    pub fn new(program: Vec<u8>) -> Self {
        let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        memory.copy_from_slice(&program);

        Cpu { memory, pc: 0 }
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

        // Decoding in the order from the manual
        let instr = match op {
            // Data Transfer Group
            0b01_000_000 => Move(B, B),
            0b01_000_001 => Move(B, C),
            0b01_000_010 => Move(B, D),
            0b01_000_011 => Move(B, E),
            0b01_000_100 => Move(B, H),
            0b01_000_101 => Move(B, L),
            0b01_000_111 => Move(B, A),
            0b01_001_000 => Move(C, B),
            0b01_001_001 => Move(C, C),
            0b01_001_010 => Move(C, D),
            0b01_001_011 => Move(C, E),
            0b01_001_100 => Move(C, H),
            0b01_001_101 => Move(C, L),
            0b01_001_111 => Move(C, A),
            0b01_010_000 => Move(D, B),
            0b01_010_001 => Move(D, C),
            0b01_010_010 => Move(D, D),
            0b01_010_011 => Move(D, E),
            0b01_010_100 => Move(D, H),
            0b01_010_101 => Move(D, L),
            0b01_010_111 => Move(D, A),
            0b01_011_000 => Move(E, B),
            0b01_011_001 => Move(E, C),
            0b01_011_010 => Move(E, D),
            0b01_011_011 => Move(E, E),
            0b01_011_100 => Move(E, H),
            0b01_011_101 => Move(E, L),
            0b01_011_111 => Move(E, A),
            0b01_100_000 => Move(H, B),
            0b01_100_001 => Move(H, C),
            0b01_100_010 => Move(H, D),
            0b01_100_011 => Move(H, E),
            0b01_100_100 => Move(H, H),
            0b01_100_101 => Move(H, L),
            0b01_100_111 => Move(H, A),
            0b01_101_000 => Move(L, B),
            0b01_101_001 => Move(L, C),
            0b01_101_010 => Move(L, D),
            0b01_101_011 => Move(L, E),
            0b01_101_100 => Move(L, H),
            0b01_101_101 => Move(L, L),
            0b01_101_111 => Move(L, A),
            0b01_111_000 => Move(A, B),
            0b01_111_001 => Move(A, C),
            0b01_111_010 => Move(A, D),
            0b01_111_011 => Move(A, E),
            0b01_111_100 => Move(A, H),
            0b01_111_101 => Move(A, L),
            0b01_111_111 => Move(A, A),

            0b01_000_110 => MoveFromMem(B),
            0b01_001_110 => MoveFromMem(C),
            0b01_010_110 => MoveFromMem(D),
            0b01_011_110 => MoveFromMem(E),
            0b01_100_110 => MoveFromMem(H),
            0b01_101_110 => MoveFromMem(L),
            0b01_111_110 => MoveFromMem(A),

            0b01110_000 => MoveToMem(B),
            0b01110_001 => MoveToMem(C),
            0b01110_010 => MoveToMem(D),
            0b01110_011 => MoveToMem(E),
            0b01110_100 => MoveToMem(H),
            0b01110_101 => MoveToMem(L),
            0b01110_111 => MoveToMem(A),

            0b00_000_110 => MoveIm(B, self.fetch_byte()),
            0b00_001_110 => MoveIm(C, self.fetch_byte()),
            0b00_010_110 => MoveIm(D, self.fetch_byte()),
            0b00_011_110 => MoveIm(E, self.fetch_byte()),
            0b00_100_110 => MoveIm(H, self.fetch_byte()),
            0b00_101_110 => MoveIm(L, self.fetch_byte()),
            0b00_111_110 => MoveIm(A, self.fetch_byte()),

            0b00110110 => MoveToMemIm(self.fetch_byte()),

            0b00_00_0001 => LoadRegPairIm(BC, self.fetch_bytes()),
            0b00_01_0001 => LoadRegPairIm(DE, self.fetch_bytes()),
            0b00_10_0001 => LoadRegPairIm(HL, self.fetch_bytes()),
            0b00_11_0001 => LoadRegPairIm(SP, self.fetch_bytes()),

            0b00111010 => LoadAcc(self.fetch_address()),

            0b00110010 => StoreAcc(self.fetch_address()),

            0b00101010 => LoadHL(self.fetch_address()),

            0b00100010 => StoreHL(self.fetch_address()),

            0b00_00_1010 => LoadAccInd(BC),
            0b00_01_1010 => LoadAccInd(DE),

            0b00_00_0010 => StoreAccInd(BC),
            0b00_01_0010 => StoreAccInd(DE),

            0b11101011 => ExchangeHLDE,

            // Arithmetic Group
            0b10000_000 => Add(B),
            0b10000_001 => Add(C),
            0b10000_010 => Add(D),
            0b10000_011 => Add(E),
            0b10000_100 => Add(H),
            0b10000_101 => Add(L),
            0b10000_111 => Add(A),

            0b10000110 => AddMem,

            0b11000110 => AddIm(self.fetch_byte()),

            0b10001_000 => AddCarry(B),
            0b10001_001 => AddCarry(C),
            0b10001_010 => AddCarry(D),
            0b10001_011 => AddCarry(E),
            0b10001_100 => AddCarry(H),
            0b10001_101 => AddCarry(L),
            0b10001_111 => AddCarry(A),

            0b10001110 => AddMemCarry,

            0b11001110 => AddImCarry(self.fetch_byte()),

            0b10010_000 => Sub(B),
            0b10010_001 => Sub(C),
            0b10010_010 => Sub(D),
            0b10010_011 => Sub(E),
            0b10010_100 => Sub(H),
            0b10010_101 => Sub(L),
            0b10010_111 => Sub(A),

            0b10010110 => SubMem,

            0b11010110 => SubIm(self.fetch_byte()),

            0b10011_000 => SubBorrow(B),
            0b10011_001 => SubBorrow(C),
            0b10011_010 => SubBorrow(D),
            0b10011_011 => SubBorrow(E),
            0b10011_100 => SubBorrow(H),
            0b10011_101 => SubBorrow(L),
            0b10011_111 => SubBorrow(A),

            0b10011110 => SubMemBorrow,

            0b11011110 => SubImBorrow(self.fetch_byte()),

            0b00_000_100 => Increment(B),
            0b00_001_100 => Increment(C),
            0b00_010_100 => Increment(D),
            0b00_011_100 => Increment(E),
            0b00_100_100 => Increment(H),
            0b00_101_100 => Increment(L),
            0b00_111_100 => Increment(A),

            0b00110100 => IncrementMem,

            0b00_000_101 => Decrement(B),
            0b00_001_101 => Decrement(C),
            0b00_010_101 => Decrement(D),
            0b00_011_101 => Decrement(E),
            0b00_100_101 => Decrement(H),
            0b00_101_101 => Decrement(L),
            0b00_111_101 => Decrement(A),

            0b00110101 => DecrementMem,

            0b00_00_0011 => IncrementRegPair(BC),
            0b00_01_0011 => IncrementRegPair(DE),
            0b00_10_0011 => IncrementRegPair(HL),
            0b00_11_0011 => IncrementRegPair(SP),

            0b00_00_1011 => DecrementRegPair(BC),
            0b00_01_1011 => DecrementRegPair(DE),
            0b00_10_1011 => DecrementRegPair(HL),
            0b00_11_1011 => DecrementRegPair(SP),

            0b00_00_1001 => AddRegPairHL(BC),
            0b00_01_1001 => AddRegPairHL(DE),
            0b00_10_1001 => AddRegPairHL(HL),
            0b00_11_1001 => AddRegPairHL(SP),

            0b00100111 => Decimal,

            // Logical Group
            0b10100_000 => And(B),
            0b10100_001 => And(C),
            0b10100_010 => And(D),
            0b10100_011 => And(E),
            0b10100_100 => And(H),
            0b10100_101 => And(L),
            0b10100_111 => And(A),

            0b10100110 => AndMem,

            0b11100110 => AndIm(self.fetch_byte()),

            0b10101_000 => Xor(B),
            0b10101_001 => Xor(C),
            0b10101_010 => Xor(D),
            0b10101_011 => Xor(E),
            0b10101_100 => Xor(H),
            0b10101_101 => Xor(L),
            0b10101_111 => Xor(A),

            0b10101110 => XorMem,

            0b11101110 => XorIm(self.fetch_byte()),

            0b10110_000 => Or(B),
            0b10110_001 => Or(C),
            0b10110_010 => Or(D),
            0b10110_011 => Or(E),
            0b10110_100 => Or(H),
            0b10110_101 => Or(L),
            0b10110_111 => Or(A),

            0b10110110 => OrMem,

            0b11110110 => OrIm(self.fetch_byte()),

            0b10111_000 => Cmp(B),
            0b10111_001 => Cmp(C),
            0b10111_010 => Cmp(D),
            0b10111_011 => Cmp(E),
            0b10111_100 => Cmp(H),
            0b10111_101 => Cmp(L),
            0b10111_111 => Cmp(A),

            0b10111110 => CmpMem,

            0b11111110 => CmpIm(self.fetch_byte()),

            0b00000111 => RotateLeft,

            0b00001111 => RotateRight,

            0b00010111 => RotateLeftCarry,

            0b00011111 => RotateRightCarry,

            0b00101111 => Complement,

            0b00111111 => ComplementCarry,

            0b00110111 => SetCarry,

            // Branch Group
            0b11000011 => Jump(self.fetch_address()),

            0b11_000_010 => JumpCond(NotZero, self.fetch_address()),
            0b11_001_010 => JumpCond(Zero, self.fetch_address()),
            0b11_010_010 => JumpCond(NoCarry, self.fetch_address()),
            0b11_011_010 => JumpCond(Carry, self.fetch_address()),
            0b11_100_010 => JumpCond(ParityOdd, self.fetch_address()),
            0b11_101_010 => JumpCond(ParityEven, self.fetch_address()),
            0b11_110_010 => JumpCond(Plus, self.fetch_address()),
            0b11_111_010 => JumpCond(Minus, self.fetch_address()),

            0b11001101 => Call(self.fetch_address()),

            0b11_000_100 => CallCond(NotZero, self.fetch_address()),
            0b11_001_100 => CallCond(Zero, self.fetch_address()),
            0b11_010_100 => CallCond(NoCarry, self.fetch_address()),
            0b11_011_100 => CallCond(Carry, self.fetch_address()),
            0b11_100_100 => CallCond(ParityOdd, self.fetch_address()),
            0b11_101_100 => CallCond(ParityEven, self.fetch_address()),
            0b11_110_100 => CallCond(Plus, self.fetch_address()),
            0b11_111_100 => CallCond(Minus, self.fetch_address()),

            0b11001001 => Return,

            0b11_000_000 => ReturnCond(NotZero),
            0b11_001_000 => ReturnCond(Zero),
            0b11_010_000 => ReturnCond(NoCarry),
            0b11_011_000 => ReturnCond(Carry),
            0b11_100_000 => ReturnCond(ParityOdd),
            0b11_101_000 => ReturnCond(ParityEven),
            0b11_110_000 => ReturnCond(Plus),
            0b11_111_000 => ReturnCond(Minus),

            0b11_000_111 => Restart(0b000),
            0b11_001_111 => Restart(0b001),
            0b11_010_111 => Restart(0b010),
            0b11_011_111 => Restart(0b011),
            0b11_100_111 => Restart(0b100),
            0b11_101_111 => Restart(0b101),
            0b11_110_111 => Restart(0b110),
            0b11_111_111 => Restart(0b111),

            0b11101001 => PCfromHL,

            // Stack, I/O and Machine Control Group
            0b11_00_0101 => Push(BC),
            0b11_01_0101 => Push(DE),
            0b11_10_0101 => Push(HL),

            0b11110101 => PushPsw,

            0b11_00_0001 => Pop(BC),
            0b11_01_0001 => Pop(DE),
            0b11_10_0001 => Pop(HL),

            0b11110001 => PopPsw,

            0b11100011 => StackTopHL,

            0b11111001 => SPfromHL,

            0b11011011 => Input(self.fetch_byte()),

            0b11010011 => Output(self.fetch_byte()),

            0b11111011 => EnableInterrupts,

            0b11110011 => DisableInterrupts,

            0b01110110 => Halt,

            0x0 | 0x8 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => NoOperation,

            _ => Err(op),
        };

        #[cfg(debug_assertions)]
        eprintln!("{:04X?}", instr);

        self.pc += 1;

        instr
    }

    /// Fetch a two-byte address from memory and advance program counter
    fn fetch_address(&mut self) -> usize {
        self.pc += 1;
        let low = self.memory[self.pc] as usize;
        self.pc += 1;
        let high = self.memory[self.pc] as usize;
        (high << 8) | low
    }

    /// Fetch one byte from memory and advance program counter
    fn fetch_byte(&mut self) -> u8 {
        self.pc += 1;
        self.memory[self.pc]
    }

    /// Fetch two bytes from memory and advance program counter
    fn fetch_bytes(&mut self) -> [u8; 2] {
        self.pc += 1;
        let low = self.memory[self.pc];
        self.pc += 1;
        let high = self.memory[self.pc];
        [low, high]
    }

    fn execute(&self, _instr: Instruction) {}
}
