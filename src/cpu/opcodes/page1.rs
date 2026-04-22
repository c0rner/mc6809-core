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

//! Page 1 opcode implementations (prefix 0x10).
//!
//! Contains: long conditional branches, SWI2, CMPD, CMPY, LDY, STY, LDS, STS.
//! Contains all undocumented page 1 opcodes except all store immediate,
//! source: <https://github.com/hoglet67/6809Decoder/wiki/Undocumented-6809-Behaviours>

use crate::alu;
use crate::bus::Memory;
use crate::cpu::Cpu;

/// Base cycle counts for Page 1 opcodes. Only valid entries are non-zero.
#[rustfmt::skip]
const PAGE1_CYCLES: [u8; 256] = {
    let mut t = [0u8; 256];
    // Long branches: 5 cycles (not taken), 6 cycles (taken).
    // We charge 5 base and add 1 if taken.
    t[0x20] = 5; t[0x21] = 5; t[0x22] = 5; t[0x23] = 5;
    t[0x24] = 5; t[0x25] = 5; t[0x26] = 5; t[0x27] = 5;
    t[0x28] = 5; t[0x29] = 5; t[0x2A] = 5; t[0x2B] = 5;
    t[0x2C] = 5; t[0x2D] = 5; t[0x2E] = 5; t[0x2F] = 5;
    t[0x3E] = 20; // SWI2 (undocumented)
    t[0x3F] = 20; // SWI2
    t[0x83] = 5;  // CMPD imm
    t[0x8C] = 5;  // CMPY imm
    t[0x8E] = 4;  // LDY imm
    t[0x93] = 7;  // CMPD direct
    t[0x9C] = 7;  // CMPY direct
    t[0x9E] = 6;  // LDY direct
    t[0x9F] = 6;  // STY direct
    t[0xA3] = 7;  // CMPD indexed
    t[0xAC] = 7;  // CMPY indexed
    t[0xAE] = 6;  // LDY indexed
    t[0xAF] = 6;  // STY indexed
    t[0xB3] = 8;  // CMPD extended
    t[0xBC] = 8;  // CMPY extended
    t[0xBE] = 7;  // LDY extended
    t[0xBF] = 7;  // STY extended
    t[0xC3] = 5;  // XADDD imm (undocumented)
    t[0xCE] = 4;  // LDS imm
    t[0xD3] = 7;  // XADDD direct (undocumented)
    t[0xDE] = 6;  // LDS direct
    t[0xDF] = 6;  // STS direct
    t[0xE3] = 7;  // XADDD indexed (undocumented)
    t[0xEE] = 6;  // LDS indexed
    t[0xEF] = 6;  // STS indexed
    t[0xF3] = 8;  // XADDD extended (undocumented)
    t[0xFE] = 7;  // LDS extended
    t[0xFF] = 7;  // STS extended
    t
};

/// Returns the base cycle count for a Page 1 (0x10xx) sub-opcode.
pub(super) fn cycles(sub: u8) -> u8 {
    PAGE1_CYCLES[sub as usize]
}

pub fn execute(cpu: &mut Cpu, mem: &mut impl Memory, opcode: u8) {
    cpu.cycles += PAGE1_CYCLES[opcode as usize] as u64;

    match opcode {
        // =================================================================
        // Long conditional branches (16-bit relative offset)
        // =================================================================
        0x20 => {
            // XLBRA (same as LBRA with one extra cycle)
            let addr = cpu.addr_relative16(mem);
            cpu.reg.pc = addr;
        }
        0x21 => {
            // LBRN
            let _addr = cpu.addr_relative16(mem);
        }
        0x22 => {
            // LBHI
            let addr = cpu.addr_relative16(mem);
            if !cpu.reg.cc.carry() && !cpu.reg.cc.zero() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x23 => {
            // LBLS
            let addr = cpu.addr_relative16(mem);
            if cpu.reg.cc.carry() || cpu.reg.cc.zero() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x24 => {
            // LBHS/LBCC
            let addr = cpu.addr_relative16(mem);
            if !cpu.reg.cc.carry() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x25 => {
            // LBLO/LBCS
            let addr = cpu.addr_relative16(mem);
            if cpu.reg.cc.carry() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x26 => {
            // LBNE
            let addr = cpu.addr_relative16(mem);
            if !cpu.reg.cc.zero() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x27 => {
            // LBEQ
            let addr = cpu.addr_relative16(mem);
            if cpu.reg.cc.zero() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x28 => {
            // LBVC
            let addr = cpu.addr_relative16(mem);
            if !cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x29 => {
            // LBVS
            let addr = cpu.addr_relative16(mem);
            if cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x2A => {
            // LBPL
            let addr = cpu.addr_relative16(mem);
            if !cpu.reg.cc.negative() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x2B => {
            // LBMI
            let addr = cpu.addr_relative16(mem);
            if cpu.reg.cc.negative() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x2C => {
            // LBGE
            let addr = cpu.addr_relative16(mem);
            if cpu.reg.cc.negative() == cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x2D => {
            // LBLT
            let addr = cpu.addr_relative16(mem);
            if cpu.reg.cc.negative() != cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x2E => {
            // LBGT
            let addr = cpu.addr_relative16(mem);
            if !cpu.reg.cc.zero() && cpu.reg.cc.negative() == cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }
        0x2F => {
            // LBLE
            let addr = cpu.addr_relative16(mem);
            if cpu.reg.cc.zero() || cpu.reg.cc.negative() != cpu.reg.cc.overflow() {
                cpu.reg.pc = addr;
                cpu.cycles += 1;
            }
        }

        // =================================================================
        // SWI2
        // =================================================================
        0x3E => {
            // SWi2 (undocumented)
            // Does not set E, I or F flags
            cpu.push_entire_state(mem);
            cpu.reg.pc = mem.read_word(crate::cpu::VEC_SWI2);
        }
        0x3F => {
            cpu.reg.cc.set_entire(true);
            cpu.push_entire_state(mem);
            // SWI2 does NOT set I or F flags
            cpu.reg.pc = mem.read_word(crate::cpu::VEC_SWI2);
        }

        // =================================================================
        // CMPD — compare D (16-bit subtract, discard result)
        // =================================================================
        0x83 => {
            let v = cpu.fetch_word(mem);
            let d = cpu.reg.d;
            alu::sub16(d, v, &mut cpu.reg.cc);
        }
        0x93 => {
            let addr = cpu.addr_direct(mem);
            let v = mem.read_word(addr);
            let d = cpu.reg.d;
            alu::sub16(d, v, &mut cpu.reg.cc);
        }
        0xA3 => {
            let (addr, ex) = cpu.addr_indexed(mem);
            cpu.cycles += ex as u64;
            let v = mem.read_word(addr);
            let d = cpu.reg.d;
            alu::sub16(d, v, &mut cpu.reg.cc);
        }
        0xB3 => {
            let addr = cpu.addr_extended(mem);
            let v = mem.read_word(addr);
            let d = cpu.reg.d;
            alu::sub16(d, v, &mut cpu.reg.cc);
        }

        // =================================================================
        // CMPY — compare Y
        // =================================================================
        0x8C => {
            let v = cpu.fetch_word(mem);
            let y = cpu.reg.y;
            alu::sub16(y, v, &mut cpu.reg.cc);
        }
        0x9C => {
            let addr = cpu.addr_direct(mem);
            let v = mem.read_word(addr);
            let y = cpu.reg.y;
            alu::sub16(y, v, &mut cpu.reg.cc);
        }
        0xAC => {
            let (addr, ex) = cpu.addr_indexed(mem);
            cpu.cycles += ex as u64;
            let v = mem.read_word(addr);
            let y = cpu.reg.y;
            alu::sub16(y, v, &mut cpu.reg.cc);
        }
        0xBC => {
            let addr = cpu.addr_extended(mem);
            let v = mem.read_word(addr);
            let y = cpu.reg.y;
            alu::sub16(y, v, &mut cpu.reg.cc);
        }

        // =================================================================
        // LDY / STY
        // =================================================================
        0x8E => {
            let v = cpu.fetch_word(mem);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.y = v;
        }
        0x9E => {
            let addr = cpu.addr_direct(mem);
            let v = mem.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.y = v;
        }
        0x9F => {
            let addr = cpu.addr_direct(mem);
            let v = cpu.reg.y;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            mem.write_word(addr, v);
        }
        0xAE => {
            let (addr, ex) = cpu.addr_indexed(mem);
            cpu.cycles += ex as u64;
            let v = mem.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.y = v;
        }
        0xAF => {
            let (addr, ex) = cpu.addr_indexed(mem);
            cpu.cycles += ex as u64;
            let v = cpu.reg.y;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            mem.write_word(addr, v);
        }
        0xBE => {
            let addr = cpu.addr_extended(mem);
            let v = mem.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.y = v;
        }
        0xBF => {
            let addr = cpu.addr_extended(mem);
            let v = cpu.reg.y;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            mem.write_word(addr, v);
        }

        // =================================================================
        // LDS / STS
        // =================================================================
        0xC3 => {
            // XADDD imm (undocumented)
            // XADDD performs a 16-bit addition of the operand with D, and
            // sets the Z,N,C,V flags in an identical manner to ADDD. The
            // result is, however, not written back to D.
            let v = cpu.fetch_word(mem);
            let d = cpu.reg.d;
            let _r = alu::add16(d, v, &mut cpu.reg.cc);
        }
        0xCE => {
            let v = cpu.fetch_word(mem);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.s = v;
            cpu.arm_nmi();
        }
        0xD3 => {
            // XADDD direct (undocumented)
            let addr = cpu.addr_direct(mem);
            let v = mem.read_word(addr);
            let d = cpu.reg.d;
            let _r = alu::add16(d, v, &mut cpu.reg.cc);
        }
        0xDE => {
            let addr = cpu.addr_direct(mem);
            let v = mem.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.s = v;
            cpu.arm_nmi();
        }
        0xDF => {
            let addr = cpu.addr_direct(mem);
            let v = cpu.reg.s;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            mem.write_word(addr, v);
        }
        0xE3 => {
            // XADDD indexed (undocumented)
            let (addr, ex) = cpu.addr_indexed(mem);
            cpu.cycles += ex as u64;
            let v = mem.read_word(addr);
            let d = cpu.reg.d;
            let _r = alu::add16(d, v, &mut cpu.reg.cc);
        }
        0xEE => {
            let (addr, ex) = cpu.addr_indexed(mem);
            cpu.cycles += ex as u64;
            let v = mem.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.s = v;
            cpu.arm_nmi();
        }
        0xEF => {
            let (addr, ex) = cpu.addr_indexed(mem);
            cpu.cycles += ex as u64;
            let v = cpu.reg.s;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            mem.write_word(addr, v);
        }
        0xF3 => {
            // XADDD extended (undocumented)
            let addr = cpu.addr_extended(mem);
            let v = mem.read_word(addr);
            let d = cpu.reg.d;
            let _r = alu::add16(d, v, &mut cpu.reg.cc);
        }
        0xFE => {
            let addr = cpu.addr_extended(mem);
            let v = mem.read_word(addr);
            alu::ld16_flags(v, &mut cpu.reg.cc);
            cpu.reg.s = v;
            cpu.arm_nmi();
        }
        0xFF => {
            let addr = cpu.addr_extended(mem);
            let v = cpu.reg.s;
            alu::ld16_flags(v, &mut cpu.reg.cc);
            mem.write_word(addr, v);
        }

        // Illegal Page 1 opcodes
        _ => {
            // 1 cycle already consumed by the page prefix fetch
            //debug!("Illegal Page 1 opcode: 0x10 {:02X}", opcode);
            cpu.illegal = true;
        }
    }
}
