use super::*;

/// Return a Cpu in a default state (zero/unset)
fn setup() -> Cpu {
    Cpu::new(vec![])
}

// Test CPU "micro-code"
#[test]
fn get_memory() {
    let mut cpu = setup();
    assert_eq!(0, cpu.get_memory(0xFF));
    cpu.memory[0xFF] = 0xAB;
    assert_eq!(0xAB, cpu.get_memory(0xFF));
}

#[test]
fn set_memory() {
    let mut cpu = setup();
    cpu.set_memory(0xFF, 0xAB);
    assert_eq!(0xAB, cpu.memory[0xFF]);
}

#[test]
fn get_register() {
    let mut cpu = setup();
    for r in [B, C, D, E, H, L, A] {
        assert_eq!(0, cpu.get_register(r));
        cpu.set_register(r, 0xF0 | r as u8);
        assert_eq!(0xF0 | r as u8, cpu.get_register(r));
    }
}

#[test]
fn set_register() {
    let mut cpu = setup();
    for r in [B, C, D, E, H, L, A] {
        cpu.set_register(r, 0xF0 | r as u8);
        assert_eq!(0xF0 | r as u8, cpu.get_register(r));
    }
}

#[test]
fn get_flag() {
    let mut cpu = setup();
    for f in [CY, P, AC, Z, S] {
        assert!(!cpu.get_flag(f));
    }
    cpu.set_register(F, 0b1101_0101);
    for f in [CY, P, AC, Z, S] {
        assert!(cpu.get_flag(f));
    }
}

#[test]
fn set_flag() {
    let mut cpu = setup();
    for f in [CY, P, AC, Z, S] {
        assert!(!cpu.get_flag(f));
        cpu.set_flag(f, true);
        assert!(cpu.get_flag(f));
    }
}

#[test]
fn get_flags() {
    let mut cpu = setup();
    assert_eq!(0, cpu.get_flags());
    cpu.set_register(F, 0xFF);
    assert_eq!(0xFF, cpu.get_flags());
}

#[test]
fn set_flags() {
    let mut cpu = setup();
    assert_eq!(0, cpu.get_flags());
    cpu.set_flags(0xFF);
    assert_eq!(0xFF, cpu.get_flags());
}

#[test]
fn get_bus() {
    let mut cpu = setup();
    cpu.bus = [0, 1, 2, 3, 4, 5, 6, 7];
    for port in 0..NPORTS {
        assert_eq!(port as u8, cpu.get_bus(port));
    }
}

#[test]
fn set_bus() {
    let mut cpu = setup();

    for port in 0..NPORTS {
        assert_eq!(0, cpu.get_bus(port));
        cpu.set_bus(port, 0xAB);
        assert_eq!(0xAB, cpu.get_bus(port));
    }
}

// Test CPU operations

#[test]
fn no_operation() {
    let mut cpu = setup();
    assert_eq!(1, cpu.execute(NoOperation));
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.registers, [0; NREGS]);
    assert_eq!(cpu.get_flags(), 0);
}

#[test]
fn jump() {
    let mut cpu = setup();
    assert_eq!(3, cpu.execute(Jump(0xABCD)));
    assert_eq!(cpu.pc, 0xABCD);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.registers, [0; NREGS]);
    assert_eq!(cpu.get_flags(), 0);
}

#[test]
fn load_register_pair_immediate() {
    let mut cpu = setup();
    assert_eq!(3, cpu.execute(LoadRegisterPairImmediate(BC, 0xABCD)));
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.registers, [0xAB, 0xCD, 0, 0, 0, 0, 0, 0]);
    assert_eq!(cpu.get_flags(), 0);

    cpu = setup();
    assert_eq!(3, cpu.execute(LoadRegisterPairImmediate(DE, 0xABCD)));
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.registers, [0, 0, 0xAB, 0xCD, 0, 0, 0, 0]);
    assert_eq!(cpu.get_flags(), 0);

    cpu = setup();
    assert_eq!(3, cpu.execute(LoadRegisterPairImmediate(HL, 0xABCD)));
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.registers, [0, 0, 0, 0, 0xAB, 0xCD, 0, 0]);
    assert_eq!(cpu.get_flags(), 0);

    cpu = setup();
    assert_eq!(3, cpu.execute(LoadRegisterPairImmediate(SP, 0xABCD)));
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.sp, 0xABCD);
    assert_eq!(cpu.registers, [0; NREGS]);
    assert_eq!(cpu.get_flags(), 0);
}

#[test]
fn move_immediate() {
    let mut cpu = setup();
    let mut v = 42u8;
    for r in [B, C, D, E, H, L, A] {
        assert_eq!(2, cpu.execute(MoveImmediate(r, v)));
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.get_register(r), v);
        assert_eq!(cpu.get_flags(), 0);
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
    assert_eq!(cpu.get_flags(), 0);
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
    cpu.set_register(B, 0x12);
    cpu.set_register(C, 0x34);
    assert_eq!(2, cpu.execute(LoadAccumulatorIndirect(BC)));
    assert_eq!(0x56, cpu.get_register(A));
    cpu.set_register(D, 0x23);
    cpu.set_register(E, 0x45);
    assert_eq!(2, cpu.execute(LoadAccumulatorIndirect(DE)));
    assert_eq!(0x67, cpu.get_register(A));

    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.get_flags(), 0);
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
        cpu.set_register(H, 1);
        cpu.set_register(L, v);
        cpu.set_register(r, v + 1);
        assert_eq!(2, cpu.execute(MoveToMemory(r)));
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.memory[0x100usize | v as usize], v + 1);
        assert_eq!(cpu.get_flags(), 0);
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
        assert_eq!(cpu.get_flag(Z), true);
        assert_eq!(cpu.get_flag(S), false);
        assert_eq!(cpu.get_flag(P), true);
        assert_eq!(cpu.get_flag(CY), false);
        assert_eq!(cpu.get_flag(AC), false);
        assert_eq!(1, cpu.execute(DecrementRegister(r)));
        assert_eq!(-1, cpu.get_register(r) as i8);
        //assert_eq!(cpu.get_flags(), [false, true, true, true, false]);
        assert_eq!(1, cpu.execute(DecrementRegister(r)));
        assert_eq!(-2, cpu.get_register(r) as i8);
        //assert_eq!(cpu.get_flags(), [false, true, false, false, false]);
        //assert_eq!(1, 2);
    }
}

#[test]
fn conditional_jump() {
    let mut cpu = setup();
    assert_eq!(3, cpu.execute(ConditionalJump(NotZero, 0x0001)));
    assert_eq!(cpu.pc, 0x0001);
    assert_eq!(3, cpu.execute(ConditionalJump(Zero, 0x0002)));
    assert_eq!(cpu.pc, 0x0001);
    assert_eq!(3, cpu.execute(ConditionalJump(NoCarry, 0x0002)));
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(3, cpu.execute(ConditionalJump(Carry, 0x0003)));
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(3, cpu.execute(ConditionalJump(ParityOdd, 0x0003)));
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(3, cpu.execute(ConditionalJump(ParityEven, 0x0004)));
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(3, cpu.execute(ConditionalJump(Plus, 0x0004)));
    assert_eq!(cpu.pc, 0x0004);
    assert_eq!(3, cpu.execute(ConditionalJump(Minus, 0x0005)));
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
    assert_eq!(cpu.get_flags(), 0);
    assert_eq!(2, cpu.execute(CompareImmediate(0xFE)));
    assert!(cpu.get_flag(Z));
    assert_eq!(2, cpu.execute(CompareImmediate(0xFF)));
    assert!(cpu.get_flag(CY));
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
    assert!(!cpu.get_flag(CY));
    assert_eq!(0xFFFE, cpu.get_register_pair(HL));
    assert_eq!(3, cpu.execute(AddRegisterPairToHL(DE)));
    assert!(cpu.get_flag(CY));
    assert_eq!(0, cpu.get_register_pair(HL));
    assert_eq!(3, cpu.execute(AddRegisterPairToHL(SP)));
    assert!(!cpu.get_flag(CY));
    assert_eq!(4, cpu.get_register_pair(HL));
    assert_eq!(3, cpu.execute(AddRegisterPairToHL(HL)));
    assert!(!cpu.get_flag(CY));
    assert_eq!(8, cpu.get_register_pair(HL));
}

#[test]
fn exchange_hl_with_de() {
    let mut cpu = setup();
    cpu.set_register_pair(DE, 0xABCD);
    assert_eq!(1, cpu.execute(ExchangeHLWithDE));
    assert_eq!(0xABCD, cpu.get_register_pair(HL));
}

#[test]
fn move_from_memory() {
    let mut cpu = setup();
    cpu.set_register_pair(HL, 0x1234);
    cpu.memory[0x1234] = 0xAB;
    for r in [A, B, C, D, E] {
        assert_eq!(cpu.get_register(r), 0);
        assert_eq!(2, cpu.execute(MoveFromMemory(r)));
        assert_eq!(cpu.get_register(r), 0xAB);
    }
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
    cpu.set_flags(0);
    assert!(cpu.is_condition(NotZero));
    assert!(!cpu.is_condition(Zero));
    assert!(cpu.is_condition(NoCarry));
    assert!(!cpu.is_condition(Carry));
    assert!(cpu.is_condition(ParityOdd));
    assert!(!cpu.is_condition(ParityEven));
    assert!(cpu.is_condition(Plus));
    assert!(!cpu.is_condition(Minus));
}

#[test]
fn push_processor_status_word() {
    let mut cpu = setup();
    cpu.set_flags(0xFF);
    cpu.set_register(A, 0xAB);
    cpu.sp = 0xFF;
    assert_eq!(3, cpu.execute(PushProcessorStatusWord));
    assert_eq!(0xFD, cpu.sp);
    assert_eq!(0b1111_1111, cpu.pop_data()); // Flags
    assert_eq!(0xAB, cpu.pop_data()); // A register
}

#[test]
fn pop_processor_status_word() {
    let mut cpu = setup();
    cpu.set_flags(0xFF);
    cpu.set_register(A, 0xAB);
    cpu.sp = 0xFF;
    cpu.execute(PushProcessorStatusWord);
    cpu.set_flags(0);
    cpu.set_register(A, 0);
    assert_eq!(3, cpu.execute(PopProcessorStatusWord));
    assert_eq!(0xFF, cpu.get_flags());
    assert_eq!(0xAB, cpu.get_register(A));
    assert_eq!(0xFF, cpu.sp);
}

#[test]
fn rotate_right() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1000_0001);
    cpu.set_flags(0);
    assert_eq!(1, cpu.execute(RotateRight));
    assert_eq!(0b1100_0000, cpu.get_register(A));
    assert_eq!(true, cpu.get_flag(CY));
    cpu.set_register(A, 0b1000_0010);
    assert_eq!(1, cpu.execute(RotateRight));
    assert_eq!(0b0100_0001, cpu.get_register(A));
    assert_eq!(false, cpu.get_flag(CY));
}

#[test]
fn rotate_right_through_carry() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1000_0001);
    cpu.set_flags(0);
    assert_eq!(1, cpu.execute(RotateRightThroughCarry));
    assert_eq!(0b0100_0000, cpu.get_register(A));
    assert_eq!(true, cpu.get_flag(CY));
    assert_eq!(1, cpu.execute(RotateRightThroughCarry));
    assert_eq!(0b1010_0000, cpu.get_register(A));
    assert_eq!(false, cpu.get_flag(CY));
}

#[test]
fn and_immediate() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1010_1010);
    cpu.set_flags(0);
    cpu.set_flag(CY, true);
    cpu.set_flag(AC, true);
    assert_eq!(2, cpu.execute(AndImmediate(0b1111_0000)));
    assert_eq!(0b1010_1010 & 0b1111_0000, cpu.get_register(A));
    assert_eq!(false, cpu.get_flag(CY));
    assert_eq!(false, cpu.get_flag(AC));
}

#[test]
fn add_immediate() {
    let mut cpu = setup();
    cpu.set_register(A, 255);
    cpu.set_flags(0);
    assert_eq!(2, cpu.execute(AddImmediate(1)));
    assert_eq!(0, cpu.get_register(A));
    assert_eq!(true, cpu.get_flag(CY));
}

#[test]
fn load_accumulator_direct() {
    let mut cpu = setup();
    let addr = MEMORY_SIZE - 1;
    cpu.set_memory(addr, 0xAB);
    assert_eq!(4, cpu.execute(LoadAccumulatorDirect(addr)));
    assert_eq!(0xAB, cpu.get_register(A));
}

#[test]
fn store_accumulator_direct() {
    let mut cpu = setup();
    let addr = MEMORY_SIZE - 1;
    cpu.set_register(A, 0xAB);
    assert_eq!(4, cpu.execute(StoreAccumulatorDirect(addr)));
    assert_eq!(0xAB, cpu.get_memory(addr));
}

#[test]
fn xor_register() {
    let mut cpu = setup();
    for r in [B, C, D, E, H, L] {
        cpu.set_flag(CY, true);
        cpu.set_flag(AC, true);
        cpu.set_register(A, 0b1010_1010);
        cpu.set_register(r, 0b0100_1111);
        assert_eq!(1, cpu.execute(XorRegister(r)));
        assert_eq!(0b1110_0101, cpu.get_register(A));
        assert!(!cpu.get_flag(CY));
        assert!(!cpu.get_flag(AC));
    }
}

#[test]
fn disable_interrupts() {
    let mut cpu = setup();
    cpu.interruptable = true;
    assert_eq!(1, cpu.execute(DisableInterrupts));
    assert!(!cpu.interruptable);
}

#[test]
fn enable_interrupts() {
    let mut cpu = setup();
    assert_eq!(1, cpu.execute(EnableInterrupts));
    assert!(cpu.interruptable);
}

#[test]
fn and_register() {
    let mut cpu = setup();
    for r in [B, C, D, E, H, L] {
        cpu.set_flag(CY, true);
        cpu.set_register(A, 0b1010_1010);
        cpu.set_register(r, 0b0100_1111);
        assert_eq!(1, cpu.execute(AndRegister(r)));
        assert_eq!(0b0000_1010, cpu.get_register(A));
        assert!(!cpu.get_flag(CY));
    }
}

#[test]
fn input() {
    let mut cpu = setup();
    for port in 0..NPORTS {
        assert_eq!(0, cpu.get_bus(port));
        cpu.set_bus(port, (port + 1) as u8);
        assert_eq!(3, cpu.execute(Input(port as u8)));
        assert_eq!((port + 1) as u8, cpu.get_register(A));
    }
}

#[test]
fn output() {
    let mut cpu = setup();
    for port in 0..NPORTS {
        assert_eq!(0, cpu.get_bus(port));
        cpu.set_register(A, (port + 1) as u8);
        assert_eq!(3, cpu.execute(Output(port as u8)));
        assert_eq!((port + 1) as u8, cpu.get_bus(port));
    }
}

#[test]
fn restart() {
    let mut cpu = setup();
    cpu.sp = 2;
    cpu.pc = 0x1234;
    assert_eq!(3, cpu.execute(Restart(0xff)));
    assert_eq!(cpu.pc, 0x7f8);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.memory[cpu.sp + 1], 0x12);
    assert_eq!(cpu.memory[cpu.sp], 0x34);
    assert_eq!(cpu.registers, [0; NREGS]);
    assert_eq!(cpu.get_flags(), 0);
}
