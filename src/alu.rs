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

//! ALU (Arithmetic Logic Unit) helpers for the 6809.
//!
//! Each operation takes operand(s) and cc flags, performs the operation,
//! and returns the result with updated flags.

use crate::registers::ConditionCodes;

// ---------------------------------------------------------------------------
// 8-bit arithmetic
// ---------------------------------------------------------------------------

/// ADD: result = a + b. Sets H, N, Z, V, C.
pub fn add8(a: u8, b: u8, cc: &mut ConditionCodes) -> u8 {
    let r16 = a as u16 + b as u16;
    let result = r16 as u8;
    cc.set_half_carry((a ^ b ^ result) & 0x10 != 0);
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow((a ^ result) & (b ^ result) & 0x80 != 0);
    cc.set_carry(r16 > 0xFF);
    result
}

/// ADC: result = a + b + carry. Sets H, N, Z, V, C.
pub fn adc8(a: u8, b: u8, cc: &mut ConditionCodes) -> u8 {
    let c = cc.carry() as u8;
    let r16 = a as u16 + b as u16 + c as u16;
    let result = r16 as u8;
    cc.set_half_carry((a ^ b ^ result) & 0x10 != 0);
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow((a ^ result) & (b ^ result) & 0x80 != 0);
    cc.set_carry(r16 > 0xFF);
    result
}

/// SUB: result = a - b. Sets H (undefined per spec, we leave it), N, Z, V, C.
pub fn sub8(a: u8, b: u8, cc: &mut ConditionCodes) -> u8 {
    let r16 = (a as u16).wrapping_sub(b as u16);
    let result = r16 as u8;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow((a ^ b) & (a ^ result) & 0x80 != 0);
    cc.set_carry(a < b);
    result
}

/// SBC: result = a - b - carry. Sets N, Z, V, C.
pub fn sbc8(a: u8, b: u8, cc: &mut ConditionCodes) -> u8 {
    let c = cc.carry() as u16;
    let r16 = (a as u16).wrapping_sub(b as u16).wrapping_sub(c);
    let result = r16 as u8;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow((a ^ b) & (a ^ result) & 0x80 != 0);
    cc.set_carry(r16 > 0xFF);
    result
}

/// NEG: result = 0 - val. Sets N, Z, V, C.
pub fn neg8(val: u8, cc: &mut ConditionCodes) -> u8 {
    let result = (val as i8).wrapping_neg() as u8;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow(val == 0x80);
    cc.set_carry(val != 0x00);
    result
}

/// COM: result = !val. Sets N, Z, V=0, C=1.
pub fn com8(val: u8, cc: &mut ConditionCodes) -> u8 {
    let result = !val;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow(false);
    cc.set_carry(true);
    result
}

/// INC: result = val + 1. Sets N, Z, V. Does NOT affect C.
pub fn inc8(val: u8, cc: &mut ConditionCodes) -> u8 {
    let result = val.wrapping_add(1);
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow(val == 0x7F);
    result
}

/// DEC: result = val - 1. Sets N, Z, V. Does NOT affect C.
pub fn dec8(val: u8, cc: &mut ConditionCodes) -> u8 {
    let result = val.wrapping_sub(1);
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow(val == 0x80);
    result
}

/// CLR: result = 0. Sets N=0, Z=1, V=0, C=0.
pub fn clr8(cc: &mut ConditionCodes) -> u8 {
    cc.set_negative(false);
    cc.set_zero(true);
    cc.set_overflow(false);
    cc.set_carry(false);
    0
}

/// TST: test value. Sets N, Z, V=0. Does NOT affect C.
pub fn tst8(val: u8, cc: &mut ConditionCodes) {
    cc.set_negative(val & 0x80 != 0);
    cc.set_zero(val == 0);
    cc.set_overflow(false);
}

// ---------------------------------------------------------------------------
// 8-bit logical
// ---------------------------------------------------------------------------

/// AND: result = a & b. Sets N, Z, V=0.
pub fn and8(a: u8, b: u8, cc: &mut ConditionCodes) -> u8 {
    let result = a & b;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow(false);
    result
}

/// OR: result = a | b. Sets N, Z, V=0.
pub fn or8(a: u8, b: u8, cc: &mut ConditionCodes) -> u8 {
    let result = a | b;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow(false);
    result
}

/// EOR: result = a ^ b. Sets N, Z, V=0.
pub fn eor8(a: u8, b: u8, cc: &mut ConditionCodes) -> u8 {
    let result = a ^ b;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow(false);
    result
}

// ---------------------------------------------------------------------------
// 8-bit shifts/rotates
// ---------------------------------------------------------------------------

/// LSR: logical shift right. Bit 0 → C, 0 → bit 7. Sets N=0, Z, C.
pub fn lsr8(val: u8, cc: &mut ConditionCodes) -> u8 {
    cc.set_carry(val & 0x01 != 0);
    let result = val >> 1;
    cc.set_negative(false);
    cc.set_zero(result == 0);
    result
}

/// ASR: arithmetic shift right. Bit 0 → C, bit 7 preserved. Sets N, Z, C.
pub fn asr8(val: u8, cc: &mut ConditionCodes) -> u8 {
    cc.set_carry(val & 0x01 != 0);
    let result = ((val as i8) >> 1) as u8;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    result
}

/// ASL/LSL: arithmetic/logical shift left. Bit 7 → C, 0 → bit 0. Sets N, Z, V, C.
pub fn asl8(val: u8, cc: &mut ConditionCodes) -> u8 {
    cc.set_carry(val & 0x80 != 0);
    let result = val << 1;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow((val ^ result) & 0x80 != 0);
    result
}

/// ROL: rotate left through carry. Old C → bit 0, bit 7 → new C. Sets N, Z, V, C.
pub fn rol8(val: u8, cc: &mut ConditionCodes) -> u8 {
    let old_c = cc.carry() as u8;
    cc.set_carry(val & 0x80 != 0);
    let result = (val << 1) | old_c;
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow((val ^ result) & 0x80 != 0);
    result
}

/// ROR: rotate right through carry. Old C → bit 7, bit 0 → new C. Sets N, Z, C.
pub fn ror8(val: u8, cc: &mut ConditionCodes) -> u8 {
    let old_c = cc.carry() as u8;
    cc.set_carry(val & 0x01 != 0);
    let result = (val >> 1) | (old_c << 7);
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    result
}

// ---------------------------------------------------------------------------
// 16-bit arithmetic
// ---------------------------------------------------------------------------

/// ADD16: result = a + b. Sets N, Z, V, C. (No half-carry for 16-bit.)
pub fn add16(a: u16, b: u16, cc: &mut ConditionCodes) -> u16 {
    let r32 = a as u32 + b as u32;
    let result = r32 as u16;
    cc.set_negative(result & 0x8000 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow((a ^ result) & (b ^ result) & 0x8000 != 0);
    cc.set_carry(r32 > 0xFFFF);
    result
}

/// SUB16: result = a - b. Sets N, Z, V, C.
pub fn sub16(a: u16, b: u16, cc: &mut ConditionCodes) -> u16 {
    let r32 = (a as u32).wrapping_sub(b as u32);
    let result = r32 as u16;
    cc.set_negative(result & 0x8000 != 0);
    cc.set_zero(result == 0);
    cc.set_overflow((a ^ b) & (a ^ result) & 0x8000 != 0);
    cc.set_carry(a < b);
    result
}

// ---------------------------------------------------------------------------
// 16-bit load/store flag helpers
// ---------------------------------------------------------------------------

/// Set flags for a 16-bit load result. Sets N, Z, V=0.
pub fn ld16_flags(val: u16, cc: &mut ConditionCodes) {
    cc.set_nz16(val);
    cc.set_overflow(false);
}

/// Set flags for an 8-bit load result. Sets N, Z, V=0.
pub fn ld8_flags(val: u8, cc: &mut ConditionCodes) {
    cc.set_nz8(val);
    cc.set_overflow(false);
}

// ---------------------------------------------------------------------------
// Special operations
// ---------------------------------------------------------------------------

/// DAA: Decimal Adjust Accumulator. Adjusts A after BCD addition.
/// Sets N, Z, C (V is undefined).
pub fn daa(a: u8, cc: &mut ConditionCodes) -> u8 {
    let mut correction: u8 = 0;
    let mut carry = cc.carry();

    // Lower nibble
    if cc.half_carry() || (a & 0x0F) > 9 {
        correction |= 0x06;
    }

    // Upper nibble
    if carry || a > 0x99 {
        correction |= 0x60;
        carry = true;
    }

    let result = a.wrapping_add(correction);
    cc.set_negative(result & 0x80 != 0);
    cc.set_zero(result == 0);
    cc.set_carry(carry);
    // V is undefined per spec
    result
}

/// MUL: unsigned multiply A × B → D. Sets Z (D==0), C (bit 7 of B, i.e., bit 7 of result low byte).
pub fn mul(a: u8, b: u8, cc: &mut ConditionCodes) -> u16 {
    let result = (a as u16) * (b as u16);
    cc.set_zero(result == 0);
    cc.set_carry(result & 0x0080 != 0); // bit 7 of low byte (B)
    result
}

/// SEX: Sign-extend B into A. If B bit 7 is set, A = 0xFF, else A = 0x00.
/// Sets N, Z. V is always cleared per observed behavior (not spec).
pub fn sex(b: u8, cc: &mut ConditionCodes) -> u16 {
    let a = if b & 0x80 != 0 { 0xFF } else { 0x00 };
    let d = ((a as u16) << 8) | (b as u16);
    cc.set_negative(a == 0xFF);
    cc.set_zero(b == 0);
    d
}
