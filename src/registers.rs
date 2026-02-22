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

use std::fmt;

// ---------------------------------------------------------------------------
// Condition Code Register
// ---------------------------------------------------------------------------

/// Bit positions in the CC register.
const CC_C: u8 = 0x01; // Carry
const CC_V: u8 = 0x02; // Overflow
const CC_Z: u8 = 0x04; // Zero
const CC_N: u8 = 0x08; // Negative
const CC_I: u8 = 0x10; // IRQ inhibit
const CC_H: u8 = 0x20; // Half-carry
const CC_F: u8 = 0x40; // FIRQ inhibit
const CC_E: u8 = 0x80; // Entire state saved

/// The 6809 Condition Code register, stored as a packed byte.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct ConditionCodes(u8);

impl ConditionCodes {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn from_byte(b: u8) -> Self {
        Self(b)
    }

    pub const fn to_byte(self) -> u8 {
        self.0
    }

    // ---- flag readers ----

    pub const fn carry(self) -> bool {
        self.0 & CC_C != 0
    }
    pub const fn overflow(self) -> bool {
        self.0 & CC_V != 0
    }
    pub const fn zero(self) -> bool {
        self.0 & CC_Z != 0
    }
    pub const fn negative(self) -> bool {
        self.0 & CC_N != 0
    }
    pub const fn irq_inhibit(self) -> bool {
        self.0 & CC_I != 0
    }
    pub const fn half_carry(self) -> bool {
        self.0 & CC_H != 0
    }
    pub const fn firq_inhibit(self) -> bool {
        self.0 & CC_F != 0
    }
    pub const fn entire(self) -> bool {
        self.0 & CC_E != 0
    }

    // ---- flag writers ----

    pub fn set_carry(&mut self, v: bool) {
        self.set_bit(CC_C, v);
    }
    pub fn set_overflow(&mut self, v: bool) {
        self.set_bit(CC_V, v);
    }
    pub fn set_zero(&mut self, v: bool) {
        self.set_bit(CC_Z, v);
    }
    pub fn set_negative(&mut self, v: bool) {
        self.set_bit(CC_N, v);
    }
    pub fn set_irq_inhibit(&mut self, v: bool) {
        self.set_bit(CC_I, v);
    }
    pub fn set_half_carry(&mut self, v: bool) {
        self.set_bit(CC_H, v);
    }
    pub fn set_firq_inhibit(&mut self, v: bool) {
        self.set_bit(CC_F, v);
    }
    pub fn set_entire(&mut self, v: bool) {
        self.set_bit(CC_E, v);
    }

    /// Set N and Z based on an 8-bit result.
    pub fn set_nz8(&mut self, val: u8) {
        self.set_negative(val & 0x80 != 0);
        self.set_zero(val == 0);
    }

    /// Set N and Z based on a 16-bit result.
    pub fn set_nz16(&mut self, val: u16) {
        self.set_negative(val & 0x8000 != 0);
        self.set_zero(val == 0);
    }

    /// OR the CC byte with a mask (used by ORCC).
    pub fn or_with(&mut self, mask: u8) {
        self.0 |= mask;
    }

    /// AND the CC byte with a mask (used by ANDCC).
    pub fn and_with(&mut self, mask: u8) {
        self.0 &= mask;
    }

    fn set_bit(&mut self, mask: u8, v: bool) {
        if v {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }
}

impl fmt::Debug for ConditionCodes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CC({:02X} {}{}{}{}{}{}{}{})",
            self.0,
            if self.entire() { 'E' } else { '.' },
            if self.firq_inhibit() { 'F' } else { '.' },
            if self.half_carry() { 'H' } else { '.' },
            if self.irq_inhibit() { 'I' } else { '.' },
            if self.negative() { 'N' } else { '.' },
            if self.zero() { 'Z' } else { '.' },
            if self.overflow() { 'V' } else { '.' },
            if self.carry() { 'C' } else { '.' },
        )
    }
}

impl fmt::Display for ConditionCodes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// ---------------------------------------------------------------------------
// Register file
// ---------------------------------------------------------------------------

/// The 6809 programmer-visible register set.
///
/// Register D is stored as a `u16` with A in the high byte and B in the low byte,
/// matching the hardware layout.
#[derive(Clone, Copy, Debug, Default)]
pub struct Registers {
    /// Accumulator D (A:B). A = high byte, B = low byte.
    pub d: u16,
    /// Index register X
    pub x: u16,
    /// Index register Y
    pub y: u16,
    /// User stack pointer
    pub u: u16,
    /// Hardware stack pointer
    pub s: u16,
    /// Program counter
    pub pc: u16,
    /// Direct page register
    pub dp: u8,
    /// Condition codes
    pub cc: ConditionCodes,
}

impl Registers {
    pub const fn new() -> Self {
        Self {
            d: 0,
            x: 0,
            y: 0,
            u: 0,
            s: 0,
            pc: 0,
            dp: 0,
            cc: ConditionCodes::new(),
        }
    }

    // ---- A / B accessors (D = A:B, big-endian) ----

    /// Read accumulator A (high byte of D).
    pub const fn a(self) -> u8 {
        (self.d >> 8) as u8
    }

    /// Read accumulator B (low byte of D).
    pub const fn b(self) -> u8 {
        self.d as u8
    }

    /// Write accumulator A (high byte of D), preserving B.
    pub fn set_a(&mut self, val: u8) {
        self.d = (self.d & 0x00FF) | ((val as u16) << 8);
    }

    /// Write accumulator B (low byte of D), preserving A.
    pub fn set_b(&mut self, val: u8) {
        self.d = (self.d & 0xFF00) | (val as u16);
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PC={:04X} A={:02X} B={:02X} X={:04X} Y={:04X} U={:04X} S={:04X} DP={:02X} {}",
            self.pc,
            self.a(),
            self.b(),
            self.x,
            self.y,
            self.u,
            self.s,
            self.dp,
            self.cc,
        )
    }
}
