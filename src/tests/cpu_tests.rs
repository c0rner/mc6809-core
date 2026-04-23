//   Copyright 2026 Martin Åkesson
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

//! Integration tests for the CPU — load short programs and verify behavior.

use crate::{Cpu, Memory, registers::CC_E};

/// Simple 64KB flat RAM mem for testing.
struct TestMem {
    mem: [u8; 65536],
}

impl TestMem {
    fn new() -> Self {
        Self { mem: [0u8; 65536] }
    }

    fn set_reset_vector(&mut self, addr: u16) {
        self.mem[0xFFFE] = (addr >> 8) as u8;
        self.mem[0xFFFF] = addr as u8;
    }

    /// Write a sequence of bytes starting at the given address.
    fn write_bytes(&mut self, addr: u16, bytes: &[u8]) {
        let start = addr as usize;
        self.mem[start..start + bytes.len()].copy_from_slice(bytes);
    }
}

impl Memory for TestMem {
    fn read(&mut self, addr: u16) -> u8 {
        println!(
            "TestBus: Read {:04X} = {:02X}",
            addr, self.mem[addr as usize]
        );
        self.mem[addr as usize]
    }
    fn write(&mut self, addr: u16, val: u8) {
        println!("TestBus: Write {:04X} = {:02X}", addr, val);
        self.mem[addr as usize] = val;
    }
}

fn setup(program: &[u8], start: u16) -> (Cpu, TestMem) {
    let mut mem = TestMem::new();
    mem.set_reset_vector(start);
    mem.write_bytes(start, program);
    let mut cpu = Cpu::new();
    cpu.reset(&mut mem);
    (cpu, mem)
}

// ---- Basic instruction tests ----

#[test]
fn nop_advances_pc() {
    let (mut cpu, mut mem) = setup(&[0x12], 0x0400); // NOP
    let cyc = cpu.step(&mut mem);
    assert_eq!(cpu.reg.pc, 0x0401);
    assert_eq!(cyc, 2);
}

#[test]
fn lda_immediate() {
    let (mut cpu, mut mem) = setup(&[0x86, 0x42], 0x0400); // LDA #$42
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x42);
    assert!(!cpu.reg.cc.zero());
    assert!(!cpu.reg.cc.negative());
}

#[test]
fn ldb_immediate() {
    let (mut cpu, mut mem) = setup(&[0xC6, 0xFF], 0x0400); // LDB #$FF
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.b(), 0xFF);
    assert!(cpu.reg.cc.negative());
}

#[test]
fn ldd_immediate() {
    let (mut cpu, mut mem) = setup(&[0xCC, 0x12, 0x34], 0x0400); // LDD #$1234
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.d, 0x1234);
    assert_eq!(cpu.reg.a(), 0x12);
    assert_eq!(cpu.reg.b(), 0x34);
}

#[test]
fn ldx_immediate() {
    let (mut cpu, mut mem) = setup(&[0x8E, 0xAB, 0xCD], 0x0400); // LDX #$ABCD
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.x, 0xABCD);
}

#[test]
fn sta_direct() {
    // LDA #$42, STA $10 (DP=0, so address $0010)
    let (mut cpu, mut mem) = setup(&[0x86, 0x42, 0x97, 0x10], 0x0400);
    cpu.step(&mut mem); // LDA
    cpu.step(&mut mem); // STA
    assert_eq!(mem.mem[0x0010], 0x42);
}

#[test]
fn adda_immediate() {
    // LDA #$10, ADDA #$20
    let (mut cpu, mut mem) = setup(&[0x86, 0x10, 0x8B, 0x20], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x30);
}

#[test]
fn suba_immediate() {
    // LDA #$30, SUBA #$10
    let (mut cpu, mut mem) = setup(&[0x86, 0x30, 0x80, 0x10], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x20);
}

#[test]
fn cmpa_immediate_flags() {
    // LDA #$42, CMPA #$42 → Z set
    let (mut cpu, mut mem) = setup(&[0x86, 0x42, 0x81, 0x42], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert!(cpu.reg.cc.zero());
    assert_eq!(cpu.reg.a(), 0x42); // CMP doesn't change A
}

#[test]
fn anda_immediate() {
    // LDA #$FF, ANDA #$0F
    let (mut cpu, mut mem) = setup(&[0x86, 0xFF, 0x84, 0x0F], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x0F);
    assert!(!cpu.reg.cc.overflow());
}

#[test]
fn ora_immediate() {
    // LDA #$F0, ORA #$0F
    let (mut cpu, mut mem) = setup(&[0x86, 0xF0, 0x8A, 0x0F], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0xFF);
}

#[test]
fn eora_immediate() {
    // LDA #$FF, EORA #$0F
    let (mut cpu, mut mem) = setup(&[0x86, 0xFF, 0x88, 0x0F], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0xF0);
}

#[test]
fn nega() {
    // LDA #$01, NEGA
    let (mut cpu, mut mem) = setup(&[0x86, 0x01, 0x40], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0xFF);
}

#[test]
fn coma() {
    // LDA #$55, COMA
    let (mut cpu, mut mem) = setup(&[0x86, 0x55, 0x43], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0xAA);
}

#[test]
fn inca_deca() {
    // LDA #$7F, INCA (overflow), DECA (back to 7F)
    let (mut cpu, mut mem) = setup(&[0x86, 0x7F, 0x4C, 0x4A], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem); // INCA
    assert_eq!(cpu.reg.a(), 0x80);
    assert!(cpu.reg.cc.overflow());
    cpu.step(&mut mem); // DECA
    assert_eq!(cpu.reg.a(), 0x7F);
}

#[test]
fn clra() {
    // LDA #$42, CLRA
    let (mut cpu, mut mem) = setup(&[0x86, 0x42, 0x4F], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x00);
    assert!(cpu.reg.cc.zero());
    assert!(!cpu.reg.cc.carry());
}

#[test]
fn lsla() {
    // LDA #$81, LSLA
    let (mut cpu, mut mem) = setup(&[0x86, 0x81, 0x48], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x02);
    assert!(cpu.reg.cc.carry()); // bit 7 was set
}

#[test]
fn lsra() {
    // LDA #$03, LSRA
    let (mut cpu, mut mem) = setup(&[0x86, 0x03, 0x44], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x01);
    assert!(cpu.reg.cc.carry()); // bit 0 was set
}

// ---- Jump tests ----

#[test]
fn jmp_direct() {
    // JMP $1234
    let (mut cpu, mut mem) = setup(&[0x7E, 0x12, 0x34], 0x0400);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.pc, 0x1234);
}

#[test]
fn jmp_indexed() {
    // JMP [$0010]
    // Place target address 0x5678 at $0010
    let (mut cpu, mut mem) = setup(&[0x6E, 0x9F, 0x00, 0x10], 0x0400);
    mem.mem[0x0010] = 0x56;
    mem.mem[0x0011] = 0x78;
    cpu.step(&mut mem);
    println!("{:?}", cpu);
    assert_eq!(cpu.reg.pc, 0x5678);
}

// ---- Branch tests ----

#[test]
fn bra_always() {
    // BRA +2 (skip next 2 bytes)
    let (mut cpu, mut mem) = setup(&[0x20, 0x02, 0x12, 0x12, 0x12], 0x0400);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.pc, 0x0404); // 0x0402 + 2
}

#[test]
fn beq_taken() {
    // LDA #$00, BEQ +2
    let (mut cpu, mut mem) = setup(&[0x86, 0x00, 0x27, 0x02, 0x12, 0x12, 0x12], 0x0400);
    cpu.step(&mut mem); // LDA #0 → Z set
    cpu.step(&mut mem); // BEQ +2
    assert_eq!(cpu.reg.pc, 0x0406);
}

#[test]
fn beq_not_taken() {
    // LDA #$01, BEQ +2
    let (mut cpu, mut mem) = setup(&[0x86, 0x01, 0x27, 0x02, 0x12], 0x0400);
    cpu.step(&mut mem); // LDA #1 → Z clear
    cpu.step(&mut mem); // BEQ +2 (not taken)
    assert_eq!(cpu.reg.pc, 0x0404); // fell through
}

#[test]
fn bne_taken() {
    // LDA #$01, BNE +2
    let (mut cpu, mut mem) = setup(&[0x86, 0x01, 0x26, 0x02, 0x12, 0x12, 0x12], 0x0400);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.pc, 0x0406);
}

#[test]
fn bra_backward() {
    // At 0x0402: BRA -2 → 0x0402 (infinite loop)
    let (mut cpu, mut mem) = setup(&[0x12, 0x12, 0x20, 0xFE], 0x0400);
    cpu.step(&mut mem); // NOP → PC=0x0401
    cpu.step(&mut mem); // NOP → PC=0x0402
    cpu.step(&mut mem); // BRA -2 → PC=0x0402
    assert_eq!(cpu.reg.pc, 0x0402);
}

// ---- Subroutine tests ----

#[test]
fn bsr_rts() {
    // BSR +2, NOP, NOP, RTS (at +4)
    // Program: BSR to 0x0404, the subroutine does RTS
    let (mut cpu, mut mem) = setup(
        &[
            0x8D, 0x02, // BSR +2 → subroutine at 0x0404
            0x12, // NOP (return point)
            0x12, // NOP
            0x39, // RTS
        ],
        0x0400,
    );
    // Set up S register
    cpu.reg.s = 0x8000;
    cpu.step(&mut mem); // BSR
    assert_eq!(cpu.reg.pc, 0x0404);
    cpu.step(&mut mem); // RTS
    assert_eq!(cpu.reg.pc, 0x0402); // returned to instruction after BSR
}

#[test]
fn jsr_extended_rts() {
    let (mut cpu, mut mem) = setup(
        &[
            0xBD, 0x04, 0x10, // JSR $0410
        ],
        0x0400,
    );
    // Place RTS at 0x0410
    mem.mem[0x0410] = 0x39;
    cpu.reg.s = 0x8000;
    cpu.step(&mut mem); // JSR
    assert_eq!(cpu.reg.pc, 0x0410);
    cpu.step(&mut mem); // RTS
    assert_eq!(cpu.reg.pc, 0x0403);
}

// ---- Stack tests ----

#[test]
fn pshs_puls_a() {
    // LDA #$42, PSHS A, LDA #$00, PULS A
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x42, // LDA #$42
            0x34, 0x02, // PSHS A
            0x86, 0x00, // LDA #$00
            0x35, 0x02, // PULS A
        ],
        0x0400,
    );
    cpu.reg.s = 0x8000;
    cpu.step(&mut mem); // LDA #$42
    cpu.step(&mut mem); // PSHS A
    assert_eq!(cpu.reg.s, 0x7FFF);
    cpu.step(&mut mem); // LDA #$00
    assert_eq!(cpu.reg.a(), 0x00);
    cpu.step(&mut mem); // PULS A
    assert_eq!(cpu.reg.a(), 0x42);
    assert_eq!(cpu.reg.s, 0x8000);
}

#[test]
fn pshs_puls_multiple() {
    // PSHS A,B,X then PULS A,B,X
    let (mut cpu, mut mem) = setup(
        &[
            0x34, 0x16, // PSHS A,B,X  (bits: 0x02 A + 0x04 B + 0x10 X)
            0x35, 0x16, // PULS A,B,X
        ],
        0x0400,
    );
    cpu.reg.s = 0x8000;
    cpu.reg.set_a(0xAA);
    cpu.reg.set_b(0xBB);
    cpu.reg.x = 0x1234;
    cpu.step(&mut mem); // PSHS
    assert_eq!(cpu.reg.s, 0x7FFC); // 4 bytes pushed (A=1, B=1, X=2)
    // Clobber registers
    cpu.reg.set_a(0x00);
    cpu.reg.set_b(0x00);
    cpu.reg.x = 0x0000;
    cpu.step(&mut mem); // PULS
    assert_eq!(cpu.reg.a(), 0xAA);
    assert_eq!(cpu.reg.b(), 0xBB);
    assert_eq!(cpu.reg.x, 0x1234);
    assert_eq!(cpu.reg.s, 0x8000);
}

// ---- Transfer / Exchange ----

#[test]
fn tfr_a_to_b() {
    // LDA #$42, TFR A,B
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x42, // LDA #$42
            0x1F, 0x89, // TFR A,B
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.b(), 0x42);
    assert_eq!(cpu.reg.a(), 0x42); // A unchanged
}

#[test]
fn exg_a_b() {
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0xAA, // LDA #$AA
            0xC6, 0x55, // LDB #$55
            0x1E, 0x89, // EXG A,B
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x55);
    assert_eq!(cpu.reg.b(), 0xAA);
}

#[test]
fn tfr_d_to_x() {
    // LDD #$1234, TFR D,X
    let (mut cpu, mut mem) = setup(
        &[
            0xCC, 0x12, 0x34, // LDD #$1234
            0x1F, 0x01, // TFR D,X
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.x, 0x1234);
}

// ---- MUL ----

#[test]
fn mul_instruction() {
    // LDA #10, LDB #20, MUL → D = 200
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x0A, // LDA #10
            0xC6, 0x14, // LDB #20
            0x3D, // MUL
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.d, 200);
}

// ---- ABX ----

#[test]
fn abx_instruction() {
    // LDX #$1000, LDB #$42, ABX → X = $1042
    let (mut cpu, mut mem) = setup(
        &[
            0x8E, 0x10, 0x00, // LDX #$1000
            0xC6, 0x42, // LDB #$42
            0x3A, // ABX
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.x, 0x1042);
}

// ---- SEX ----

#[test]
fn sex_instruction() {
    // LDB #$80, SEX → D = $FF80
    let (mut cpu, mut mem) = setup(
        &[
            0xC6, 0x80, // LDB #$80
            0x1D, // SEX
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.d, 0xFF80);
    assert!(cpu.reg.cc.negative());
}

// ---- ORCC / ANDCC ----

#[test]
fn orcc_andcc() {
    let (mut cpu, mut mem) = setup(
        &[
            0x1A, 0xFF, // ORCC #$FF  → all flags set
            0x1C, 0x00, // ANDCC #$00 → all flags clear
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.cc.to_byte(), 0xFF);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.cc.to_byte(), 0x00);
}

// ---- SUBD immediate ----

#[test]
fn subd_immediate() {
    // LDD #$1000, SUBD #$0001
    let (mut cpu, mut mem) = setup(
        &[
            0xCC, 0x10, 0x00, // LDD #$1000
            0x83, 0x00, 0x01, // SUBD #$0001
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.d, 0x0FFF);
}

// ---- ADDD immediate ----

#[test]
fn addd_immediate() {
    // LDD #$1000, ADDD #$0234
    let (mut cpu, mut mem) = setup(
        &[
            0xCC, 0x10, 0x00, // LDD #$1000
            0xC3, 0x02, 0x34, // ADDD #$0234
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.d, 0x1234);
}

// ---- Indexed addressing basic ----

#[test]
fn lda_indexed_zero_offset() {
    // LDX #$0500, write $42 at $0500, LDA ,X
    let (mut cpu, mut mem) = setup(
        &[
            0x8E, 0x05, 0x00, // LDX #$0500
            0xA6, 0x84, // LDA ,X (zero offset, non-indirect)
        ],
        0x0400,
    );
    mem.mem[0x0500] = 0x42;
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x42);
}

#[test]
fn lda_indexed_5bit_offset() {
    // LDX #$0500, LDA 3,X (post-byte: 0b0_00_00011 = 0x03)
    let (mut cpu, mut mem) = setup(
        &[
            0x8E, 0x05, 0x00, // LDX #$0500
            0xA6, 0x03, // LDA 3,X
        ],
        0x0400,
    );
    mem.mem[0x0503] = 0x99;
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x99);
}

#[test]
fn lda_indexed_postinc2() {
    // LDX #$0500, LDA ,X++ (post-byte: 0x81)
    let (mut cpu, mut mem) = setup(
        &[
            0x8E, 0x05, 0x00, // LDX #$0500
            0xA6, 0x81, // LDA ,X++
        ],
        0x0400,
    );
    mem.mem[0x0500] = 0xAB;
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0xAB);
    assert_eq!(cpu.reg.x, 0x0502);
}

#[test]
fn lda_indexed_predec2() {
    // LDX #$0502, LDA ,--X (post-byte: 0x83)
    let (mut cpu, mut mem) = setup(
        &[
            0x8E, 0x05, 0x02, // LDX #$0502
            0xA6, 0x83, // LDA ,--X
        ],
        0x0400,
    );
    mem.mem[0x0500] = 0xCD;
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0xCD);
    assert_eq!(cpu.reg.x, 0x0500);
}

// ---- LEAX ----

#[test]
fn leax_indexed() {
    // LDX #$1000, LEAX 5,X → X = $1005
    let (mut cpu, mut mem) = setup(
        &[
            0x8E, 0x10, 0x00, // LDX #$1000
            0x30, 0x05, // LEAX 5,X (5-bit offset = 5)
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.x, 0x1005);
    assert!(!cpu.reg.cc.zero());
}

// ---- SWI ----

#[test]
fn swi_instruction() {
    let (mut cpu, mut mem) = setup(
        &[0x3F], // SWI
        0x0400,
    );
    cpu.reg.s = 0x8000;
    // Set SWI vector
    mem.mem[0xFFFA] = 0x10;
    mem.mem[0xFFFB] = 0x00;

    cpu.step(&mut mem);
    assert_eq!(cpu.reg.pc, 0x1000);
    assert!(cpu.reg.cc.irq_inhibit());
    assert!(cpu.reg.cc.firq_inhibit());
    assert!(cpu.reg.cc.entire());
    // S should have decremented by 12 bytes (entire state)
    assert_eq!(cpu.reg.s, 0x8000 - 12);
}

// ---- RTI (full from NMI) ----

#[test]
fn rti_full() {
    // Simulate: push entire state (E=1) and PC, then RTI
    let mut mem = TestMem::new();
    mem.set_reset_vector(0x0400);
    mem.mem[0x0400] = 0x3B; // RTI

    let mut cpu = Cpu::new();
    cpu.reset(&mut mem);
    cpu.reg.s = 0x8000;

    // Manually push entire state (NMI style: CC, A, B, DP, X, Y, U, PC)
    let return_pc: u16 = 0x1234;
    let cc_byte: u8 = CC_E; // E=1 for full state
    cpu.reg.s -= 1;
    mem.mem[cpu.reg.s as usize] = (return_pc & 0xFF) as u8; // PC lo
    cpu.reg.s -= 1;
    mem.mem[cpu.reg.s as usize] = (return_pc >> 8) as u8; // PC hi
    cpu.reg.s -= 9; // A, B, DP, X, Y, U; Skip 9 bytes (1+1+1+2+2+2) for remaining registers
    cpu.reg.s -= 1;
    mem.mem[cpu.reg.s as usize] = cc_byte; // CC

    cpu.step(&mut mem); // RTI
    assert_eq!(cpu.reg.pc, 0x1234);
    assert!(cpu.reg.cc.entire());
}

// ---- RTI (short from FIRQ) ----

#[test]
fn rti_short() {
    // Simulate: push CC (E=0) and PC, then RTI
    let mut mem = TestMem::new();
    mem.set_reset_vector(0x0400);
    mem.mem[0x0400] = 0x3B; // RTI

    let mut cpu = Cpu::new();
    cpu.reset(&mut mem);
    cpu.reg.s = 0x8000;

    // Manually push a short frame (FIRQ style: CC, PC)
    let return_pc: u16 = 0x1234;
    let cc_byte: u8 = 0x00; // E=0
    cpu.reg.s -= 1;
    mem.mem[cpu.reg.s as usize] = (return_pc & 0xFF) as u8; // PC lo
    cpu.reg.s -= 1;
    mem.mem[cpu.reg.s as usize] = (return_pc >> 8) as u8; // PC hi
    cpu.reg.s -= 1;
    mem.mem[cpu.reg.s as usize] = cc_byte; // CC

    cpu.step(&mut mem); // RTI
    assert_eq!(cpu.reg.pc, 0x1234);
    assert!(!cpu.reg.cc.entire());
}

// ---- Page 1 long branch ----

#[test]
fn lbeq_taken() {
    // LDA #0, LBEQ +$0100
    // After fetching the 4-byte LBEQ instruction (at 0x0402..0x0405),
    // PC = 0x0406. The 16-bit offset 0x0100 is added to that.
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x00, // LDA #$00
            0x10, 0x27, 0x01, 0x00, // LBEQ +$0100
        ],
        0x0400,
    );
    cpu.step(&mut mem); // LDA → Z set
    cpu.step(&mut mem); // LBEQ
    assert_eq!(cpu.reg.pc, 0x0406 + 0x0100);
}

#[test]
fn lbeq_not_taken() {
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x01, // LDA #$01
            0x10, 0x27, 0x01, 0x00, // LBEQ +$0100
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.pc, 0x0406); // fell through
}

// ---- Page 1 LDY ----

#[test]
fn ldy_immediate() {
    let (mut cpu, mut mem) = setup(
        &[0x10, 0x8E, 0xBE, 0xEF], // LDY #$BEEF
        0x0400,
    );
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.y, 0xBEEF);
}

// ---- Page 1 LDS ----

#[test]
fn lds_immediate() {
    let (mut cpu, mut mem) = setup(
        &[0x10, 0xCE, 0x80, 0x00], // LDS #$8000
        0x0400,
    );
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.s, 0x8000);
}

// ---- Page 1 Immediate A/D/X ----

#[test]
fn cmpa_immediate() {
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x42, // LDA #$42
            0x81, 0x42, // CMPA #$42
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert!(cpu.reg.cc.zero());
}

// ---- Page 1 Extended A/D/X ----

#[test]
fn cmpa_extended() {
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x42, // LDA #$42
            0xB1, 0x04, 0x10, // CMPA $0410
        ],
        0x0400,
    );
    mem.mem[0x0410] = 0x42;
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert!(cpu.reg.cc.zero());
}

// ---- Page 2 CMPU ----

#[test]
fn cmpu_immediate() {
    let (mut cpu, mut mem) = setup(
        &[
            0xCE, 0x10, 0x00, // LDU #$1000
            0x11, 0x83, 0x10, 0x00, // CMPU #$1000
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert!(cpu.reg.cc.zero());
}

// ---- Counting loop integration test ----

#[test]
fn counting_loop() {
    // Count from 0 to 10 in register B using a loop:
    //   CLRB          ; B = 0
    // loop:
    //   INCB          ; B++
    //   CMPB #10
    //   BNE loop
    //   SWI           ; halt (we'll use it as a stop marker)
    let program: &[u8] = &[
        0x5F, // CLRB
        0x5C, // INCB
        0xC1, 0x0A, // CMPB #10
        0x26, 0xFB, // BNE -5 (back to INCB)
        0x3F, // SWI
    ];

    let (mut cpu, mut mem) = setup(program, 0x0400);
    cpu.reg.s = 0x8000;
    // Set SWI vector to a known address so execution doesn't fly off
    mem.mem[0xFFFA] = 0xFF;
    mem.mem[0xFFFB] = 0x00;

    // Run until SWI is hit (PC jumps to $FF00)
    for _ in 0..200 {
        cpu.step(&mut mem);
        if cpu.reg.pc == 0xFF00 {
            break;
        }
    }
    assert_eq!(cpu.reg.b(), 10);
    assert_eq!(cpu.reg.pc, 0xFF00); // SWI vector
}

// ---- Direct page tests ----

#[test]
fn direct_page_register() {
    // Set DP=$10 via TFR, then LDA <$20 reads from $1020
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x10, // LDA #$10
            0x1F, 0x8B, // TFR A,DP
            0x96, 0x20, // LDA <$20 (direct page: DP:$20 = $1020)
        ],
        0x0400,
    );
    mem.mem[0x1020] = 0x77;
    cpu.step(&mut mem); // LDA #$10
    cpu.step(&mut mem); // TFR A,DP
    assert_eq!(cpu.reg.dp, 0x10);
    cpu.step(&mut mem); // LDA <$20
    assert_eq!(cpu.reg.a(), 0x77);
}

// ---- Extended addressing ----

#[test]
fn lda_extended() {
    let (mut cpu, mut mem) = setup(
        &[0xB6, 0x12, 0x34], // LDA $1234
        0x0400,
    );
    mem.mem[0x1234] = 0xEE;
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0xEE);
}

// ---- CMPX ----

#[test]
fn cmpx_immediate() {
    let (mut cpu, mut mem) = setup(
        &[
            0x8E, 0x10, 0x00, // LDX #$1000
            0x8C, 0x10, 0x00, // CMPX #$1000
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert!(cpu.reg.cc.zero());
    assert_eq!(cpu.reg.x, 0x1000); // unchanged
}

// ---- Memory read-modify-write (INC direct) ----

#[test]
fn inc_direct() {
    let (mut cpu, mut mem) = setup(
        &[0x0C, 0x50], // INC <$50
        0x0400,
    );
    mem.mem[0x0050] = 0x41;
    cpu.step(&mut mem);
    assert_eq!(mem.mem[0x0050], 0x42);
}

// ===========================================================================
// Undocumented opcodes
// ===========================================================================

// ---- XHCF: Halt and Catch Fire (0x14, 0x15, 0xCD) ----

#[test]
fn xhcf_0x14_halts_cpu() {
    let (mut cpu, mut mem) = setup(&[0x14], 0x0400);
    assert!(!cpu.halted);
    cpu.step(&mut mem);
    assert!(cpu.halted);
}

#[test]
fn xhcf_0xcd_halts_cpu() {
    let (mut cpu, mut mem) = setup(&[0xCD], 0x0400);
    assert!(!cpu.halted);
    cpu.step(&mut mem);
    assert!(cpu.halted);
}

#[test]
fn illegal_opcode_sets_flag_but_execution_continues() {
    let (mut cpu, mut mem) = setup(&[0x87, 0x12], 0x0400);

    let first_cycles = cpu.step(&mut mem);
    assert_eq!(first_cycles, 1);
    assert!(cpu.illegal);
    assert!(!cpu.halted);
    assert_eq!(cpu.reg.pc, 0x0401);

    let second_cycles = cpu.step(&mut mem);
    assert_eq!(second_cycles, 2);
    assert_eq!(cpu.reg.pc, 0x0402);
    assert!(!cpu.halted);
}

// ---- X18: undocumented flag rotate (0x18) ----

#[test]
fn x18_flag_transform() {
    // ORCC #$FF then X18. After fetch of 0x18 PC = 0x0403; post-byte is 0xFF.
    // With CC=0xFF and post=0xFF: post_cc=0xFF
    //   E' = post_cc & CC_F(0x40) != 0 → true
    //   F' = post_cc & CC_H(0x20) != 0 → true
    //   H' = post_cc & CC_I(0x10) != 0 → true
    //   I' = post_cc & CC_N(0x08) != 0 → true
    //   N' = post_cc & CC_Z(0x04) != 0 → true
    //   Z' = post_cc & CC_V(0x02) != 0 → true
    //   V' = (post_cc & CC_C(0x01)) | (post_cc & CC_Z(0x04)) != 0 → true
    //   C' = false
    // Result CC = 0xFE (all set except C).
    let (mut cpu, mut mem) = setup(
        &[
            0x1A, 0xFF, // ORCC #$FF → CC = 0xFF
            0x18, // X18 (1-byte; reads post from PC+1 = 0x0403)
            0xFF, // post byte
        ],
        0x0400,
    );
    cpu.step(&mut mem); // ORCC
    assert_eq!(cpu.reg.cc.to_byte(), 0xFF);
    cpu.step(&mut mem); // X18
    assert_eq!(cpu.reg.cc.to_byte(), 0xFE); // C cleared, all others set
    assert_eq!(cpu.reg.pc, 0x0403); // PC advanced past 0x18 only
}

// ---- XANDCC immediate (0x38) ----

#[test]
fn xandcc_undoc_0x38() {
    // Clears all flags like ANDCC; uses the undocumented opcode 0x38.
    let (mut cpu, mut mem) = setup(
        &[
            0x1A, 0xFF, // ORCC #$FF → all flags set
            0x38, 0x00, // XANDCC #$00 → all flags cleared
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.cc.to_byte(), 0xFF);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.cc.to_byte(), 0x00);
}

// ---- RESET undocumented (0x3E) ----

#[test]
fn reset_undoc_0x3e_jumps_and_preserves_fi() {
    // ANDCC #$AF clears F(0x40) and I(0x10) bits. Then RESET (0x3E) should
    // NOT set them (unlike software RESET would).
    let (mut cpu, mut mem) = setup(
        &[
            0x1C, 0xAF, // ANDCC #$AF → clear F and I
            0x3E, // RESET undoc
        ],
        0x0400,
    );
    cpu.reg.s = 0x8000;
    mem.mem[0xFFFE] = 0x10;
    mem.mem[0xFFFF] = 0x00; // RESET vector → 0x1000
    cpu.step(&mut mem); // ANDCC
    assert!(!cpu.reg.cc.firq_inhibit());
    assert!(!cpu.reg.cc.irq_inhibit());
    cpu.step(&mut mem); // RESET undoc
    assert_eq!(cpu.reg.pc, 0x1000);
    assert!(!cpu.reg.cc.firq_inhibit()); // F not set by instruction
    assert!(!cpu.reg.cc.irq_inhibit()); // I not set by instruction
    assert_eq!(cpu.reg.s, 0x8000 - 12); // full 12-byte frame pushed
    let saved_cc = mem.mem[cpu.reg.s as usize];
    assert_eq!(saved_cc & 0x80, 0x00); // E was NOT set before push
}

// ---- XCLRA: CLR A but C unchanged (0x4E) ----

#[test]
fn xclra_undoc_zeroes_a_leaves_carry() {
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0xFF, // LDA #$FF
            0x1A, 0x01, // ORCC #$01 → set C
            0x4E, // XCLRA
        ],
        0x0400,
    );
    cpu.step(&mut mem); // LDA #$FF
    cpu.step(&mut mem); // ORCC — set C
    cpu.step(&mut mem); // XCLRA
    assert_eq!(cpu.reg.a(), 0x00);
    assert!(cpu.reg.cc.zero());
    assert!(!cpu.reg.cc.negative());
    assert!(!cpu.reg.cc.overflow());
    assert!(cpu.reg.cc.carry()); // C must remain set
}

// ---- XCLRB: CLR B but C unchanged (0x5E) ----

#[test]
fn xclrb_undoc_zeroes_b_leaves_carry() {
    let (mut cpu, mut mem) = setup(
        &[
            0xC6, 0xAA, // LDB #$AA
            0x1A, 0x01, // ORCC — set C
            0x5E, // XCLRB
        ],
        0x0400,
    );
    cpu.step(&mut mem); // LDB
    cpu.step(&mut mem); // ORCC
    cpu.step(&mut mem); // XCLRB
    assert_eq!(cpu.reg.b(), 0x00);
    assert!(cpu.reg.cc.zero());
    assert!(!cpu.reg.cc.negative());
    assert!(!cpu.reg.cc.overflow());
    assert!(cpu.reg.cc.carry()); // C unchanged
}

// ---- TFR / EXG mixed-size undocumented behaviour ----

#[test]
fn tfr_mixed_size_gives_0xff() {
    // TFR B (8-bit, code=9) → X (16-bit, code=1): post-byte = 0x91
    // Because sizes differ, X receives 0xFFFF (undocumented result).
    let (mut cpu, mut mem) = setup(
        &[
            0xC6, 0x42, // LDB #$42
            0x1F, 0x91, // TFR B,X
        ],
        0x0400,
    );
    cpu.step(&mut mem); // LDB
    cpu.step(&mut mem); // TFR B,X
    assert_eq!(cpu.reg.x, 0xFFFF);
    assert_eq!(cpu.reg.b(), 0x42); // source unchanged
}

#[test]
fn tfr_mixed_size_16_to_8_gives_ff() {
    // TFR X (16-bit, code=1) → A (8-bit, code=8): post-byte = 0x18
    // A receives 0xFF.
    let (mut cpu, mut mem) = setup(
        &[
            0x8E, 0x12, 0x34, // LDX #$1234
            0x1F, 0x18, // TFR X,A
        ],
        0x0400,
    );
    cpu.step(&mut mem); // LDX
    cpu.step(&mut mem); // TFR X,A
    assert_eq!(cpu.reg.a(), 0xFF);
    assert_eq!(cpu.reg.x, 0x1234); // source unchanged
}

#[test]
fn exg_mixed_size_both_get_ff() {
    // EXG A (8-bit, code=8) ↔ X (16-bit, code=1): post-byte = 0x81
    // Both A and X receive 0xFF / 0xFFFF respectively.
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x55, // LDA #$55
            0x8E, 0x12, 0x34, // LDX #$1234
            0x1E, 0x81, // EXG A,X
        ],
        0x0400,
    );
    cpu.step(&mut mem); // LDA
    cpu.step(&mut mem); // LDX
    cpu.step(&mut mem); // EXG A,X
    assert_eq!(cpu.reg.a(), 0xFF);
    assert_eq!(cpu.reg.x, 0xFFFF);
}

// ---- Undocumented SWI2 (0x10 0x3E): does not set E before pushing ----

#[test]
fn swi2_undoc_page1_no_e_flag_in_frame() {
    let (mut cpu, mut mem) = setup(
        &[
            0x1C, 0x7F, // ANDCC #$7F → clear E
            0x10, 0x3E, // SWI2 undocumented
        ],
        0x0400,
    );
    cpu.reg.s = 0x8000;
    mem.mem[0xFFF4] = 0x20;
    mem.mem[0xFFF5] = 0x00; // SWI2 vector → 0x2000
    cpu.step(&mut mem); // ANDCC — E clear
    cpu.step(&mut mem); // SWI2 undoc
    assert_eq!(cpu.reg.pc, 0x2000);
    assert_eq!(cpu.reg.s, 0x8000 - 12);
    let saved_cc = mem.mem[cpu.reg.s as usize];
    assert_eq!(saved_cc & 0x80, 0x00); // E was NOT set before push
}

// ---- XFIRQ (0x11 0x3E): push state, jump via FIRQ vector, no flag changes ----

#[test]
fn xfirq_undoc_jumps_via_firq_vector_no_flag_changes() {
    let (mut cpu, mut mem) = setup(
        &[
            0x1C, 0xAF, // ANDCC #$AF → clear F and I
            0x11, 0x3E, // XFIRQ
        ],
        0x0400,
    );
    cpu.reg.s = 0x8000;
    mem.mem[0xFFF6] = 0x30;
    mem.mem[0xFFF7] = 0x00; // FIRQ vector → 0x3000
    cpu.step(&mut mem); // ANDCC
    assert!(!cpu.reg.cc.firq_inhibit());
    assert!(!cpu.reg.cc.irq_inhibit());
    cpu.step(&mut mem); // XFIRQ
    assert_eq!(cpu.reg.pc, 0x3000);
    assert!(!cpu.reg.cc.firq_inhibit()); // F still clear
    assert!(!cpu.reg.cc.irq_inhibit()); // I still clear
    assert_eq!(cpu.reg.s, 0x8000 - 12); // full state pushed
}

// ---- XNC: NEG if C=0, COM if C=1 ----

#[test]
fn xnc_a_carry_clear_acts_as_nega() {
    // 0x42: XNC A — C=0 → behaves like NEGA
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x05, // LDA #$05
            0x1C, 0xFE, // ANDCC #$FE → clear C
            0x42, // XNC A
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0xFB); // neg(0x05) = 0xFB (two's complement)
    assert!(cpu.reg.cc.carry()); // neg of non-zero sets C
}

#[test]
fn xnc_a_carry_set_acts_as_coma() {
    // 0x42: XNC A — C=1 → behaves like COMA
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0xAA, // LDA #$AA
            0x1A, 0x01, // ORCC #$01 → set C
            0x42, // XNC A
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x55); // ~$AA = $55
    assert!(cpu.reg.cc.carry()); // COM always sets C
}

#[test]
fn xnc_b_carry_clear_acts_as_negb() {
    // 0x52: XNC B — C=0 → behaves like NEGB
    let (mut cpu, mut mem) = setup(
        &[
            0xC6, 0x01, // LDB #$01
            0x1C, 0xFE, // ANDCC #$FE → clear C
            0x52, // XNC B
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.b(), 0xFF);
}

#[test]
fn xnc_direct_carry_set_acts_as_com() {
    // 0x02: XNC direct — C=1 → behaves like COM
    let (mut cpu, mut mem) = setup(
        &[
            0x1A, 0x01, // ORCC #$01 → set C
            0x02, 0x50, // XNC <$50
        ],
        0x0400,
    );
    mem.mem[0x0050] = 0x0F;
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(mem.mem[0x0050], 0xF0); // ~0x0F = 0xF0
}

// ---- XDEC: DEC but C reflects (original != 0) ----

#[test]
fn xdec_a_nonzero_sets_carry() {
    // 0x4B: XDEC A — operand=3 → result=2, C=1 (3 != 0)
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x03, // LDA #$03
            0x1C, 0xFE, // ANDCC #$FE → clear C
            0x4B, // XDEC A
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0x02);
    assert!(cpu.reg.cc.carry()); // 3 != 0 → C set
}

#[test]
fn xdec_a_zero_clears_carry() {
    // 0x4B: XDEC A — operand=0 → result=0xFF, C=0 (0 == 0)
    let (mut cpu, mut mem) = setup(
        &[
            0x86, 0x00, // LDA #$00
            0x1A, 0x01, // ORCC #$01 → set C (to verify it gets cleared)
            0x4B, // XDEC A
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.a(), 0xFF);
    assert!(!cpu.reg.cc.carry()); // 0 == 0 → C cleared
    assert!(cpu.reg.cc.negative()); // 0xFF is negative
}

#[test]
fn xdec_b_nonzero_sets_carry() {
    // 0x5B: XDEC B — operand=0x80 → result=0x7F, V set, C set (0x80 != 0)
    let (mut cpu, mut mem) = setup(
        &[
            0xC6, 0x80, // LDB #$80
            0x1C, 0xFE, // ANDCC #$FE → clear C
            0x5B, // XDEC B
        ],
        0x0400,
    );
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(cpu.reg.b(), 0x7F);
    assert!(cpu.reg.cc.overflow()); // 0x80 → V set (same as DEC)
    assert!(cpu.reg.cc.carry()); // 0x80 != 0 → C set
}

#[test]
fn xdec_direct_zero_clears_carry() {
    // 0x0B: XDEC direct — operand=0 → carry cleared
    let (mut cpu, mut mem) = setup(
        &[
            0x1A, 0x01, // ORCC #$01 → set C
            0x0B, 0x50, // XDEC <$50
        ],
        0x0400,
    );
    mem.mem[0x0050] = 0x00;
    cpu.step(&mut mem);
    cpu.step(&mut mem);
    assert_eq!(mem.mem[0x0050], 0xFF);
    assert!(!cpu.reg.cc.carry()); // 0 == 0 → C cleared
}

// ---- XADDD (page 1, 0x10 0xC3 / 0xD3): sets flags like ADDD, result discarded ----

#[test]
fn xaddd_imm_sets_flags_discards_result() {
    // XADDD #$0001 with D=$FFFF should set C and Z (wraps to 0x0000).
    let (mut cpu, mut mem) = setup(
        &[
            0xCC, 0xFF, 0xFF, // LDD #$FFFF
            0x10, 0xC3, 0x00, 0x01, // XADDD #$0001
        ],
        0x0400,
    );
    cpu.step(&mut mem); // LDD
    cpu.step(&mut mem); // XADDD imm
    assert_eq!(cpu.reg.d, 0xFFFF); // D unchanged
    assert!(cpu.reg.cc.carry()); // overflow into carry
    assert!(cpu.reg.cc.zero()); // result would be 0
}

#[test]
fn xaddd_direct_sets_flags_discards_result() {
    // XADDD <$50 with D=$1000 and mem[$50]=$0234 → result 0x1234, no C/Z.
    let (mut cpu, mut mem) = setup(
        &[
            0xCC, 0x10, 0x00, // LDD #$1000
            0x10, 0xD3, 0x50, // XADDD <$50
        ],
        0x0400,
    );
    mem.mem[0x0050] = 0x02;
    mem.mem[0x0051] = 0x34;
    cpu.step(&mut mem); // LDD
    cpu.step(&mut mem); // XADDD direct
    assert_eq!(cpu.reg.d, 0x1000); // D unchanged
    assert!(!cpu.reg.cc.carry());
    assert!(!cpu.reg.cc.zero());
    assert!(!cpu.reg.cc.negative()); // 0x1234 is positive
}

// ---- XADDU (page 2, 0x11 0xC3 / 0xD3): adds (U | 0xFF00) with operand ----

#[test]
fn xaddu_imm_sets_flags_discards_result() {
    // U=$0000, so (U | 0xFF00) = 0xFF00. XADDU #$0100 → 0xFF00 + 0x0100 = 0x0000 + carry.
    let (mut cpu, mut mem) = setup(
        &[
            0xCE, 0x00, 0x00, // LDU #$0000
            0x11, 0xC3, 0x01, 0x00, // XADDU #$0100
        ],
        0x0400,
    );
    cpu.reg.s = 0x8100; // arm NMI safely
    cpu.step(&mut mem); // LDU
    cpu.step(&mut mem); // XADDU imm
    assert_eq!(cpu.reg.u, 0x0000); // U unchanged
    assert!(cpu.reg.cc.carry()); // 0xFF00 + 0x0100 overflows
    assert!(cpu.reg.cc.zero()); // result is 0x0000
}

#[test]
fn xaddu_direct_sets_flags_discards_result() {
    // U=$00FF → (U | 0xFF00) = 0xFFFF. XADDU <$50 = $0001 → 0xFFFF + 0x0001 = 0x0000 + C.
    let (mut cpu, mut mem) = setup(
        &[
            0xCE, 0x00, 0xFF, // LDU #$00FF
            0x11, 0xD3, 0x50, // XADDU <$50
        ],
        0x0400,
    );
    cpu.reg.s = 0x8100;
    mem.mem[0x0050] = 0x00;
    mem.mem[0x0051] = 0x01;
    cpu.step(&mut mem); // LDU
    cpu.step(&mut mem); // XADDU direct
    assert_eq!(cpu.reg.u, 0x00FF); // U unchanged
    assert!(cpu.reg.cc.carry());
    assert!(cpu.reg.cc.zero());
}
