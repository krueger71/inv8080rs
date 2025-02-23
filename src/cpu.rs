//! CPU module

use crate::{
    utils::*, DISPLAY_HEIGHT, FRAMEBUFFER, MEMORY, MEMORY_SIZE, NPORTS, NREGS, RAM, ROM, STACK,
};
use Condition::*;
use Flag::*;
use Instruction::*;
use Register::*;
use RegisterPair::*;

#[cfg(test)]
mod tests;

// Type aliases to match terminology in manual
type Address = usize;
type Data = u8;
type Data16 = u16;

/// Instructions of the Cpu in the order of Chapter 4 of the manual.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Instruction {
    /// Move register - MOV r1, r2
    MoveRegister(Register, Register),
    /// Move from memory - MOV r, M
    MoveFromMemory(Register),
    /// Move to memory - MOV M, r
    MoveToMemory(Register),
    /// Move to register immediate - MVI r, data
    MoveImmediate(Register, Data),
    /// Move to memory immediate - MVI M, data
    MoveToMemoryImmediate(Data),
    /// Load register pair immediate - LXI rp, data16
    LoadRegisterPairImmediate(RegisterPair, Data16),
    /// Load accumulator direct - LDA addr
    LoadAccumulatorDirect(Address),
    /// Store accumulator direct - STA addr
    StoreAccumulatorDirect(Address),
    /// Load H and L direct - LHLD addr
    LoadHLDirect(Address),
    /// Store H and L direct - SHLD addr
    StoreHLDirect(Address),
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
    AddImmediate(Data),
    /// Add register with carry - ADC r
    AddRegisterWithCarry(Register),
    /// Add memory with carry - ADC M
    AddMemoryWithCarry,
    /// Add immediate with carry - ACI data
    AddImmediateWithCarry(Data),
    /// Subtract register - SUB r
    SubtractRegister(Register),
    /// Subtract memory - SUB M
    SubtractMemory,
    /// Subtract immediate - SUI data
    SubtractImmediate(Data),
    /// Subtract register with borrow - SBB r
    SubtractRegisterWithBorrow(Register),
    /// Subtract memory with borrow - SBB M
    SubtractMemoryWithBorrow,
    /// Subtract immediate with borrow - SBI data
    SubtractImmediateWithBorrow(Data),
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
    AndImmediate(Data),
    /// Exclusive OR register - XRA r
    XorRegister(Register),
    /// Exclusive OR memory - XRA M
    XorMemory,
    /// Exclusive OR immediate - XRI data
    XorImmediate(Data),
    /// OR register - ORA r
    OrRegister(Register),
    /// OR memory - ORA M
    OrMemory,
    /// OR immediate - ORI data
    OrImmediate(Data),
    /// Compare register - CMP r
    CompareRegister(Register),
    /// Compare memory - CMP M
    CompareMemory,
    /// Compare immediate - CPI data
    CompareImmediate(Data),
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
    Jump(Address),
    /// Conditional jump - Jcondition addr
    ConditionalJump(Condition, Address),
    /// Call - CALL addr
    Call(Address),
    /// Conditional call - Ccondition addr
    ConditionalCall(Condition, Address),
    /// Return - RET
    Return,
    /// Conditional return - Rcondition addr
    ConditionalReturn(Condition),
    /// Restart - RST n
    Restart(Data),
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
    Input(Data),
    /// Output - OUT port
    Output(Data),
    /// Enable interrupts - EI
    EnableInterrupts,
    /// Disable interrupts - DI
    DisableInterrupts,
    /// Halt - HLT
    Halt,
    /// No operation - NOP
    NoOperation,
    /// Error in decoding opcode (something is wrong)
    Err(Data),
}

/// Register pairs
#[derive(Copy, Clone, Debug, PartialEq)]
enum RegisterPair {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11,
}

/// Register
#[derive(Copy, Clone, Debug, PartialEq)]
enum Register {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    F = 0b110, // Flags
    A = 0b111, // Accumulator
}

/// Condition
#[derive(Copy, Clone, Debug, PartialEq)]
enum Condition {
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
enum Flag {
    Z = 0,
    S = 1,
    P = 2,
    CY = 3,
    AC = 4,
}

/// The CPU-model including memory etc.
pub struct Cpu {
    /// ROM/RAM all writable for now
    memory: [Data; MEMORY_SIZE],
    /// Program counter
    pc: Address,
    /// Registers B,C,D,E,H,L,F (flags) and A (accumulator). Register pairs BC, DE, HL.
    registers: [Data; NREGS],
    /// Stack pointer/register pair SP
    sp: Address,
    /// 8-bit input bus
    bus_in: [Data; NPORTS],
    /// 8-bit output bus
    bus_out: [Data; NPORTS],
    /// 16-bit shift register, communication via I/O (port 4 write)
    shift: Data16,
    /// 8-bit shift offset, communication via I/O (port 2 write)
    offset: Data,
    /// CPU interruptable
    interruptable: bool,
    /// Display should be updated (this is set to true on memory writes to the framebuffer region of memory, then emulator clears it after drawing is finished)
    /// Probably next to useless optimization for a game where everything is moving on the screen :)
    display_update: bool,
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
            bus_in: [0b0000_1110, 0b0000_1000, 0, 0, 0, 0, 0, 0],
            bus_out: [0; NPORTS],
            shift: 0,
            offset: 0,
            interruptable: false,
            display_update: true,
        }
    }

    /// Fetch, decode and execute one instruction
    pub fn step(&mut self) -> u32 {
        let instr = self.fetch_and_decode();
        self.execute(instr)
    }

    /// Return true if pixel at logical display coordinate (x, y) is on.
    pub fn display(&self, x: u32, y: u32) -> bool {
        let framebuffer = &self.memory[0x2400..0x4000];
        let byte =
            framebuffer[(x * DISPLAY_HEIGHT / 8 + (DISPLAY_HEIGHT / 8 - y / 8) - 1) as usize];
        get_bit(byte, 7 - (y % 8) as u8)
    }

    /// Get display update
    pub fn get_display_update(&self) -> bool {
        self.display_update
    }

    /// Set display update
    pub fn set_display_update(&mut self, value: bool) {
        self.display_update = value;
    }

    /// Get CPU input bus (read external input)
    fn get_bus_in(&self, port: usize) -> u8 {
        if port == 3 {
            return ((self.shift << self.offset) >> 8) as u8;
        }
        self.bus_in[port]
    }

    /// Set CPU input bus (write external input)
    pub fn set_bus_in(&mut self, port: usize, data: u8) {
        self.bus_in[port] = data;
    }

    /// Set CPU input bus bit (write external input)
    pub fn set_bus_in_bit(&mut self, port: usize, bit: u8, value: bool) {
        set_bit(&mut self.bus_in[port], bit, value);
    }

    /// Get CPU output bus (read CPU output)
    pub fn get_bus_out(&self, port: usize) -> u8 {
        self.bus_out[port]
    }

    /// Set CPU output bus (write CPU output)
    fn set_bus_out(&mut self, port: usize, data: u8) {
        if port == 2 {
            self.offset = data & 0x7
        } else if port == 4 {
            self.shift = ((data as Data16) << 8) | (self.shift >> 8);
        }

        self.bus_out[port] = data
    }

    /// Fetch and decode one instruction, including immediate data, and increment program counter
    #[allow(clippy::unusual_byte_groupings)]
    fn fetch_and_decode(&mut self) -> Instruction {
        let op = self.get_memory(self.get_pc());

        // For debugging
        // if self.get_pc() == 0x0A8E {
        //     println!("Start debugging");
        // }

        self.incr_pc();

        // Decoding in the order from the manual
        match op {
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
        }
    }

    /// Fetch one byte from memory and advance program counter
    fn fetch_data(&mut self) -> Data {
        let ret = self.get_memory(self.get_pc());
        self.incr_pc();

        ret
    }

    /// Fetch two bytes from memory and advance program counter
    fn fetch_data16(&mut self) -> Data16 {
        let low = self.get_memory(self.get_pc()) as Data16;
        self.incr_pc();
        let high = self.get_memory(self.get_pc()) as Data16;
        self.incr_pc();

        (high << 8) | low
    }

    /// Fetch a two-byte address from memory and advance program counter
    fn fetch_address(&mut self) -> Address {
        self.fetch_data16() as Address
    }

    /// Execute one instruction and return number of cycles taken
    fn execute(&mut self, instr: Instruction) -> u32 {
        match instr {
            NoOperation => 4,
            Jump(addr) => {
                self.set_pc(addr);
                10
            }
            JumpHLIndirect => {
                self.set_pc(self.get_register_pair(HL) as Address);
                5
            }
            LoadRegisterPairImmediate(rp, data) => {
                self.set_register_pair(rp, data);
                10
            }
            MoveImmediate(r, data) => {
                self.set_register(r, data);
                7
            }
            Call(addr) => {
                self.push(self.get_pc());
                self.set_pc(addr);
                17
            }
            Return => {
                let addr = self.pop();
                self.set_pc(addr);
                10
            }
            LoadAccumulatorIndirect(rp) => {
                match rp {
                    BC | DE => {
                        self.set_register(
                            A,
                            self.get_memory(self.get_register_pair(rp) as Address),
                        );
                    }
                    _ => panic!("Invalid instruction {:04X?}", instr),
                }
                7
            }
            StoreAccumulatorIndirect(rp) => {
                match rp {
                    BC | DE => {
                        self.set_memory(
                            self.get_register_pair(rp) as Address,
                            self.get_register(A),
                        );
                    }
                    _ => panic!("Invalid instruction {:04X?}", instr),
                }
                7
            }
            MoveToMemory(r) => {
                self.set_memory(self.get_register_pair(HL) as Address, self.get_register(r));
                7
            }
            IncrementRegisterPair(rp) => {
                let (val, _) = self.get_register_pair(rp).overflowing_add(1);
                self.set_register_pair(rp, val);
                5
            }
            DecrementRegisterPair(rp) => {
                let (val, _) = self.get_register_pair(rp).overflowing_sub(1);
                self.set_register_pair(rp, val);
                5
            }
            DecrementRegister(r) => {
                let before = self.get_register(r);
                let (after, _) = before.overflowing_sub(1);
                self.set_register(r, after);
                self.set_flags_for_arithmetic(before, after, self.get_flag(CY));
                5
            }
            IncrementRegister(r) => {
                let before = self.get_register(r);
                let (after, _) = before.overflowing_add(1);
                self.set_register(r, after);
                self.set_flags_for_arithmetic(before, after, self.get_flag(CY));
                5
            }
            DecrementMemory => {
                let addr = self.get_register_pair(HL) as Address;
                let before = self.get_memory(addr);
                let (after, _) = before.overflowing_sub(1);
                self.set_memory(addr, after);
                self.set_flags_for_arithmetic(before, after, self.get_flag(CY));
                10
            }
            IncrementMemory => {
                let addr = self.get_register_pair(HL) as Address;
                let before = self.get_memory(addr);
                let (after, _) = before.overflowing_add(1);
                self.set_memory(addr, after);
                self.set_flags_for_arithmetic(before, after, self.get_flag(CY));
                10
            }
            ConditionalJump(c, addr) => {
                if self.is_condition(c) {
                    self.set_pc(addr);
                }
                10
            }
            ConditionalCall(c, addr) => {
                if self.is_condition(c) {
                    self.push(self.get_pc());
                    self.set_pc(addr);
                    17
                } else {
                    11
                }
            }
            ConditionalReturn(c) => {
                if self.is_condition(c) {
                    let addr = self.pop();
                    self.set_pc(addr);
                    11
                } else {
                    5
                }
            }
            MoveToMemoryImmediate(data) => {
                self.set_memory(self.get_register_pair(HL) as Address, data);
                10
            }
            MoveRegister(to, from) => {
                self.set_register(to, self.get_register(from));
                5
            }
            CompareImmediate(data) => {
                let before = self.get_register(A);
                let (after, carry) = before.overflowing_sub(data);
                self.set_flags_for_arithmetic(before, after, carry);
                7
            }
            CompareRegister(r) => {
                let before = self.get_register(A);
                let data = self.get_register(r);
                let (after, carry) = before.overflowing_sub(data);
                self.set_flags_for_arithmetic(before, after, carry);
                4
            }
            CompareMemory => {
                let before = self.get_register(A);
                let data = self.get_memory(self.get_register_pair(HL) as Address);
                let (after, carry) = before.overflowing_sub(data);
                self.set_flags_for_arithmetic(before, after, carry);
                7
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
                11
            }
            Pop(rp) => {
                match rp {
                    BC | DE | HL => {
                        let data = self.pop() as Data16;
                        self.set_register_pair(rp, data);
                    }
                    SP => {
                        panic!("Can't pop SP");
                    }
                }
                10
            }
            AddRegisterPairToHL(rp) => {
                let (value, carry) = self
                    .get_register_pair(HL)
                    .overflowing_add(self.get_register_pair(rp));
                self.set_register_pair(HL, value);
                self.set_flag(CY, carry);
                10
            }
            ExchangeHLWithDE => {
                let hl = self.get_register_pair(HL);
                self.set_register_pair(HL, self.get_register_pair(DE));
                self.set_register_pair(DE, hl);
                4
            }
            ExchangeSPWithHL => {
                let h = self.get_register(H);
                let l = self.get_register(L);
                let sl = self.get_memory(self.get_sp());
                let sh = self.get_memory(self.get_sp() + 1);
                self.set_register(L, sl);
                self.set_register(H, sh);
                self.set_memory(self.get_sp(), l);
                self.set_memory(self.get_sp() + 1, h);
                18
            }
            Output(port) => {
                self.set_bus_out(port as usize, self.get_register(A));
                10
            }
            Input(port) => {
                let bus = self.get_bus_in(port as usize);
                self.set_register(A, bus);
                10
            }
            MoveFromMemory(r) => {
                self.set_register(r, self.get_memory(self.get_register_pair(HL) as Address));
                7
            }
            PushProcessorStatusWord => {
                self.push_data(self.get_register(A));
                self.push_data(self.get_flags());
                11
            }
            PopProcessorStatusWord => {
                let flags = self.pop_data();
                self.set_flags(flags);
                let a = self.pop_data();
                self.set_register(A, a);
                10
            }
            RotateRight => {
                let acc = self.get_register(A);
                self.set_flag(CY, get_bit(acc, 0));
                self.set_register(A, acc.rotate_right(1));
                4
            }
            RotateLeft => {
                let acc = self.get_register(A);
                self.set_flag(CY, get_bit(acc, 7));
                self.set_register(A, acc.rotate_left(1));
                4
            }
            RotateRightThroughCarry => {
                let mut acc = self.get_register(A);
                let low = get_bit(acc, 0);
                acc >>= 1;
                set_bit(&mut acc, 7, self.get_flag(CY));
                self.set_flag(CY, low);
                self.set_register(A, acc);
                4
            }
            OrMemory => {
                let before = self.get_register(A);
                let val = self.get_memory(self.get_register_pair(HL) as usize);
                self.set_register(A, before | val);
                self.set_flags_for_arithmetic(before, self.get_register(A), false);
                self.set_flag(AC, false);
                7
            }
            OrRegister(r) => {
                let before = self.get_register(A);
                let val = self.get_register(r);
                self.set_register(A, before | val);
                self.set_flags_for_arithmetic(before, self.get_register(A), false);
                self.set_flag(AC, false);
                4
            }
            OrImmediate(val) => {
                let before = self.get_register(A);
                self.set_register(A, before | val);
                self.set_flags_for_arithmetic(before, self.get_register(A), false);
                self.set_flag(AC, false);
                7
            }
            AndImmediate(data) => {
                let before = self.get_register(A);
                self.set_register(A, before & data);
                self.set_flags_for_arithmetic(before, self.get_register(A), false);
                self.set_flag(AC, false);
                7
            }
            AndMemory => {
                let before = self.get_register(A);
                let data = self.get_memory(self.get_register_pair(HL) as usize);
                self.set_register(A, before & data);
                self.set_flags_for_arithmetic(before, self.get_register(A), false);
                7
            }
            AddImmediate(addend) => {
                self.add(addend);
                7
            }
            AddRegister(r) => {
                self.add(self.get_register(r));
                4
            }
            AddRegisterWithCarry(r) => {
                self.add(self.get_register(r) + if self.get_flag(CY) { 1 } else { 0 });
                4
            }
            AddMemory => {
                self.add(self.get_memory(self.get_register_pair(HL) as Address));
                7
            }
            SubtractRegister(r) => {
                let before = self.get_register(A);
                let data = self.get_register(r);
                let (after, carry) = before.overflowing_sub(data);
                self.set_register(A, after);
                self.set_flags_for_arithmetic(before, self.get_register(A), carry);
                4
            }
            SubtractImmediate(data) => {
                let before = self.get_register(A);
                let (after, carry) = before.overflowing_sub(data);
                self.set_register(A, after);
                self.set_flags_for_arithmetic(before, self.get_register(A), carry);
                7
            }
            SubtractImmediateWithBorrow(data) => {
                let before = self.get_register(A);
                let (after, carry) =
                    before.overflowing_sub(data + if self.get_flag(CY) { 1 } else { 0 });
                self.set_register(A, after);
                self.set_flags_for_arithmetic(before, self.get_register(A), carry);
                7
            }
            LoadAccumulatorDirect(addr) => {
                self.set_register(A, self.get_memory(addr));
                13
            }
            StoreAccumulatorDirect(addr) => {
                self.set_memory(addr, self.get_register(A));
                13
            }
            ComplementAccumulator => {
                self.set_register(A, self.get_register(A) ^ 0xFF);
                4
            }
            XorRegister(r) => {
                let before = self.get_register(A);
                self.set_register(A, before ^ self.get_register(r));
                self.set_flags_for_arithmetic(before, self.get_register(A), false);
                self.set_flag(AC, false);
                4
            }
            AndRegister(r) => {
                let before = self.get_register(A);
                self.set_register(A, before & self.get_register(r));
                self.set_flags_for_arithmetic(before, self.get_register(A), false);
                4
            }
            DisableInterrupts => {
                self.interruptable = false;
                4
            }
            EnableInterrupts => {
                // TODO The CPU should be interruptable following the next instruction
                self.interruptable = true;
                4
            }
            Restart(data) => {
                self.push(self.get_pc());
                self.set_pc((8 * data as i32) as Address);
                11
            }
            SetCarry => {
                self.set_flag(CY, true);
                4
            }
            LoadHLDirect(addr) => {
                self.set_register(L, self.get_memory(addr));
                self.set_register(H, self.get_memory(addr + 1));
                16
            }
            StoreHLDirect(addr) => {
                self.set_memory(addr, self.get_register(L));
                self.set_memory(addr + 1, self.get_register(H));
                16
            }
            DecimalAdjustAccumulator => {
                let acc = self.get_register(A);
                let mut new_acc = acc;

                if acc & 0xF > 9 || self.get_flag(AC) {
                    new_acc = new_acc.wrapping_add(0x6);
                    self.set_flag(AC, true);
                }

                if acc > 0x99 || self.get_flag(CY) {
                    new_acc = new_acc.wrapping_add(0x60);
                    self.set_flag(CY, true);
                }

                self.set_register(A, new_acc);
                4
            }
            _ => panic!("Unimplemented {:04X?} now at {:04X?}", instr, self.pc),
        }
    }

    /// Interrupt
    pub fn interrupt(&mut self, data: Data) -> u32 {
        if self.interruptable {
            self.interruptable = false; // TODO Should this be done?
            self.execute(Restart(data))
        } else {
            0
        }
    }

    // CPU "micro-code" below

    /// Get program counter
    fn get_pc(&self) -> usize {
        self.pc
    }

    /// Set program counter
    fn set_pc(&mut self, pc: usize) {
        debug_assert!(
            ROM.contains(&pc),
            "Program counter {:04X} outside ROM memory!",
            pc
        );
        self.pc = pc;
    }

    /// Increment pc
    fn incr_pc(&mut self) {
        self.set_pc(self.get_pc() + 1);
    }

    /// Get stack pointer
    fn get_sp(&self) -> usize {
        self.sp
    }

    /// Set stack pointer
    fn set_sp(&mut self, sp: usize) {
        debug_assert!(
            STACK.contains(&sp),
            "Stack pointer {:04X} outside STACK memory!",
            sp
        );
        self.sp = sp;
    }

    /// Get memory
    fn get_memory(&self, addr: Address) -> Data {
        debug_assert!(
            MEMORY.contains(&addr),
            "Reading outside memory at {:02X}",
            addr
        );
        self.memory[addr]
    }

    /// Set memory
    fn set_memory(&mut self, addr: Address, data: Data) {
        debug_assert!(RAM.contains(&addr), "Writing outside ram at {:02X}", addr);
        self.memory[addr] = data;

        if FRAMEBUFFER.contains(&addr) {
            self.display_update = true;
        }
    }

    /// Get register
    fn get_register(&self, r: Register) -> Data {
        self.registers[r as usize]
    }

    /// Set register
    fn set_register(&mut self, r: Register, data: Data) {
        self.registers[r as usize] = data;
    }

    /// Get flag
    fn get_flag(&self, flag: Flag) -> bool {
        let flags = self.get_register(F);
        match flag {
            CY => get_bit(flags, 0),
            P => get_bit(flags, 2),
            AC => get_bit(flags, 4),
            Z => get_bit(flags, 6),
            S => get_bit(flags, 7),
        }
    }

    /// Set flag
    fn set_flag(&mut self, flag: Flag, val: bool) {
        let mut flags = self.get_register(F);
        match flag {
            CY => set_bit(&mut flags, 0, val),
            P => set_bit(&mut flags, 2, val),
            AC => set_bit(&mut flags, 4, val),
            Z => set_bit(&mut flags, 6, val),
            S => set_bit(&mut flags, 7, val),
        };
        self.set_register(F, flags);
    }

    /// Get flags
    fn get_flags(&self) -> Data {
        self.get_register(F)
    }

    /// Set flags
    fn set_flags(&mut self, flags: Data) {
        // TODO This might be necessary
        //set_bit(&mut flags, 1, true);   // Always set
        //set_bit(&mut flags, 3, false);   // Always unset
        //set_bit(&mut flags, 5, false);   // Always unset
        self.set_register(F, flags);
    }

    /// Set the flags for arithmetic operations taking into account carry using the before and after values
    fn set_flags_for_arithmetic(&mut self, before: u8, after: u8, carry: bool) {
        self.set_flag(Z, after == 0);
        self.set_flag(S, after & 0x80 == 0x80);
        self.set_flag(P, after.count_ones() % 2 == 0);
        self.set_flag(CY, carry);
        self.set_flag(
            AC,
            (before & (0b0000_1000 >> 3)) == 1 && (after & (0b0001_0000 >> 4)) == 1,
        );
    }

    /// Add and set flags
    fn add(&mut self, addend: Data) {
        let acc = self.get_register(A);

        self.set_flag(AC, (acc & 0xF) + (addend & 0xF) > 0xF);
        let (result, carry) = acc.overflowing_add(addend);
        self.set_register(A, result);
        self.set_flag(CY, carry);
        self.set_flag(Z, result == 0);
        self.set_flag(S, result & 0x80 == 0x80);
        self.set_flag(P, result.count_ones() % 2 == 0);
    }

    /// Set register pair
    fn set_register_pair(&mut self, rp: RegisterPair, data: Data16) {
        match rp {
            BC | DE | HL => {
                let i = (rp as usize) * 2;
                self.registers[i] = ((data & 0xFF00) >> 8) as u8;
                self.registers[i + 1] = (data & 0x00FF) as u8;
            }
            SP => {
                self.set_sp(data as usize);
            }
        }
    }

    /// Get register pair
    fn get_register_pair(&self, rp: RegisterPair) -> Data16 {
        match rp {
            BC | DE | HL => {
                let i = (rp as usize) * 2;
                ((self.registers[i] as u16) << 8) | self.registers[i + 1] as u16
            }
            SP => self.get_sp() as Data16,
        }
    }

    /// Check condition
    fn is_condition(&self, c: Condition) -> bool {
        match c {
            NotZero => !self.get_flag(Z),
            Zero => self.get_flag(Z),
            NoCarry => !self.get_flag(CY),
            Carry => self.get_flag(CY),
            ParityOdd => !self.get_flag(P),
            ParityEven => self.get_flag(P),
            Plus => !self.get_flag(S),
            Minus => self.get_flag(S),
        }
    }

    /// Push
    fn push(&mut self, data: Address) {
        self.push_data(((data & 0xFF00) >> 8) as Data);
        self.push_data((data & 0x00FF) as Data);
    }

    fn push_data(&mut self, data: Data) {
        self.set_sp(self.get_sp() - 1);
        self.set_memory(self.get_sp(), data);
    }

    /// Pop
    fn pop(&mut self) -> Address {
        let ret = self.peek();
        self.set_sp(self.get_sp() + 2);
        ret
    }

    fn pop_data(&mut self) -> Data {
        let ret = self.peek_data();
        self.set_sp(self.get_sp() + 1);
        ret
    }

    /// Peek
    fn peek(&self) -> Address {
        (self.get_memory(self.get_sp()) as Address)
            | ((self.get_memory(self.get_sp() + 1) as Address) << 8)
    }

    fn peek_data(&self) -> Data {
        self.get_memory(self.get_sp())
    }
}
