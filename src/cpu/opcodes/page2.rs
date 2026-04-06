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

//! Page 2 opcode implementations (prefix 0x11).
//!
//! Contains: SWI3, CMPU, CMPS.
//! Contains all undocumented page 2 opcodes except all store immediate,
//! source: <https://github.com/hoglet67/6809Decoder/wiki/Undocumented-6809-Behaviours>

use crate::alu;
use crate::bus::Bus;
use crate::cpu::Cpu;

/// Base cycle counts for Page 2 opcodes.
#[rustfmt::skip]
const PAGE2_CYCLES: [u8; 256] = {
    let mut t = [0u8; 256];
    t[0x3E] = 20; // XFIRQ (undocumented)
    t[0x3F] = 20; // SWI3
    t[0x83] = 5;  // CMPU imm
    t[0x8C] = 5;  // CMPS imm
    t[0x93] = 7;  // CMPU direct
    t[0x9C] = 7;  // CMPS direct
    t[0xA3] = 7;  // CMPU indexed
    t[0xAC] = 7;  // CMPS indexed
    t[0xB3] = 8;  // CMPU extended
    t[0xBC] = 8;  // CMPS extended
    t[0xC3] = 5;  // XADDU imm (undocumented)
    t[0xD3] = 7;  // XADDU direct (undocumented)
    t[0xE3] = 7;  // XADDU indexed (undocumented)
    t[0xF3] = 8;  // XADDU extended (undocumented)
    t
};

/// Returns the base cycle count for a Page 2 (0x11xx) sub-opcode.
pub(super) fn cycles(sub: u8) -> u8 {
    PAGE2_CYCLES[sub as usize]
}

pub fn execute(cpu: &mut Cpu, bus: &mut impl Bus, opcode: u8) {
    cpu.cycles += PAGE2_CYCLES[opcode as usize] as u64;

    match opcode {
        // =================================================================
        // XFIRQ (undocumented)
        // =================================================================
        // This instruction is similar to SWI (0x3F), except the
        // FIRQ vector (0xFFF6/7) is used to determine the next
        // PC value, and it does not correctly set the E flag in
        // the saved machine state.
        // Flags: all flags are unchanged
        // Note: unlike a hardware FIRQ, the F and I flags are not set.
        0x3E => {
            cpu.push_entire_state(bus);
            cpu.reg.pc = bus.read_word(crate::cpu::VEC_FIRQ);
        }
        // =================================================================
        // SWI3
        // =================================================================
        0x3F => {
            cpu.reg.cc.set_entire(true);
            cpu.push_entire_state(bus);
            // SWI3 does NOT set I or F flags
            cpu.reg.pc = bus.read_word(crate::cpu::VEC_SWI3);
        }

        // =================================================================
        // CMPU — compare U
        // =================================================================
        0x83 => {
            let v = cpu.fetch_word(bus);
            let u = cpu.reg.u;
            alu::sub16(u, v, &mut cpu.reg.cc);
        }
        0x93 => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read_word(addr);
            let u = cpu.reg.u;
            alu::sub16(u, v, &mut cpu.reg.cc);
        }
        0xA3 => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read_word(addr);
            let u = cpu.reg.u;
            alu::sub16(u, v, &mut cpu.reg.cc);
        }
        0xB3 => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read_word(addr);
            let u = cpu.reg.u;
            alu::sub16(u, v, &mut cpu.reg.cc);
        }

        // =================================================================
        // CMPS — compare S
        // =================================================================
        0x8C => {
            let v = cpu.fetch_word(bus);
            let s = cpu.reg.s;
            alu::sub16(s, v, &mut cpu.reg.cc);
        }
        0x9C => {
            let addr = cpu.addr_direct(bus);
            let v = bus.read_word(addr);
            let s = cpu.reg.s;
            alu::sub16(s, v, &mut cpu.reg.cc);
        }
        0xAC => {
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read_word(addr);
            let s = cpu.reg.s;
            alu::sub16(s, v, &mut cpu.reg.cc);
        }
        0xBC => {
            let addr = cpu.addr_extended(bus);
            let v = bus.read_word(addr);
            let s = cpu.reg.s;
            alu::sub16(s, v, &mut cpu.reg.cc);
        }

        // =================================================================
        // XADDU - add U (undocumented)
        // =================================================================
        // XADDU performs a 16-bit addition of the operand with
        // (U | 0xFF00), and sets the Z,N,C,V flags in an identical manner
        // to ADDD. The result is, however, not written back to U.
        0xC3 => {
            // XADDU imm
            let v = cpu.fetch_word(bus);
            let u = cpu.reg.u | 0xFF00;
            let _r = alu::add16(u, v, &mut cpu.reg.cc);
        }
        0xD3 => {
            // XADDU direct
            let addr = cpu.addr_direct(bus);
            let v = bus.read_word(addr);
            let u = cpu.reg.u | 0xFF00;
            let _r = alu::add16(u, v, &mut cpu.reg.cc);
        }
        0xE3 => {
            // XADDU indexed
            let (addr, ex) = cpu.addr_indexed(bus);
            cpu.cycles += ex as u64;
            let v = bus.read_word(addr);
            let u = cpu.reg.u | 0xFF00;
            let _r = alu::add16(u, v, &mut cpu.reg.cc);
        }
        0xF3 => {
            // XADDU extended
            let addr = cpu.addr_extended(bus);
            let v = bus.read_word(addr);
            let u = cpu.reg.u | 0xFF00;
            let _r = alu::add16(u, v, &mut cpu.reg.cc);
        }

        // Illegal Page 2 opcodes
        _ => {
            // 1 cycle already consumed by the page prefix fetch
            //debug!("Illegal Page 2 opcode: {:02X}", opcode);
            cpu.illegal = true;
        }
    }
}
