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

//! Unit tests for `instruction_cycles`.

use crate::instruction_cycles;

// ---------------------------------------------------------------------------
// Empty input
// ---------------------------------------------------------------------------

#[test]
fn empty_slice_returns_zero() {
    assert_eq!(instruction_cycles(&[]), 0);
}

// ---------------------------------------------------------------------------
// Page 0 opcodes (no prefix byte)
// ---------------------------------------------------------------------------

/// NOP (0x12) costs 2 cycles on the 6809.
#[test]
fn page0_nop_is_2_cycles() {
    assert_eq!(instruction_cycles(&[0x12]), 2);
}

/// NEGA (0x40) costs 2 cycles.
#[test]
fn page0_nega_is_2_cycles() {
    assert_eq!(instruction_cycles(&[0x40]), 2);
}

// ---------------------------------------------------------------------------
// Page 1 opcodes (0x10 prefix)
// ---------------------------------------------------------------------------

/// LDY immediate (0x10 0x8E) costs 4 cycles.
#[test]
fn page1_ldy_imm_is_4_cycles() {
    assert_eq!(instruction_cycles(&[0x10, 0x8E]), 4);
}

/// SWI2 (0x10 0x3F) costs 20 cycles.
#[test]
fn page1_swi2_is_20_cycles() {
    assert_eq!(instruction_cycles(&[0x10, 0x3F]), 20);
}

/// A page 1 prefix with no sub-opcode byte returns 0.
#[test]
fn page1_prefix_only_returns_zero() {
    assert_eq!(instruction_cycles(&[0x10]), 0);
}

/// An unrecognised page 1 sub-opcode (0x10 0x00) returns 0.
#[test]
fn page1_illegal_sub_opcode_returns_zero() {
    assert_eq!(instruction_cycles(&[0x10, 0x00]), 0);
}

// ---------------------------------------------------------------------------
// Page 2 opcodes (0x11 prefix)
// ---------------------------------------------------------------------------

/// CMPU immediate (0x11 0x83) costs 5 cycles.
#[test]
fn page2_cmpu_imm_is_5_cycles() {
    assert_eq!(instruction_cycles(&[0x11, 0x83]), 5);
}

/// SWI3 (0x11 0x3F) costs 20 cycles.
#[test]
fn page2_swi3_is_20_cycles() {
    assert_eq!(instruction_cycles(&[0x11, 0x3F]), 20);
}

/// A page 2 prefix with no sub-opcode byte returns 0.
#[test]
fn page2_prefix_only_returns_zero() {
    assert_eq!(instruction_cycles(&[0x11]), 0);
}

/// An unrecognised page 2 sub-opcode (0x11 0x00) returns 0.
#[test]
fn page2_illegal_sub_opcode_returns_zero() {
    assert_eq!(instruction_cycles(&[0x11, 0x00]), 0);
}
