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
        assert_eq!(cpu.memory[0x100usize | v as usize], v + 1);
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
    assert_eq!(cpu.flags, [false; NFLAGS]);
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
    cpu.flags = [false; NFLAGS];
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
    cpu.flags = [true; NFLAGS];
    cpu.registers[A as usize] = 0xAB;
    cpu.sp = 0xFF;
    assert_eq!(3, cpu.execute(PushProcessorStatusWord));
    assert_eq!(0b11010111, cpu.pop_data()); // Flags
    assert_eq!(0xAB, cpu.pop_data()); // A register
}

#[test]
fn rotate_right() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1000_0001);
    cpu.flags = [false; NFLAGS];
    assert_eq!(1, cpu.execute(RotateRight));
    assert_eq!(0b1100_0000, cpu.get_register(A));
    assert_eq!(true, cpu.get_flag(CY));
    cpu.set_register(A, 0b1000_0010);
    assert_eq!(1, cpu.execute(RotateRight));
    assert_eq!(0b0100_0001, cpu.get_register(A));
    assert_eq!(false, cpu.get_flag(CY));
}

#[test]
fn and_immediate() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1010_1010);
    cpu.flags = [false; NFLAGS];
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
    cpu.flags = [false; NFLAGS];
    assert_eq!(2, cpu.execute(AddImmediate(1)));
    assert_eq!(0, cpu.get_register(A));
    assert_eq!(true, cpu.get_flag(CY));
}
