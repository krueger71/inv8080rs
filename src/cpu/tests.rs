use crate::{RAM, STACK};

use super::*;

/// Return a Cpu in a default state (zero/unset)
fn setup() -> Cpu {
    Cpu::new(vec![])
}

// Test CPU "micro-code"

#[test]
fn get_set_and_incr_pc() {
    let mut cpu = setup();
    assert_eq!(0, cpu.get_pc());
    cpu.set_pc(*ROM.end() - 1);
    assert_eq!(*ROM.end() - 1, cpu.get_pc());
    cpu.incr_pc();
    assert_eq!(*ROM.end(), cpu.get_pc());
}

#[test]
#[should_panic]
fn set_pc_panic_start() {
    let mut cpu = setup();
    cpu.set_pc(*ROM.start() - 1);
}

#[test]
#[should_panic]
fn set_pc_panic_end() {
    let mut cpu = setup();
    cpu.set_pc(*ROM.end() + 1);
}

#[test]
fn get_and_set_sp() {
    let mut cpu = setup();
    assert_eq!(0, cpu.get_sp());
    cpu.set_sp(*STACK.end());
    assert_eq!(*STACK.end(), cpu.get_sp());
    cpu.set_sp(*STACK.start());
    assert_eq!(*STACK.start(), cpu.get_sp());
}

#[test]
#[should_panic]
fn set_sp_panic_start() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.start() - 1);
}

#[test]
#[should_panic]
fn set_sp_panic_end() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.end() + 1);
}

#[test]
fn get_memory() {
    let mut cpu = setup();
    assert_eq!(0, cpu.get_memory(*RAM.start()));
    cpu.set_memory(*RAM.start(), 0xAB);
    assert_eq!(0xAB, cpu.get_memory(*RAM.start()));
}

#[test]
#[should_panic]
fn get_memory_end() {
    let cpu = setup();
    cpu.get_memory(*MEMORY.end() + 1);
}

#[test]
fn set_memory() {
    let mut cpu = setup();
    cpu.set_memory(*RAM.start(), 0xAB);
    assert_eq!(0xAB, cpu.get_memory(*RAM.start()));
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
fn set_flags_for_arithmetic() {
    let mut cpu = setup();

    for cy in [false, true] {
        cpu.set_flags(0);
        cpu.set_flags_for_arithmetic(0, 0, cy);
        assert!(cpu.get_flag(Z));
        assert!(cpu.get_flag(P));
        assert_eq!(cy, cpu.get_flag(CY));
        assert!(!cpu.get_flag(AC));
        assert!(!cpu.get_flag(S));
    }
}

#[test]
fn get_bus() {
    let mut _cpu = setup();
}

#[test]
fn set_bus() {
    let mut _cpu = setup();
}

#[test]
fn get_and_set_reg_pair() {
    let mut cpu = setup();

    for rp in [BC, DE, HL, SP] {
        cpu.set_register_pair(rp, *STACK.end() as Data16);
        assert_eq!(*STACK.end() as Data16, cpu.get_register_pair(rp));
    }

    cpu.set_sp(*STACK.start());
    assert_eq!(*STACK.start() as Data16, cpu.get_register_pair(SP));
    cpu.set_register_pair(SP, *STACK.end() as Data16);
    assert_eq!(*STACK.end(), cpu.get_sp());
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

// Test CPU operations

#[test]
fn no_operation() {
    let mut cpu = setup();
    assert_eq!(4, cpu.execute(NoOperation));
    assert_eq!(cpu.get_pc(), 0);
    assert_eq!(cpu.get_sp(), 0);
    assert_eq!(cpu.registers, [0; NREGS]);
    assert_eq!(cpu.get_flags(), 0);
}

#[test]
fn jump() {
    let mut cpu = setup();
    assert_eq!(10, cpu.execute(Jump(*ROM.end())));
    assert_eq!(cpu.get_pc(), *ROM.end());
    assert_eq!(cpu.get_sp(), 0);
    assert_eq!(cpu.registers, [0; NREGS]);
    assert_eq!(cpu.get_flags(), 0);
}

#[test]
fn jump_hl_indirect() {
    let mut cpu = setup();
    cpu.set_register_pair(HL, *ROM.end() as Data16);
    assert_eq!(5, cpu.execute(JumpHLIndirect));
    assert_eq!(*ROM.end(), cpu.get_pc());
}

#[test]
fn load_register_pair_immediate() {
    let mut cpu = setup();
    assert_eq!(10, cpu.execute(LoadRegisterPairImmediate(BC, 0xABCD)));
    assert_eq!(cpu.get_pc(), 0);
    assert_eq!(cpu.get_sp(), 0);
    assert_eq!(cpu.registers, [0xAB, 0xCD, 0, 0, 0, 0, 0, 0]);
    assert_eq!(cpu.get_flags(), 0);

    cpu = setup();
    assert_eq!(10, cpu.execute(LoadRegisterPairImmediate(DE, 0xABCD)));
    assert_eq!(cpu.get_pc(), 0);
    assert_eq!(cpu.get_sp(), 0);
    assert_eq!(cpu.registers, [0, 0, 0xAB, 0xCD, 0, 0, 0, 0]);
    assert_eq!(cpu.get_flags(), 0);

    cpu = setup();
    assert_eq!(10, cpu.execute(LoadRegisterPairImmediate(HL, 0xABCD)));
    assert_eq!(cpu.get_pc(), 0);
    assert_eq!(cpu.get_sp(), 0);
    assert_eq!(cpu.registers, [0, 0, 0, 0, 0xAB, 0xCD, 0, 0]);
    assert_eq!(cpu.get_flags(), 0);

    cpu = setup();
    assert_eq!(
        10,
        cpu.execute(LoadRegisterPairImmediate(SP, *STACK.end() as Data16))
    );
    assert_eq!(cpu.get_pc(), 0);
    assert_eq!(cpu.get_sp(), *STACK.end());
    assert_eq!(cpu.registers, [0; NREGS]);
    assert_eq!(cpu.get_flags(), 0);
}

#[test]
fn move_immediate() {
    let mut cpu = setup();
    let mut v = 42u8;
    for r in [B, C, D, E, H, L, A] {
        assert_eq!(7, cpu.execute(MoveImmediate(r, v)));
        assert_eq!(cpu.get_pc(), 0);
        assert_eq!(cpu.get_sp(), 0);
        assert_eq!(cpu.get_register(r), v);
        assert_eq!(cpu.get_flags(), 0);
        v += 1;
    }
}

#[test]
fn call() {
    let mut cpu = setup();
    cpu.set_sp(0x23FF);
    cpu.set_pc(0x1234);
    assert_eq!(17, cpu.execute(Call(0x1567)));
    assert_eq!(cpu.get_pc(), 0x1567);
    assert_eq!(cpu.get_sp(), 0x23FD);
    assert_eq!(cpu.get_memory(cpu.get_sp() + 1), 0x12);
    assert_eq!(cpu.get_memory(cpu.get_sp()), 0x34);
    assert_eq!(cpu.registers, [0; NREGS]);
    assert_eq!(cpu.get_flags(), 0);
}

#[test]
fn ret() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.start());
    cpu.set_memory(cpu.get_sp(), 0xFF);
    cpu.set_memory(cpu.get_sp() + 1, 0x1F);
    assert_eq!(10, cpu.execute(Return));
    assert_eq!(cpu.get_pc(), 0x1FFF);
    assert_eq!(*STACK.start() + 2, cpu.get_sp());
}

#[test]
fn load_accumulator_indirect() {
    let mut cpu = setup();
    cpu.set_memory(0x2234, 0x56);
    cpu.set_memory(0x2345, 0x67);
    cpu.set_register(B, 0x22);
    cpu.set_register(C, 0x34);
    assert_eq!(7, cpu.execute(LoadAccumulatorIndirect(BC)));
    assert_eq!(0x56, cpu.get_register(A));
    cpu.set_register(D, 0x23);
    cpu.set_register(E, 0x45);
    assert_eq!(7, cpu.execute(LoadAccumulatorIndirect(DE)));
    assert_eq!(0x67, cpu.get_register(A));

    assert_eq!(cpu.get_pc(), 0);
    assert_eq!(cpu.get_sp(), 0);
    assert_eq!(cpu.get_flags(), 0);
}

#[test]
fn store_accumulator_indirect() {
    //panic!("Implement the test!");
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
        cpu.set_register(H, 0x20);
        cpu.set_register(L, v);
        cpu.set_register(r, v + 1);
        assert_eq!(7, cpu.execute(MoveToMemory(r)));
        assert_eq!(cpu.get_pc(), 0);
        assert_eq!(cpu.get_sp(), 0);
        assert_eq!(cpu.get_memory(0x2000usize | v as usize), v + 1);
        assert_eq!(cpu.get_flags(), 0);
        v += 1;
    }
}

#[test]
fn increment_register_pair() {
    let mut cpu = setup();
    for rp in [BC, DE, HL, SP] {
        cpu.set_register_pair(rp, *STACK.start() as Data16);
        assert_eq!(5, cpu.execute(IncrementRegisterPair(rp)));
        assert_eq!(1 + (*STACK.start() as Data16), cpu.get_register_pair(rp));
    }
}

#[test]
fn decrement_register_pair() {
    let mut cpu = setup();
    for rp in [BC, DE, HL, SP] {
        cpu.set_register_pair(rp, *STACK.end() as Data16);
        assert_eq!(5, cpu.execute(DecrementRegisterPair(rp)));
        assert_eq!((*STACK.end() as Data16) - 1, cpu.get_register_pair(rp));
    }
}

#[test]
fn decrement_register() {
    let mut cpu = setup();
    for r in [B, C, D, E, H, L, A] {
        cpu.set_register(r, 1);
        assert_eq!(5, cpu.execute(DecrementRegister(r)));
        assert_eq!(0, cpu.get_register(r));
        assert!(cpu.get_flag(Z));
        assert!(!cpu.get_flag(S));
        assert!(cpu.get_flag(P));
        assert!(!cpu.get_flag(CY));
        assert!(!cpu.get_flag(AC));
        assert_eq!(5, cpu.execute(DecrementRegister(r)));
        assert_eq!(-1, cpu.get_register(r) as i8);
        //assert_eq!(cpu.get_flags(), [false, true, true, true, false]);
        assert_eq!(5, cpu.execute(DecrementRegister(r)));
        assert_eq!(-2, cpu.get_register(r) as i8);
        //assert_eq!(cpu.get_flags(), [false, true, false, false, false]);
        //assert_eq!(1, 2);
    }
}

#[test]
fn increment_register() {
    let mut cpu = setup();
    for r in [B, C, D, E, H, L, A] {
        cpu.set_register(r, 0);
        assert_eq!(5, cpu.execute(IncrementRegister(r)));
        assert_eq!(1, cpu.get_register(r));
        assert!(!cpu.get_flag(Z));
        assert!(!cpu.get_flag(S));
        assert!(!cpu.get_flag(P));
        assert!(!cpu.get_flag(CY));
        assert!(!cpu.get_flag(AC));
    }
}

#[test]
fn increment_memory() {
    let mut cpu = setup();
    let addr = *RAM.start();
    cpu.set_register_pair(HL, addr as u16);
    cpu.set_memory(addr, 2);
    assert_eq!(10, cpu.execute(IncrementMemory));
    assert_eq!(3, cpu.get_memory(addr));
    assert!(!cpu.get_flag(Z));
    assert!(!cpu.get_flag(S));
    assert!(cpu.get_flag(P));
    assert!(!cpu.get_flag(CY));
    assert!(!cpu.get_flag(AC));
}

#[test]
fn decrement_memory() {
    let mut cpu = setup();
    let addr = *RAM.start();
    cpu.set_register_pair(HL, addr as u16);
    cpu.set_memory(addr, 1);
    assert_eq!(10, cpu.execute(DecrementMemory));
    assert_eq!(0, cpu.get_memory(addr));
    assert!(cpu.get_flag(Z));
    assert!(!cpu.get_flag(S));
    assert!(cpu.get_flag(P));
    assert!(!cpu.get_flag(CY));
    assert!(!cpu.get_flag(AC));
}

#[test]
fn conditional_jump() {
    let mut cpu = setup();
    assert_eq!(10, cpu.execute(ConditionalJump(NotZero, 0x0001)));
    assert_eq!(cpu.get_pc(), 0x0001);
    assert_eq!(10, cpu.execute(ConditionalJump(Zero, 0x0002)));
    assert_eq!(cpu.get_pc(), 0x0001);
    assert_eq!(10, cpu.execute(ConditionalJump(NoCarry, 0x0002)));
    assert_eq!(cpu.get_pc(), 0x0002);
    assert_eq!(10, cpu.execute(ConditionalJump(Carry, 0x0003)));
    assert_eq!(cpu.get_pc(), 0x0002);
    assert_eq!(10, cpu.execute(ConditionalJump(ParityOdd, 0x0003)));
    assert_eq!(cpu.get_pc(), 0x0003);
    assert_eq!(10, cpu.execute(ConditionalJump(ParityEven, 0x0004)));
    assert_eq!(cpu.get_pc(), 0x0003);
    assert_eq!(10, cpu.execute(ConditionalJump(Plus, 0x0004)));
    assert_eq!(cpu.get_pc(), 0x0004);
    assert_eq!(10, cpu.execute(ConditionalJump(Minus, 0x0005)));
    assert_eq!(cpu.get_pc(), 0x0004);
}

#[test]
fn conditional_call() {
    let mut cpu = setup();

    for (condition, flag, value) in [
        (NotZero, Z, false),
        (Zero, Z, true),
        (NoCarry, CY, false),
        (Carry, CY, true),
        (ParityOdd, P, false),
        (ParityEven, P, true),
        (Plus, S, false),
        (Minus, S, true),
    ] {
        cpu.set_pc(0);
        cpu.set_sp(*STACK.end());
        cpu.set_flag(flag, value);
        assert_eq!(17, cpu.execute(ConditionalCall(condition, 0x1FAB)));
        assert_eq!(0x1FAB, cpu.get_pc());
        assert_eq!(*STACK.end() - 2, cpu.get_sp());
        cpu.set_flag(flag, !value);
        assert_eq!(11, cpu.execute(ConditionalCall(condition, 0x1FFF)));
        assert_eq!(0x1FAB, cpu.get_pc());
        assert_eq!(*STACK.end() - 2, cpu.get_sp());
    }
}

#[test]
fn conditional_return() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.end());

    for (condition, flag, value) in [
        (NotZero, Z, false),
        (Zero, Z, true),
        (NoCarry, CY, false),
        (Carry, CY, true),
        (ParityOdd, P, false),
        (ParityEven, P, true),
        (Plus, S, false),
        (Minus, S, true),
    ] {
        cpu.set_pc(0);
        cpu.push(0x1FAB);
        cpu.set_flag(flag, value);
        assert_eq!(11, cpu.execute(ConditionalReturn(condition)));
        assert_eq!(0x1FAB, cpu.get_pc());
        cpu.set_pc(0);
        cpu.push(0x1FAB);
        cpu.set_flag(flag, !value);
        assert_eq!(5, cpu.execute(ConditionalReturn(condition)));
        assert_eq!(0, cpu.get_pc());
    }
}

#[test]
fn move_to_memory_immediate() {
    let mut cpu = setup();
    cpu.set_register_pair(HL, 0x2000);
    assert_eq!(10, cpu.execute(MoveToMemoryImmediate(0xFE)));
    assert_eq!(cpu.get_memory(0x2000), 0xFE);
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
            assert_eq!(5, cpu.execute(MoveRegister(t, f)));
            assert_eq!(cpu.get_register(t), v);
        }
        v += 1;
    }
}

#[test]
fn compare_immediate() {
    let mut cpu = setup();

    cpu.set_register(A, 1);
    assert_eq!(7, cpu.execute(CompareImmediate(1)));
    assert!(cpu.get_flag(Z));
    assert!(!cpu.get_flag(CY));
    cpu.set_flags(0);
    assert_eq!(7, cpu.execute(CompareImmediate(0)));
    assert!(!cpu.get_flag(Z));
    assert!(!cpu.get_flag(CY));
    cpu.set_flags(0);
    assert_eq!(7, cpu.execute(CompareImmediate(2)));
    assert!(!cpu.get_flag(Z));
    assert!(cpu.get_flag(CY));
}

#[test]
fn compare_register() {
    let mut cpu = setup();

    cpu.set_register(A, 1);
    cpu.set_register(B, 1);
    assert_eq!(4, cpu.execute(CompareRegister(B)));
    assert!(cpu.get_flag(Z));
    assert!(!cpu.get_flag(CY));
    cpu.set_flags(0);
    cpu.set_register(B, 0);
    assert_eq!(4, cpu.execute(CompareRegister(B)));
    assert!(!cpu.get_flag(Z));
    assert!(!cpu.get_flag(CY));
    cpu.set_flags(0);
    cpu.set_register(B, 2);
    assert_eq!(4, cpu.execute(CompareRegister(B)));
    assert!(!cpu.get_flag(Z));
    assert!(cpu.get_flag(CY));
}

#[test]
fn compare_memory() {
    let mut cpu = setup();

    cpu.set_register(A, 1);
    cpu.set_register_pair(HL, *RAM.start() as Data16);
    cpu.set_memory(*RAM.start(), 1);
    assert_eq!(7, cpu.execute(CompareMemory));
    assert!(cpu.get_flag(Z));
    assert!(!cpu.get_flag(CY));
    cpu.set_flags(0);
    cpu.set_memory(*RAM.start(), 0);
    assert_eq!(7, cpu.execute(CompareMemory));
    assert!(!cpu.get_flag(Z));
    assert!(!cpu.get_flag(CY));
    cpu.set_flags(0);
    cpu.set_memory(*RAM.start(), 2);
    assert_eq!(7, cpu.execute(CompareMemory));
    assert!(!cpu.get_flag(Z));
    assert!(cpu.get_flag(CY));
}

#[test]
fn push() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.end());
    let mut v = 0xA1;
    for rp in [BC, DE, HL] {
        cpu.set_register_pair(rp, v);
        let sp = cpu.get_sp();
        assert_eq!(11, cpu.execute(Push(rp)));
        assert_eq!(cpu.peek() as u16, v);
        v += 1;
        assert_eq!(cpu.get_sp(), sp - 2);
    }
}

#[test]
#[should_panic]
fn push_sp() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.end());
    assert_eq!(11, cpu.execute(Push(SP)));
}

#[test]
fn pop() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.start());
    for rp in [BC, DE, HL] {
        cpu.set_register_pair(rp, 42);
        let sp = cpu.get_sp();
        assert_eq!(10, cpu.execute(Pop(rp)));
        assert_eq!(cpu.get_register_pair(rp) as u16, 0);
        assert_eq!(cpu.get_sp(), sp + 2);
    }
}

#[test]
#[should_panic]
fn pop_sp() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.start());
    assert_eq!(10, cpu.execute(Pop(SP)));
}

#[test]
fn add_register_pair_to_hl() {
    let mut cpu = setup();
    cpu.set_register_pair(BC, 1);
    cpu.set_register_pair(DE, 2);
    cpu.set_register_pair(SP, *STACK.end() as Data16);
    cpu.set_register_pair(HL, 0xFFFD);
    assert_eq!(10, cpu.execute(AddRegisterPairToHL(BC)));
    assert!(!cpu.get_flag(CY));
    assert_eq!(0xFFFE, cpu.get_register_pair(HL));
    assert_eq!(10, cpu.execute(AddRegisterPairToHL(DE)));
    assert!(cpu.get_flag(CY));
    assert_eq!(0, cpu.get_register_pair(HL));
    assert_eq!(10, cpu.execute(AddRegisterPairToHL(SP)));
    assert!(!cpu.get_flag(CY));
    assert_eq!(*STACK.end() as Data16, cpu.get_register_pair(HL));
    assert_eq!(10, cpu.execute(AddRegisterPairToHL(HL)));
    assert!(!cpu.get_flag(CY));
    assert_eq!(2 * (*STACK.end() as Data16), cpu.get_register_pair(HL));
}

#[test]
fn exchange_hl_with_de() {
    let mut cpu = setup();
    cpu.set_register_pair(HL, 0x1234);
    cpu.set_register_pair(DE, 0xABCD);
    assert_eq!(4, cpu.execute(ExchangeHLWithDE));
    assert_eq!(0xABCD, cpu.get_register_pair(HL));
    assert_eq!(0x1234, cpu.get_register_pair(DE));
}

#[test]
fn exchange_sp_with_hl() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.end());
    cpu.push(0xFEDC);
    cpu.set_register_pair(HL, 0xABCD);
    assert_eq!(18, cpu.execute(ExchangeSPWithHL));
    assert_eq!(0xFEDC, cpu.get_register_pair(HL));
    assert_eq!(*STACK.end() - 2, cpu.get_sp());
    assert_eq!(0xABCD, cpu.peek());
}

#[test]
fn move_from_memory() {
    let mut cpu = setup();
    cpu.set_register_pair(HL, *RAM.start() as Data16);
    cpu.set_memory(*RAM.start(), 0xAB);
    for r in [A, B, C, D, E] {
        assert_eq!(cpu.get_register(r), 0);
        assert_eq!(7, cpu.execute(MoveFromMemory(r)));
        assert_eq!(cpu.get_register(r), 0xAB);
    }
}

#[test]
fn set_carry() {
    let mut cpu = setup();
    assert_eq!(4, cpu.execute(SetCarry));
    assert!(cpu.get_flag(CY));
}

#[test]
fn push_processor_status_word() {
    let mut cpu = setup();
    cpu.set_flags(0xFF);
    cpu.set_register(A, 0xAB);
    cpu.set_sp(*STACK.end());
    assert_eq!(11, cpu.execute(PushProcessorStatusWord));
    assert_eq!(*STACK.end() - 2, cpu.get_sp());
    assert_eq!(0b1111_1111, cpu.pop_data()); // Flags
    assert_eq!(0xAB, cpu.pop_data()); // A register
}

#[test]
fn pop_processor_status_word() {
    let mut cpu = setup();
    cpu.set_flags(0xFF);
    cpu.set_register(A, 0xAB);
    cpu.set_sp(*STACK.end());
    cpu.execute(PushProcessorStatusWord);
    cpu.set_flags(0);
    cpu.set_register(A, 0);
    assert_eq!(10, cpu.execute(PopProcessorStatusWord));
    assert_eq!(0xFF, cpu.get_flags());
    assert_eq!(0xAB, cpu.get_register(A));
    assert_eq!(*STACK.end(), cpu.get_sp());
}

#[test]
fn rotate_right() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1000_0001);
    cpu.set_flags(0);
    assert_eq!(4, cpu.execute(RotateRight));
    assert_eq!(0b1100_0000, cpu.get_register(A));
    assert!(cpu.get_flag(CY));
    cpu.set_register(A, 0b1000_0010);
    assert_eq!(4, cpu.execute(RotateRight));
    assert_eq!(0b0100_0001, cpu.get_register(A));
    assert!(!cpu.get_flag(CY));
}

#[test]
fn rotate_left() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1000_0001);
    cpu.set_flags(0);
    assert_eq!(4, cpu.execute(RotateLeft));
    assert_eq!(0b0000_0011, cpu.get_register(A));
    assert!(cpu.get_flag(CY));
    cpu.set_register(A, 0b0100_0001);
    assert_eq!(4, cpu.execute(RotateLeft));
    assert_eq!(0b1000_0010, cpu.get_register(A));
    assert!(!cpu.get_flag(CY));
}

#[test]
fn rotate_right_through_carry() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1000_0001);
    cpu.set_flags(0);
    assert_eq!(4, cpu.execute(RotateRightThroughCarry));
    assert_eq!(0b0100_0000, cpu.get_register(A));
    assert!(cpu.get_flag(CY));
    assert_eq!(4, cpu.execute(RotateRightThroughCarry));
    assert_eq!(0b1010_0000, cpu.get_register(A));
    assert!(!cpu.get_flag(CY));
}

#[test]
fn and_immediate() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1010_1010);
    cpu.set_flags(0);
    cpu.set_flag(CY, true);
    cpu.set_flag(AC, true);
    assert_eq!(7, cpu.execute(AndImmediate(0b1111_0000)));
    assert_eq!(0b1010_1010 & 0b1111_0000, cpu.get_register(A));
    assert!(!cpu.get_flag(CY));
    assert!(!cpu.get_flag(AC));
}

#[test]
fn and_memory() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1010_1010);
    cpu.set_flag(CY, true);
    cpu.set_register_pair(HL, *RAM.start() as Data16);
    cpu.set_memory(*RAM.start(), 0b1111_0000);
    assert_eq!(7, cpu.execute(AndMemory));
    assert_eq!(0b1010_1010 & 0b1111_0000, cpu.get_register(A));
    assert!(!cpu.get_flag(CY));
}

#[test]
fn add_immediate() {
    let mut cpu = setup();
    cpu.set_register(A, 0xFF);
    cpu.set_flags(0);
    assert_eq!(7, cpu.execute(AddImmediate(1)));
    assert_eq!(0, cpu.get_register(A));
    assert!(cpu.get_flag(CY));
}

#[test]
fn add_memory() {
    let mut cpu = setup();
    cpu.set_register(A, 0xFF);
    cpu.set_register_pair(HL, *RAM.start() as Data16);
    cpu.set_memory(*RAM.start(), 1);
    assert_eq!(7, cpu.execute(AddMemory));
    assert_eq!(0, cpu.get_register(A));
    assert!(cpu.get_flag(CY));
}

#[test]
fn add_register() {
    let mut cpu = setup();
    cpu.set_register(A, 0xFF);
    cpu.set_register(B, 0x1);
    assert_eq!(4, cpu.execute(AddRegister(B)));
    assert_eq!(0, cpu.get_register(A));
    assert!(cpu.get_flag(CY));
    assert!(cpu.get_flag(Z));
}

#[test]
fn add_register_with_carry() {
    //panic!("Implement the test!");
}

#[test]
fn subtract_register() {
    let mut cpu = setup();
    cpu.set_register(A, 0);
    cpu.set_register(B, 0x1);
    assert_eq!(4, cpu.execute(SubtractRegister(B)));
    assert_eq!(0xFF, cpu.get_register(A));
    assert!(cpu.get_flag(CY));
    assert!(cpu.get_flag(P));
}

#[test]
fn subtract_immediate() {
    let mut cpu = setup();
    cpu.set_register(A, 0);
    cpu.set_flags(0);
    assert_eq!(7, cpu.execute(SubtractImmediate(1)));
    assert_eq!(0xFF, cpu.get_register(A));
    assert!(cpu.get_flag(CY));
}

#[test]
fn subtract_immediate_with_borrow() {
    let mut cpu = setup();
    cpu.set_register(A, 0);
    cpu.set_flags(0);
    cpu.set_flag(CY, true);
    assert_eq!(7, cpu.execute(SubtractImmediateWithBorrow(1)));
    assert_eq!(0xFE, cpu.get_register(A));
    assert!(cpu.get_flag(CY));
    cpu.set_flag(CY, false);
    assert_eq!(7, cpu.execute(SubtractImmediateWithBorrow(1)));
    assert_eq!(0xFD, cpu.get_register(A));
    assert!(!cpu.get_flag(CY));
}

#[test]
fn load_accumulator_direct() {
    let mut cpu = setup();
    let addr = *RAM.start();
    cpu.set_memory(addr, 0xAB);
    assert_eq!(13, cpu.execute(LoadAccumulatorDirect(addr)));
    assert_eq!(0xAB, cpu.get_register(A));
}

#[test]
fn store_accumulator_direct() {
    let mut cpu = setup();
    let addr = 0x2000;
    cpu.set_register(A, 0xAB);
    assert_eq!(13, cpu.execute(StoreAccumulatorDirect(addr)));
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
        assert_eq!(4, cpu.execute(XorRegister(r)));
        assert_eq!(0b1110_0101, cpu.get_register(A));
        assert!(!cpu.get_flag(CY));
        assert!(!cpu.get_flag(AC));
    }
}

#[test]
fn disable_interrupts() {
    let mut cpu = setup();
    cpu.interruptable = true;
    assert_eq!(4, cpu.execute(DisableInterrupts));
    assert!(!cpu.interruptable);
}

#[test]
fn enable_interrupts() {
    let mut cpu = setup();
    assert_eq!(4, cpu.execute(EnableInterrupts));
    assert!(cpu.interruptable);
}

#[test]
fn and_register() {
    let mut cpu = setup();
    for r in [B, C, D, E, H, L] {
        cpu.set_flag(CY, true);
        cpu.set_register(A, 0b1010_1010);
        cpu.set_register(r, 0b0100_1111);
        assert_eq!(4, cpu.execute(AndRegister(r)));
        assert_eq!(0b0000_1010, cpu.get_register(A));
        assert!(!cpu.get_flag(CY));
    }
}

#[test]
fn input() {
    let mut cpu = setup();
    for port in 0..NPORTS {
        assert_eq!(10, cpu.execute(Input(port as u8)));
    }
}

#[test]
fn output() {
    let mut cpu = setup();
    for port in 0..NPORTS {
        assert_eq!(10, cpu.execute(Output(port as u8)));
    }
}

#[test]
fn restart() {
    let mut cpu = setup();
    cpu.set_sp(*STACK.end());
    cpu.set_pc(0x1234);
    assert_eq!(11, cpu.execute(Restart(0xFF)));
    assert_eq!(cpu.get_pc(), 0x7F8);
    assert_eq!(cpu.get_sp(), *STACK.end() - 2);
    assert_eq!(cpu.get_memory(cpu.get_sp() + 1), 0x12);
    assert_eq!(cpu.get_memory(cpu.get_sp()), 0x34);
    assert_eq!(cpu.registers, [0; NREGS]);
    assert_eq!(cpu.get_flags(), 0);
}

#[test]
fn or_memory() {
    let mut cpu = setup();
    cpu.set_register_pair(HL, 0x2000);
    cpu.set_memory(0x2000, 0b1010_1010);
    cpu.set_register(A, 0b0101_0101);
    cpu.set_flag(AC, true);
    cpu.set_flag(CY, true);
    assert_eq!(7, cpu.execute(OrMemory));
    assert_eq!(0b1111_1111, cpu.get_register(A));
    assert!(!cpu.get_flag(CY));
    assert!(!cpu.get_flag(AC));
}

#[test]
fn or_register() {
    let mut cpu = setup();
    for r in [B, C, D, E, H, L] {
        cpu.set_register(r, 0b1010_1010);
        cpu.set_register(A, 0b0101_0101);
        cpu.set_flag(AC, true);
        cpu.set_flag(CY, true);
        assert_eq!(4, cpu.execute(OrRegister(r)));
        assert_eq!(0b1111_1111, cpu.get_register(A));
        assert!(!cpu.get_flag(CY));
        assert!(!cpu.get_flag(AC));
    }
}

#[test]
fn or_immediate() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1010_1010);
    cpu.set_flag(AC, true);
    cpu.set_flag(CY, true);
    assert_eq!(7, cpu.execute(OrImmediate(0b0101_0101)));
    assert_eq!(0b1111_1111, cpu.get_register(A));
    assert!(!cpu.get_flag(CY));
    assert!(!cpu.get_flag(AC));
}

#[test]
fn load_hl_direct() {
    let mut cpu = setup();
    cpu.set_memory(*RAM.start(), 0xCD);
    cpu.set_memory(*RAM.start() + 1, 0xAB);
    assert_eq!(16, cpu.execute(LoadHLDirect(*RAM.start())));
    assert_eq!(0xABCD, cpu.get_register_pair(HL));
}

#[test]
fn store_hl_direct() {
    let mut cpu = setup();
    cpu.set_register_pair(HL, 0xABCD);
    assert_eq!(16, cpu.execute(StoreHLDirect(*RAM.start())));
    assert_eq!(0xCD, cpu.get_memory(*RAM.start()));
    assert_eq!(0xAB, cpu.get_memory(*RAM.start() + 1));
}

#[test]
fn shift_register() {
    let mut cpu = setup();

    cpu.set_register(A, 0x1);
    assert_eq!(10, cpu.execute(Output(4)));
    assert_eq!(cpu.shift, 0b0000_0001_0000_0000);
    cpu.set_register(A, 0x3);
    assert_eq!(10, cpu.execute(Output(4)));
    assert_eq!(cpu.shift, 0b0000_0011_0000_0001);
    assert_eq!(0x3, cpu.get_bus_in(3));
    cpu.set_register(A, 0x7);
    assert_eq!(10, cpu.execute(Output(2)));
    assert_eq!(0b1000_0000, cpu.get_bus_in(3));
    cpu.set_register(A, 0x6);
    assert_eq!(10, cpu.execute(Output(2)));
    assert_eq!(0b1100_0000, cpu.get_bus_in(3));
}

#[test]
fn complement_accumulator() {
    let mut cpu = setup();
    cpu.set_register(A, 0b1010_1010);
    assert_eq!(4, cpu.execute(ComplementAccumulator));
    assert_eq!(0b0101_0101, cpu.get_register(A));
}

#[test]
fn add() {
    let mut cpu = setup();
    cpu.add(0);
    assert_eq!(0, cpu.get_register(A));
    assert!(!cpu.get_flag(AC));
    assert!(!cpu.get_flag(CY));
    cpu.add(0x10);
    assert!(!cpu.get_flag(AC));
    assert!(!cpu.get_flag(CY));
    cpu.set_register(A, 0x8);
    cpu.add(0x8);
    assert!(cpu.get_flag(AC));
    assert!(!cpu.get_flag(CY));
    cpu.add(0xFF - 0x10 + 1);
    assert!(!cpu.get_flag(AC));
    assert!(cpu.get_flag(CY));
    assert_eq!(0, cpu.get_register(A));
}
