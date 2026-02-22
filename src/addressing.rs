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

//! Indexed addressing mode decoder for the 6809.
//!
//! The post-byte encodes the index register, offset type, and indirection.
//! Returns `(effective_address, extra_cycles)`.

use crate::bus::Bus;
use crate::cpu::Cpu;

/// Decode an indexed addressing post-byte and compute the effective address.
///
/// Returns `(ea, extra_cycles)` where `extra_cycles` is the additional cycle
/// count beyond the base instruction cycles.
pub fn indexed(cpu: &mut Cpu, bus: &impl Bus) -> (u16, u8) {
    let post = cpu.fetch_byte(bus);

    // Bit 7 == 0: 5-bit signed offset from R, no indirection
    if post & 0x80 == 0 {
        let reg = index_reg(cpu, post);
        // Bits 4..0 are a 5-bit signed offset (-16 to +15)
        let offset = if post & 0x10 != 0 {
            // sign-extend: set bits 7..5
            (post | 0xE0) as i8 as i16 as u16
        } else {
            (post & 0x1F) as u16
        };
        let ea = reg.wrapping_add(offset);
        return (ea, 1);
    }

    // Bit 7 == 1: complex indexed modes
    let indirect = post & 0x10 != 0;
    let mode = post & 0x0F;

    let (ea, extra) = match mode {
        // 0x00: ,R+ (post-increment by 1) — no indirect variant
        0x00 => {
            let reg = index_reg(cpu, post);
            let ea = reg;
            set_index_reg(cpu, post, reg.wrapping_add(1));
            (ea, 2)
        }
        // 0x01: ,R++ (post-increment by 2)
        0x01 => {
            let reg = index_reg(cpu, post);
            let ea = reg;
            set_index_reg(cpu, post, reg.wrapping_add(2));
            (ea, 3)
        }
        // 0x02: ,-R (pre-decrement by 1) — no indirect variant
        0x02 => {
            let reg = index_reg(cpu, post).wrapping_sub(1);
            set_index_reg(cpu, post, reg);
            (reg, 2)
        }
        // 0x03: ,--R (pre-decrement by 2)
        0x03 => {
            let reg = index_reg(cpu, post).wrapping_sub(2);
            set_index_reg(cpu, post, reg);
            (reg, 3)
        }
        // 0x04: ,R (zero offset)
        0x04 => {
            let ea = index_reg(cpu, post);
            (ea, 0)
        }
        // 0x05: B,R
        0x05 => {
            let reg = index_reg(cpu, post);
            let offset = cpu.reg.b() as i8 as i16 as u16;
            (reg.wrapping_add(offset), 1)
        }
        // 0x06: A,R
        0x06 => {
            let reg = index_reg(cpu, post);
            let offset = cpu.reg.a() as i8 as i16 as u16;
            (reg.wrapping_add(offset), 1)
        }
        // 0x08: 8-bit offset, R
        0x08 => {
            let reg = index_reg(cpu, post);
            let offset = cpu.fetch_byte(bus) as i8 as i16 as u16;
            (reg.wrapping_add(offset), 1)
        }
        // 0x09: 16-bit offset, R
        0x09 => {
            let reg = index_reg(cpu, post);
            let offset = cpu.fetch_word(bus);
            (reg.wrapping_add(offset), 4)
        }
        // 0x0B: D,R
        0x0B => {
            let reg = index_reg(cpu, post);
            let offset = cpu.reg.d;
            (reg.wrapping_add(offset), 4)
        }
        // 0x0C: 8-bit offset, PC
        0x0C => {
            let offset = cpu.fetch_byte(bus) as i8 as i16 as u16;
            let ea = cpu.reg.pc.wrapping_add(offset);
            (ea, 1)
        }
        // 0x0D: 16-bit offset, PC
        0x0D => {
            let offset = cpu.fetch_word(bus);
            let ea = cpu.reg.pc.wrapping_add(offset);
            (ea, 5)
        }
        // 0x0F: Extended indirect [address] (only valid with indirect bit)
        0x0F if indirect => {
            let ea = cpu.fetch_word(bus);
            // The indirect dereference happens below. Base extra = 5, then +3 for indirect.
            // But for extended indirect, total extra = 5 (already includes indirection).
            let ptr = bus.read_word(ea);
            return (ptr, 5);
        }
        // Illegal indexed modes
        _ => {
            // Undefined behavior — return 0 with 0 extra cycles
            (0, 0)
        }
    };

    if indirect {
        // Add 3 cycles for indirection and dereference the EA
        let ptr = bus.read_word(ea);
        (ptr, extra + 3)
    } else {
        (ea, extra)
    }
}

/// Read the index register selected by bits 6-5 of the post-byte.
fn index_reg(cpu: &Cpu, post: u8) -> u16 {
    match (post >> 5) & 0x03 {
        0 => cpu.reg.x,
        1 => cpu.reg.y,
        2 => cpu.reg.u,
        3 => cpu.reg.s,
        _ => unreachable!(),
    }
}

/// Write to the index register selected by bits 6-5 of the post-byte.
fn set_index_reg(cpu: &mut Cpu, post: u8, val: u16) {
    match (post >> 5) & 0x03 {
        0 => cpu.reg.x = val,
        1 => cpu.reg.y = val,
        2 => cpu.reg.u = val,
        3 => {
            cpu.reg.s = val;
            cpu.arm_nmi();
        }
        _ => unreachable!(),
    }
}
