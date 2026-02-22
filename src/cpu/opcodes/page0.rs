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

//! Page 0 opcode implementations (0x00..0xFF, excluding 0x10/0x11 page prefixes).

use crate::alu;
use crate::bus::Bus;
use crate::cpu::Cpu;

/// Base cycle counts for Page 0 opcodes (0x00..0xFF).
/// Indexed-mode entries show the *base* cycles; extra cycles from the
/// post-byte are added separately.
#[rustfmt::skip]
const PAGE0_CYCLES: [u8; 256] = [
//  0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    6,  1,  1,  6,  6,  1,  6,  6,  6,  6,  6,  1,  6,  6,  3,  6, // 0x
    1,  1,  2,  2,  1,  1,  5,  9,  1,  2,  3,  1,  3,  2,  8,  7, // 1x (10,11 = page prefix)
    3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3, // 2x
    4,  4,  4,  4,  5,  5,  5,  5,  1,  5,  3,  6, 21, 11,  1, 19, // 3x
    2,  1,  1,  2,  2,  1,  2,  2,  2,  2,  2,  1,  2,  2,  1,  2, // 4x
    2,  1,  1,  2,  2,  1,  2,  2,  2,  2,  2,  1,  2,  2,  1,  2, // 5x
    6,  1,  1,  6,  6,  1,  6,  6,  6,  6,  6,  1,  6,  6,  3,  6, // 6x
    7,  1,  1,  7,  7,  1,  7,  7,  7,  7,  7,  1,  7,  7,  4,  7, // 7x
    2,  2,  2,  4,  2,  2,  2,  1,  2,  2,  2,  2,  4,  7,  3,  1, // 8x
    4,  4,  4,  6,  4,  4,  4,  4,  4,  4,  4,  4,  6,  7,  5,  5, // 9x
    4,  4,  4,  6,  4,  4,  4,  4,  4,  4,  4,  4,  6,  7,  5,  5, // Ax
    5,  5,  5,  7,  5,  5,  5,  5,  5,  5,  5,  5,  7,  8,  6,  6, // Bx
    2,  2,  2,  4,  2,  2,  2,  1,  2,  2,  2,  2,  3,  1,  3,  1, // Cx
    4,  4,  4,  6,  4,  4,  4,  4,  4,  4,  4,  4,  5,  5,  5,  5, // Dx
    4,  4,  4,  6,  4,  4,  4,  4,  4,  4,  4,  4,  5,  5,  5,  5, // Ex
    5,  5,  5,  7,  5,  5,  5,  5,  5,  5,  5,  5,  6,  6,  6,  6, // Fx
];

pub fn execute(cpu: &mut Cpu, bus: &mut impl Bus, opcode: u8) {
    cpu.cycles += PAGE0_CYCLES[opcode as usize] as u64;

    match opcode {
        // =================================================================
        // 0x00..0x0F — Direct-page read-modify-write + JMP/CLR
        // =================================================================
        0x00 | 0x01 => {
            // NEG direct (0x00) and (0x01, undoc)
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            let r = alu::neg8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x03 => {
            // COM direct
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            let r = alu::com8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x04 | 0x05 => {
            // LSR direct (0x04) and (0x05, undoc)
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            let r = alu::lsr8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x06 => {
            // ROR direct
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            let r = alu::ror8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x07 => {
            // ASR direct
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            let r = alu::asr8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x08 => {
            // ASL/LSL direct
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            let r = alu::asl8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x09 => {
            // ROL direct
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            let r = alu::rol8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x0A => {
            // DEC direct
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            let r = alu::dec8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x0C => {
            // INC direct
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            let r = alu::inc8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x0D => {
            // TST direct
            let addr = cpu.addr_direct(bus);
            let val = bus.read(addr);
            alu::tst8(val, &mut cpu.reg.cc);
        }
        0x0E => {
            // JMP direct
            cpu.reg.pc = cpu.addr_direct(bus);
        }
        0x0F => {
            // CLR direct
            let addr = cpu.addr_direct(bus);
            let r = alu::clr8(&mut cpu.reg.cc);
            bus.write(addr, r);
        }

        // =================================================================
        // 0x12..0x1F — Inherent / misc
        // =================================================================
        0x12 => {} // NOP
        0x13 => {
            // SYNC
            cpu.sync = true;
        }
        0x16 => {
            // LBRA
            let addr = cpu.addr_relative16(bus);
            cpu.reg.pc = addr;
        }
        0x17 => {
            // LBSR
            let addr = cpu.addr_relative16(bus);
            cpu.push_word_s(bus, cpu.reg.pc);
            cpu.reg.pc = addr;
        }
        0x19 => {
            // DAA
            let a = cpu.reg.a();
            let r = alu::daa(a, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x1A => {
            // ORCC immediate
            let val = cpu.fetch_byte(bus);
            cpu.reg.cc.or_with(val);
        }
        0x1C => {
            // ANDCC immediate
            let val = cpu.fetch_byte(bus);
            cpu.reg.cc.and_with(val);
        }
        0x1D => {
            // SEX
            let b = cpu.reg.b();
            let d = alu::sex(b, &mut cpu.reg.cc);
            cpu.reg.d = d;
        }
        0x1E => {
            // EXG
            let post = cpu.fetch_byte(bus);
            exg(cpu, post);
        }
        0x1F => {
            // TFR
            let post = cpu.fetch_byte(bus);
            tfr(cpu, post);
        }

        // =================================================================
        // 0x20..0x2F — Short branches
        // =================================================================
        0x20 => {
            // BRA
            let addr = cpu.addr_relative8(bus);
            cpu.reg.pc = addr;
        }
        0x21 => {
            // BRN
            let _addr = cpu.addr_relative8(bus);
            // never branch
        }
        0x22 => {
            // BHI: !(C|Z)
            let addr = cpu.addr_relative8(bus);
            if !cpu.reg.cc.carry() && !cpu.reg.cc.zero() {
                cpu.reg.pc = addr;
            }
        }
        0x23 => {
            // BLS: C|Z
            let addr = cpu.addr_relative8(bus);
            if cpu.reg.cc.carry() || cpu.reg.cc.zero() {
                cpu.reg.pc = addr;
            }
        }
        0x24 => {
            // BHS/BCC: !C
            let addr = cpu.addr_relative8(bus);
            if !cpu.reg.cc.carry() {
                cpu.reg.pc = addr;
            }
        }
        0x25 => {
            // BLO/BCS: C
            let addr = cpu.addr_relative8(bus);
            if cpu.reg.cc.carry() {
                cpu.reg.pc = addr;
            }
        }
        0x26 => {
            // BNE: !Z
            let addr = cpu.addr_relative8(bus);
            if !cpu.reg.cc.zero() {
                cpu.reg.pc = addr;
            }
        }
        0x27 => {
            // BEQ: Z
            let addr = cpu.addr_relative8(bus);
            if cpu.reg.cc.zero() {
                cpu.reg.pc = addr;
            }
        }
        0x28 => {
            // BVC: !V
            let addr = cpu.addr_relative8(bus);
            if !cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
            }
        }
        0x29 => {
            // BVS: V
            let addr = cpu.addr_relative8(bus);
            if cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
            }
        }
        0x2A => {
            // BPL: !N
            let addr = cpu.addr_relative8(bus);
            if !cpu.reg.cc.negative() {
                cpu.reg.pc = addr;
            }
        }
        0x2B => {
            // BMI: N
            let addr = cpu.addr_relative8(bus);
            if cpu.reg.cc.negative() {
                cpu.reg.pc = addr;
            }
        }
        0x2C => {
            // BGE: N==V  (N*V + !N*!V)
            let addr = cpu.addr_relative8(bus);
            if cpu.reg.cc.negative() == cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
            }
        }
        0x2D => {
            // BLT: N!=V  (N*!V + !N*V)
            let addr = cpu.addr_relative8(bus);
            if cpu.reg.cc.negative() != cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
            }
        }
        0x2E => {
            // BGT: !Z && N==V
            let addr = cpu.addr_relative8(bus);
            if !cpu.reg.cc.zero() && cpu.reg.cc.negative() == cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
            }
        }
        0x2F => {
            // BLE: Z || N!=V
            let addr = cpu.addr_relative8(bus);
            if cpu.reg.cc.zero() || cpu.reg.cc.negative() != cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
            }
        }

        // =================================================================
        // 0x30..0x3F — LEA, stack, misc inherent
        // =================================================================
        0x30 => {
            // LEAX indexed
            let (ea, extra) = cpu.addr_indexed(bus);
            cpu.reg.x = ea;
            cpu.reg.cc.set_zero(ea == 0);
            cpu.cycles += extra as u64;
        }
        0x31 => {
            // LEAY indexed
            let (ea, extra) = cpu.addr_indexed(bus);
            cpu.reg.y = ea;
            cpu.reg.cc.set_zero(ea == 0);
            cpu.cycles += extra as u64;
        }
        0x32 => {
            // LEAS indexed
            let (ea, extra) = cpu.addr_indexed(bus);
            cpu.reg.s = ea;
            cpu.arm_nmi();
            cpu.cycles += extra as u64;
        }
        0x33 => {
            // LEAU indexed
            let (ea, extra) = cpu.addr_indexed(bus);
            cpu.reg.u = ea;
            cpu.cycles += extra as u64;
        }
        0x34 => {
            // PSHS
            let post = cpu.fetch_byte(bus);
            pshs(cpu, bus, post);
        }
        0x35 => {
            // PULS
            let post = cpu.fetch_byte(bus);
            puls(cpu, bus, post);
        }
        0x36 => {
            // PSHU
            let post = cpu.fetch_byte(bus);
            pshu(cpu, bus, post);
        }
        0x37 => {
            // PULU
            let post = cpu.fetch_byte(bus);
            pulu(cpu, bus, post);
        }
        0x39 => {
            // RTS
            cpu.reg.pc = cpu.pull_word_s(bus);
        }
        0x3A => {
            // ABX: X = X + B (unsigned)
            cpu.reg.x = cpu.reg.x.wrapping_add(cpu.reg.b() as u16);
        }
        0x3B => {
            // RTI
            let cc = cpu.pull_byte_s(bus);
            cpu.reg.cc = crate::registers::ConditionCodes::from_byte(cc);
            if cpu.reg.cc.entire() {
                // Full restore: 15 cycles total
                let a = cpu.pull_byte_s(bus);
                cpu.reg.set_a(a);
                let b = cpu.pull_byte_s(bus);
                cpu.reg.set_b(b);
                cpu.reg.dp = cpu.pull_byte_s(bus);
                cpu.reg.x = cpu.pull_word_s(bus);
                cpu.reg.y = cpu.pull_word_s(bus);
                cpu.reg.u = cpu.pull_word_s(bus);
                cpu.cycles += 9; // 6 base + 9 extra = 15
            }
            cpu.reg.pc = cpu.pull_word_s(bus);
        }
        0x3C => {
            // CWAI
            let post = cpu.fetch_byte(bus);
            cpu.reg.cc.and_with(post);
            cpu.reg.cc.set_entire(true);
            cpu.push_entire_state(bus);
            cpu.cwai = true;
        }
        0x3D => {
            // MUL
            let a = cpu.reg.a();
            let b = cpu.reg.b();
            let d = alu::mul(a, b, &mut cpu.reg.cc);
            cpu.reg.d = d;
        }
        0x3E => {
            // RESET (undocumented)
            cpu.halted = true;
        }
        0x3F => {
            // SWI
            cpu.reg.cc.set_entire(true);
            cpu.push_entire_state(bus);
            cpu.reg.cc.set_irq_inhibit(true);
            cpu.reg.cc.set_firq_inhibit(true);
            cpu.reg.pc = bus.read_word(crate::cpu::VEC_SWI);
        }

        // =================================================================
        // 0x40..0x4F — Inherent A
        // =================================================================
        0x40 | 0x41 => {
            // NEGA (0x40) and (0x41, undoc)
            let v = cpu.reg.a();
            let r = alu::neg8(v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x43 => {
            let v = cpu.reg.a();
            let r = alu::com8(v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x44 | 0x45 => {
            // LSRA (0x44) and (0x45, undoc)
            let v = cpu.reg.a();
            let r = alu::lsr8(v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x46 => {
            let v = cpu.reg.a();
            let r = alu::ror8(v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x47 => {
            let v = cpu.reg.a();
            let r = alu::asr8(v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x48 => {
            let v = cpu.reg.a();
            let r = alu::asl8(v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x49 => {
            let v = cpu.reg.a();
            let r = alu::rol8(v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x4A => {
            let v = cpu.reg.a();
            let r = alu::dec8(v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x4C => {
            let v = cpu.reg.a();
            let r = alu::inc8(v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x4D => {
            let v = cpu.reg.a();
            alu::tst8(v, &mut cpu.reg.cc);
        }
        0x4F => {
            // CLRA
            let r = alu::clr8(&mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }

        // =================================================================
        // 0x50..0x5F — Inherent B
        // =================================================================
        0x50 | 0x51 => {
            // NEGB (0x50) and (0x51, undoc)
            let v = cpu.reg.b();
            let r = alu::neg8(v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0x53 => {
            let v = cpu.reg.b();
            let r = alu::com8(v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0x54 | 0x55 => {
            // LSRB (0x54) and (0x55, undoc)
            let v = cpu.reg.b();
            let r = alu::lsr8(v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0x56 => {
            let v = cpu.reg.b();
            let r = alu::ror8(v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0x57 => {
            let v = cpu.reg.b();
            let r = alu::asr8(v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0x58 => {
            let v = cpu.reg.b();
            let r = alu::asl8(v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0x59 => {
            let v = cpu.reg.b();
            let r = alu::rol8(v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0x5A => {
            let v = cpu.reg.b();
            let r = alu::dec8(v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0x5C => {
            let v = cpu.reg.b();
            let r = alu::inc8(v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0x5D => {
            let v = cpu.reg.b();
            alu::tst8(v, &mut cpu.reg.cc);
        }
        0x5F => {
            // CLRB
            let r = alu::clr8(&mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }

        // =================================================================
        // 0x60..0x6F — Indexed read-modify-write
        // =================================================================
        0x60 | 0x61 => {
            // NEG indexed (0x60) and (0x61, undoc)
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            let r = alu::neg8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x63 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            let r = alu::com8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x64 | 0x65 => {
            // LSR indexed (0x64) and (0x65, undoc)
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            let r = alu::lsr8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x66 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            let r = alu::ror8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x67 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            let r = alu::asr8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x68 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            let r = alu::asl8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x69 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            let r = alu::rol8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x6A => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            let r = alu::dec8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x6C => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            let r = alu::inc8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x6D => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let val = bus.read(addr);
            alu::tst8(val, &mut cpu.reg.cc);
        }
        0x6E => {
            // JMP indexed
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            cpu.reg.pc = addr;
        }
        0x6F => {
            // CLR indexed
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let r = alu::clr8(&mut cpu.reg.cc);
            bus.write(addr, r);
        }

        // =================================================================
        // 0x70..0x7F — Extended read-modify-write
        // =================================================================
        0x70 | 0x71 => {
            // NEG extended (0x70) and (0x71, undoc)
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            let r = alu::neg8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x73 => {
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            let r = alu::com8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x74 | 0x75 => {
            // LSR extended (0x74) and (0x75, undoc)
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            let r = alu::lsr8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x76 => {
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            let r = alu::ror8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x77 => {
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            let r = alu::asr8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x78 => {
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            let r = alu::asl8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x79 => {
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            let r = alu::rol8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x7A => {
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            let r = alu::dec8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x7C => {
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            let r = alu::inc8(val, &mut cpu.reg.cc);
            bus.write(addr, r);
        }
        0x7D => {
            let addr = cpu.addr_extended(bus);
            let val = bus.read(addr);
            alu::tst8(val, &mut cpu.reg.cc);
        }
        0x7E => {
            // JMP extended
            cpu.reg.pc = cpu.addr_extended(bus);
        }
        0x7F => {
            // CLR
            let addr = cpu.addr_extended(bus);
            let r = alu::clr8(&mut cpu.reg.cc);
            bus.write(addr, r);
        }

        // =================================================================
        // 0x80..0x8F — Immediate A / D / X
        // =================================================================
        0x80 => {
            let v = cpu.fetch_byte(bus);
            let a = cpu.reg.a();
            let r = alu::sub8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x81 => {
            // CMPA immediate
            let v = cpu.fetch_byte(bus);
            let a = cpu.reg.a();
            alu::sub8(a, v, &mut cpu.reg.cc);
        }
        0x82 => {
            let v = cpu.fetch_byte(bus);
            let a = cpu.reg.a();
            let r = alu::sbc8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x83 => {
            // SUBD immediate
            let v = cpu.fetch_word(bus);
            let d = cpu.reg.d;
            let r = alu::sub16(d, v, &mut cpu.reg.cc);
            cpu.reg.d = r;
        }
        0x84 => {
            let v = cpu.fetch_byte(bus);
            let a = cpu.reg.a();
            let r = alu::and8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x85 => {
            // BITA immediate
            let v = cpu.fetch_byte(bus);
            let a = cpu.reg.a();
            alu::and8(a, v, &mut cpu.reg.cc);
        }
        0x86 => {
            // LDA immediate
            let v = cpu.fetch_byte(bus);
            alu::ld8_flags(v, &mut cpu.reg.cc);
            cpu.reg.set_a(v);
        }
        // 0x87 illegal
        0x88 => {
            let v = cpu.fetch_byte(bus);
            let a = cpu.reg.a();
            let r = alu::eor8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x89 => {
            let v = cpu.fetch_byte(bus);
            let a = cpu.reg.a();
            let r = alu::adc8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x8A => {
            let v = cpu.fetch_byte(bus);
            let a = cpu.reg.a();
            let r = alu::or8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x8B => {
            let v = cpu.fetch_byte(bus);
            let a = cpu.reg.a();
            let r = alu::add8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x8C => {
            // CMPX immediate
            let v = cpu.fetch_word(bus);
            let x = cpu.reg.x;
            alu::sub16(x, v, &mut cpu.reg.cc);
        }
        0x8D => {
            // BSR immediate
            let addr = cpu.addr_relative8(bus);
            cpu.push_word_s(bus, cpu.reg.pc);
            cpu.reg.pc = addr;
        }
        0x8E => {
            // LDX immediate
            let v = cpu.fetch_word(bus);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.x = v;
        }
        // 0x8F illegal

        // =================================================================
        // 0x90..0x9F — Direct A / D / X
        // =================================================================
        0x90 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::sub8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x91 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            alu::sub8(a, v, &mut cpu.reg.cc);
        }
        0x92 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::sbc8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x93 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read_word(addr);
            let d = cpu.reg.d;
            let r = alu::sub16(d, v, &mut cpu.reg.cc);
            cpu.reg.d = r;
        }
        0x94 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::and8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x95 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            alu::and8(a, v, &mut cpu.reg.cc);
        }
        0x96 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            alu::ld8_flags(v, &mut cpu.reg.cc);
            cpu.reg.set_a(v);
        }
        0x97 => {
            // STA direct
            let addr = cpu.addr_direct(bus);
            let v = cpu.reg.a();
            alu::ld8_flags(v, &mut cpu.reg.cc);
            bus.write(addr, v);
        }
        0x98 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::eor8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x99 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::adc8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x9A => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::or8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x9B => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::add8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0x9C => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read_word(addr);
            let x = cpu.reg.x;
            alu::sub16(x, v, &mut cpu.reg.cc);
        }
        0x9D => {
            // JSR direct
            let addr = cpu.addr_direct(bus);
            cpu.push_word_s(bus, cpu.reg.pc);
            cpu.reg.pc = addr;
        }
        0x9E => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.x = v;
        }
        0x9F => {
            let addr = cpu.addr_direct(bus);
            let v = cpu.reg.x;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            bus.write_word(addr, v);
        }

        // =================================================================
        // 0xA0..0xAF — Indexed A / D / X
        // =================================================================
        0xA0 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::sub8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xA1 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let a = cpu.reg.a();
            alu::sub8(a, v, &mut cpu.reg.cc);
        }
        0xA2 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::sbc8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xA3 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read_word(addr);
            let d = cpu.reg.d;
            let r = alu::sub16(d, v, &mut cpu.reg.cc);
            cpu.reg.d = r;
        }
        0xA4 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::and8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xA5 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let a = cpu.reg.a();
            alu::and8(a, v, &mut cpu.reg.cc);
        }
        0xA6 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            alu::ld8_flags(v, &mut cpu.reg.cc);
            cpu.reg.set_a(v);
        }
        0xA7 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = cpu.reg.a();
            alu::ld8_flags(v, &mut cpu.reg.cc);
            bus.write(addr, v);
        }
        0xA8 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::eor8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xA9 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::adc8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xAA => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::or8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xAB => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::add8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xAC => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read_word(addr);
            let x = cpu.reg.x;
            alu::sub16(x, v, &mut cpu.reg.cc);
        }
        0xAD => {
            // JSR indexed
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            cpu.push_word_s(bus, cpu.reg.pc);
            cpu.reg.pc = addr;
        }
        0xAE => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.x = v;
        }
        0xAF => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = cpu.reg.x;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            bus.write_word(addr, v);
        }

        // =================================================================
        // 0xB0..0xBF — Extended A / D / X
        // =================================================================
        0xB0 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::sub8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xB1 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            alu::sub8(a, v, &mut cpu.reg.cc);
        }
        0xB2 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::sbc8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xB3 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read_word(addr);
            let d = cpu.reg.d;
            let r = alu::sub16(d, v, &mut cpu.reg.cc);
            cpu.reg.d = r;
        }
        0xB4 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::and8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xB5 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            alu::and8(a, v, &mut cpu.reg.cc);
        }
        0xB6 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            alu::ld8_flags(v, &mut cpu.reg.cc);
            cpu.reg.set_a(v);
        }
        0xB7 => {
            let addr = cpu.addr_extended(bus);
            let v = cpu.reg.a();
            alu::ld8_flags(v, &mut cpu.reg.cc);
            bus.write(addr, v);
        }
        0xB8 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::eor8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xB9 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::adc8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xBA => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::or8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xBB => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let a = cpu.reg.a();
            let r = alu::add8(a, v, &mut cpu.reg.cc);
            cpu.reg.set_a(r);
        }
        0xBC => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read_word(addr);
            let x = cpu.reg.x;
            alu::sub16(x, v, &mut cpu.reg.cc);
        }
        0xBD => {
            // JSR extended
            let addr = cpu.addr_extended(bus);
            cpu.push_word_s(bus, cpu.reg.pc);
            cpu.reg.pc = addr;
        }
        0xBE => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.x = v;
        }
        0xBF => {
            let addr = cpu.addr_extended(bus);
            let v = cpu.reg.x;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            bus.write_word(addr, v);
        }

        // =================================================================
        // 0xC0..0xCF — Immediate B / D / U
        // =================================================================
        0xC0 => {
            let v = cpu.fetch_byte(bus);
            let b = cpu.reg.b();
            let r = alu::sub8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xC1 => {
            let v = cpu.fetch_byte(bus);
            let b = cpu.reg.b();
            alu::sub8(b, v, &mut cpu.reg.cc);
        }
        0xC2 => {
            let v = cpu.fetch_byte(bus);
            let b = cpu.reg.b();
            let r = alu::sbc8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xC3 => {
            let v = cpu.fetch_word(bus);
            let d = cpu.reg.d;
            let r = alu::add16(d, v, &mut cpu.reg.cc);
            cpu.reg.d = r;
        } // ADDD
        0xC4 => {
            let v = cpu.fetch_byte(bus);
            let b = cpu.reg.b();
            let r = alu::and8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xC5 => {
            let v = cpu.fetch_byte(bus);
            let b = cpu.reg.b();
            alu::and8(b, v, &mut cpu.reg.cc);
        }
        0xC6 => {
            let v = cpu.fetch_byte(bus);
            alu::ld8_flags(v, &mut cpu.reg.cc);
            cpu.reg.set_b(v);
        }
        // 0xC7 illegal
        0xC8 => {
            let v = cpu.fetch_byte(bus);
            let b = cpu.reg.b();
            let r = alu::eor8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xC9 => {
            let v = cpu.fetch_byte(bus);
            let b = cpu.reg.b();
            let r = alu::adc8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xCA => {
            let v = cpu.fetch_byte(bus);
            let b = cpu.reg.b();
            let r = alu::or8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xCB => {
            let v = cpu.fetch_byte(bus);
            let b = cpu.reg.b();
            let r = alu::add8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xCC => {
            // LDD immediate
            let v = cpu.fetch_word(bus);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.d = v;
        }
        // 0xCD illegal
        0xCE => {
            let v = cpu.fetch_word(bus);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.u = v;
        } // LDU
        // 0xCF illegal

        // =================================================================
        // 0xD0..0xDF — Direct B / D / U
        // =================================================================
        0xD0 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::sub8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xD1 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            alu::sub8(b, v, &mut cpu.reg.cc);
        }
        0xD2 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::sbc8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xD3 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read_word(addr);
            let d = cpu.reg.d;
            let r = alu::add16(d, v, &mut cpu.reg.cc);
            cpu.reg.d = r;
        }
        0xD4 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::and8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xD5 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            alu::and8(b, v, &mut cpu.reg.cc);
        }
        0xD6 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            alu::ld8_flags(v, &mut cpu.reg.cc);
            cpu.reg.set_b(v);
        }
        0xD7 => {
            let addr = cpu.addr_direct(bus);
            let v = cpu.reg.b();
            alu::ld8_flags(v, &mut cpu.reg.cc);
            bus.write(addr, v);
        }
        0xD8 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::eor8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xD9 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::adc8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xDA => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::or8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xDB => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::add8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xDC => {
            // LDD direct
            let addr = cpu.addr_direct(bus);
            let v = bus.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.d = v;
        }
        0xDD => {
            let addr = cpu.addr_direct(bus);
            let v = cpu.reg.d;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            bus.write_word(addr, v);
        }
        0xDE => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.u = v;
        }
        0xDF => {
            let addr = cpu.addr_direct(bus);
            let v = cpu.reg.u;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            bus.write_word(addr, v);
        }

        // =================================================================
        // 0xE0..0xEF — Indexed B / D / U
        // =================================================================
        0xE0 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::sub8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xE1 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let b = cpu.reg.b();
            alu::sub8(b, v, &mut cpu.reg.cc);
        }
        0xE2 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::sbc8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xE3 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read_word(addr);
            let d = cpu.reg.d;
            let r = alu::add16(d, v, &mut cpu.reg.cc);
            cpu.reg.d = r;
        }
        0xE4 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::and8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xE5 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let b = cpu.reg.b();
            alu::and8(b, v, &mut cpu.reg.cc);
        }
        0xE6 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            alu::ld8_flags(v, &mut cpu.reg.cc);
            cpu.reg.set_b(v);
        }
        0xE7 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = cpu.reg.b();
            alu::ld8_flags(v, &mut cpu.reg.cc);
            bus.write(addr, v);
        }
        0xE8 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::eor8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xE9 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::adc8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xEA => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::or8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xEB => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::add8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xEC => {
            // LDD indexed
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.d = v;
        }
        0xED => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = cpu.reg.d;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            bus.write_word(addr, v);
        }
        0xEE => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.u = v;
        }
        0xEF => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = cpu.reg.u;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            bus.write_word(addr, v);
        }

        // =================================================================
        // 0xF0..0xFF — Extended B / D / U
        // =================================================================
        0xF0 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::sub8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xF1 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            alu::sub8(b, v, &mut cpu.reg.cc);
        }
        0xF2 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::sbc8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xF3 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read_word(addr);
            let d = cpu.reg.d;
            let r = alu::add16(d, v, &mut cpu.reg.cc);
            cpu.reg.d = r;
        }
        0xF4 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::and8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xF5 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            alu::and8(b, v, &mut cpu.reg.cc);
        }
        0xF6 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            alu::ld8_flags(v, &mut cpu.reg.cc);
            cpu.reg.set_b(v);
        }
        0xF7 => {
            let addr = cpu.addr_extended(bus);
            let v = cpu.reg.b();
            alu::ld8_flags(v, &mut cpu.reg.cc);
            bus.write(addr, v);
        }
        0xF8 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::eor8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xF9 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::adc8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xFA => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::or8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xFB => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read(addr);
            let b = cpu.reg.b();
            let r = alu::add8(b, v, &mut cpu.reg.cc);
            cpu.reg.set_b(r);
        }
        0xFC => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.d = v;
        }
        0xFD => {
            let addr = cpu.addr_extended(bus);
            let v = cpu.reg.d;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            bus.write_word(addr, v);
        }
        0xFE => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.u = v;
        }
        0xFF => {
            let addr = cpu.addr_extended(bus);
            let v = cpu.reg.u;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            bus.write_word(addr, v);
        }

        // Illegal / undefined opcodes — treat as NOP (1 cycle already added)
        _ => {
            //debug!("Illegal opcode: {:02X}", opcode);
            cpu.illegal = true;
        }
    }
}

// ---------------------------------------------------------------------------
// PSHS / PULS / PSHU / PULU
// ---------------------------------------------------------------------------

/// PSHS: push selected registers onto S. Each byte pushed adds 1 cycle.
fn pshs(cpu: &mut Cpu, bus: &mut impl Bus, post: u8) {
    // Push order: PC, U, Y, X, DP, B, A, CC (highest bit first)
    if post & 0x80 != 0 {
        cpu.push_word_s(bus, cpu.reg.pc);
        cpu.cycles += 2;
    }
    if post & 0x40 != 0 {
        cpu.push_word_s(bus, cpu.reg.u);
        cpu.cycles += 2;
    }
    if post & 0x20 != 0 {
        cpu.push_word_s(bus, cpu.reg.y);
        cpu.cycles += 2;
    }
    if post & 0x10 != 0 {
        cpu.push_word_s(bus, cpu.reg.x);
        cpu.cycles += 2;
    }
    if post & 0x08 != 0 {
        cpu.push_byte_s(bus, cpu.reg.dp);
        cpu.cycles += 1;
    }
    if post & 0x04 != 0 {
        cpu.push_byte_s(bus, cpu.reg.b());
        cpu.cycles += 1;
    }
    if post & 0x02 != 0 {
        cpu.push_byte_s(bus, cpu.reg.a());
        cpu.cycles += 1;
    }
    if post & 0x01 != 0 {
        cpu.push_byte_s(bus, cpu.reg.cc.to_byte());
        cpu.cycles += 1;
    }
}

/// PULS: pull selected registers from S. Each byte pulled adds 1 cycle.
fn puls(cpu: &mut Cpu, bus: &mut impl Bus, post: u8) {
    // Pull order: CC, A, B, DP, X, Y, U, PC (lowest bit first)
    if post & 0x01 != 0 {
        let v = cpu.pull_byte_s(bus);
        cpu.reg.cc = crate::registers::ConditionCodes::from_byte(v);
        cpu.cycles += 1;
    }
    if post & 0x02 != 0 {
        let v = cpu.pull_byte_s(bus);
        cpu.reg.set_a(v);
        cpu.cycles += 1;
    }
    if post & 0x04 != 0 {
        let v = cpu.pull_byte_s(bus);
        cpu.reg.set_b(v);
        cpu.cycles += 1;
    }
    if post & 0x08 != 0 {
        cpu.reg.dp = cpu.pull_byte_s(bus);
        cpu.cycles += 1;
    }
    if post & 0x10 != 0 {
        cpu.reg.x = cpu.pull_word_s(bus);
        cpu.cycles += 2;
    }
    if post & 0x20 != 0 {
        cpu.reg.y = cpu.pull_word_s(bus);
        cpu.cycles += 2;
    }
    if post & 0x40 != 0 {
        cpu.reg.u = cpu.pull_word_s(bus);
        cpu.cycles += 2;
    }
    if post & 0x80 != 0 {
        cpu.reg.pc = cpu.pull_word_s(bus);
        cpu.cycles += 2;
    }
}

/// PSHU: push selected registers onto U.
fn pshu(cpu: &mut Cpu, bus: &mut impl Bus, post: u8) {
    if post & 0x80 != 0 {
        cpu.push_word_u(bus, cpu.reg.pc);
        cpu.cycles += 2;
    }
    if post & 0x40 != 0 {
        cpu.push_word_u(bus, cpu.reg.s);
        cpu.cycles += 2;
    } // S instead of U
    if post & 0x20 != 0 {
        cpu.push_word_u(bus, cpu.reg.y);
        cpu.cycles += 2;
    }
    if post & 0x10 != 0 {
        cpu.push_word_u(bus, cpu.reg.x);
        cpu.cycles += 2;
    }
    if post & 0x08 != 0 {
        cpu.push_byte_u(bus, cpu.reg.dp);
        cpu.cycles += 1;
    }
    if post & 0x04 != 0 {
        cpu.push_byte_u(bus, cpu.reg.b());
        cpu.cycles += 1;
    }
    if post & 0x02 != 0 {
        cpu.push_byte_u(bus, cpu.reg.a());
        cpu.cycles += 1;
    }
    if post & 0x01 != 0 {
        cpu.push_byte_u(bus, cpu.reg.cc.to_byte());
        cpu.cycles += 1;
    }
}

/// PULU: pull selected registers from U.
fn pulu(cpu: &mut Cpu, bus: &mut impl Bus, post: u8) {
    if post & 0x01 != 0 {
        let v = cpu.pull_byte_u(bus);
        cpu.reg.cc = crate::registers::ConditionCodes::from_byte(v);
        cpu.cycles += 1;
    }
    if post & 0x02 != 0 {
        let v = cpu.pull_byte_u(bus);
        cpu.reg.set_a(v);
        cpu.cycles += 1;
    }
    if post & 0x04 != 0 {
        let v = cpu.pull_byte_u(bus);
        cpu.reg.set_b(v);
        cpu.cycles += 1;
    }
    if post & 0x08 != 0 {
        cpu.reg.dp = cpu.pull_byte_u(bus);
        cpu.cycles += 1;
    }
    if post & 0x10 != 0 {
        cpu.reg.x = cpu.pull_word_u(bus);
        cpu.cycles += 2;
    }
    if post & 0x20 != 0 {
        cpu.reg.y = cpu.pull_word_u(bus);
        cpu.cycles += 2;
    }
    if post & 0x40 != 0 {
        cpu.reg.s = cpu.pull_word_u(bus);
        cpu.arm_nmi();
        cpu.cycles += 2;
    } // S instead of U
    if post & 0x80 != 0 {
        cpu.reg.pc = cpu.pull_word_u(bus);
        cpu.cycles += 2;
    }
}

// ---------------------------------------------------------------------------
// TFR / EXG
// ---------------------------------------------------------------------------

/// Read a register identified by a 4-bit code (from TFR/EXG post-byte).
/// Returns (value, is_16bit).
fn read_reg(cpu: &Cpu, code: u8) -> (u16, bool) {
    match code {
        0x0 => (cpu.reg.d, true),
        0x1 => (cpu.reg.x, true),
        0x2 => (cpu.reg.y, true),
        0x3 => (cpu.reg.u, true),
        0x4 => (cpu.reg.s, true),
        0x5 => (cpu.reg.pc, true),
        0x8 => (cpu.reg.a() as u16, false),
        0x9 => (cpu.reg.b() as u16, false),
        0xA => (cpu.reg.cc.to_byte() as u16, false),
        0xB => (cpu.reg.dp as u16, false),
        _ => (0xFF, false), // undefined → 0xFF
    }
}

/// Write a register identified by a 4-bit code.
fn write_reg(cpu: &mut Cpu, code: u8, val: u16) {
    match code {
        0x0 => cpu.reg.d = val,
        0x1 => cpu.reg.x = val,
        0x2 => cpu.reg.y = val,
        0x3 => cpu.reg.u = val,
        0x4 => {
            cpu.reg.s = val;
            cpu.arm_nmi();
        }
        0x5 => cpu.reg.pc = val,
        0x8 => cpu.reg.set_a(val as u8),
        0x9 => cpu.reg.set_b(val as u8),
        0xA => cpu.reg.cc = crate::registers::ConditionCodes::from_byte(val as u8),
        0xB => cpu.reg.dp = val as u8,
        _ => {} // undefined register — ignore
    }
}

/// TFR: transfer source → destination.
fn tfr(cpu: &mut Cpu, post: u8) {
    let src_code = (post >> 4) & 0x0F;
    let dst_code = post & 0x0F;
    let (src_val, src_16) = read_reg(cpu, src_code);
    let (_, dst_16) = read_reg(cpu, dst_code);

    let val = if src_16 != dst_16 {
        // Mixed 8/16-bit transfer → 0xFF (undocumented)
        if dst_16 { 0xFFFF } else { 0xFF }
    } else {
        src_val
    };
    write_reg(cpu, dst_code, val);
}

/// EXG: exchange source ↔ destination.
fn exg(cpu: &mut Cpu, post: u8) {
    let src_code = (post >> 4) & 0x0F;
    let dst_code = post & 0x0F;
    let (src_val, src_16) = read_reg(cpu, src_code);
    let (dst_val, dst_16) = read_reg(cpu, dst_code);

    if src_16 != dst_16 {
        // Mixed 8/16-bit exchange → both get 0xFF (undocumented)
        let sv = if src_16 { 0xFFFF } else { 0xFF };
        let dv = if dst_16 { 0xFFFF } else { 0xFF };
        write_reg(cpu, src_code, sv);
        write_reg(cpu, dst_code, dv);
    } else {
        write_reg(cpu, src_code, dst_val);
        write_reg(cpu, dst_code, src_val);
    }
}
