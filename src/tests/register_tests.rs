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

//! Unit tests for the CPU registers.

use crate::registers::{ConditionCodes, Registers};
use std::mem;

// These are offsets into `reg` (which starts at offset 0), matching the
// JIT/FFI contract defined by `#[repr(C)]` in `Registers`.
const OFF_D: usize = 0;
const OFF_X: usize = 2;
const OFF_Y: usize = 4;
const OFF_U: usize = 6;
const OFF_S: usize = 8;
const OFF_PC: usize = 10;
const OFF_DP: usize = 12;
const OFF_CC: usize = 13;

// ---------------------------------------------------------------------------
// ConditionCodes layout
// ---------------------------------------------------------------------------

/// `ConditionCodes` is `repr(transparent)` over `u8`: JIT code reads and
/// writes it as a plain byte, so its size and alignment must match exactly.
#[test]
fn condition_codes_layout() {
    assert_eq!(mem::size_of::<ConditionCodes>(), mem::size_of::<u8>());
    assert_eq!(mem::align_of::<ConditionCodes>(), mem::align_of::<u8>());
}

// ---------------------------------------------------------------------------
// Registers layout
// ---------------------------------------------------------------------------

/// Total size and alignment of the `repr(C)` register file.
#[test]
fn registers_size_and_align() {
    assert_eq!(mem::size_of::<Registers>(), 14);
    assert_eq!(mem::align_of::<Registers>(), 2);
}

/// Each field offset must match the constants used by JIT/FFI callers.
#[test]
fn registers_field_offsets() {
    assert_eq!(mem::offset_of!(Registers, d), OFF_D);
    assert_eq!(mem::offset_of!(Registers, x), OFF_X);
    assert_eq!(mem::offset_of!(Registers, y), OFF_Y);
    assert_eq!(mem::offset_of!(Registers, u), OFF_U);
    assert_eq!(mem::offset_of!(Registers, s), OFF_S);
    assert_eq!(mem::offset_of!(Registers, pc), OFF_PC);
    assert_eq!(mem::offset_of!(Registers, dp), OFF_DP);
    assert_eq!(mem::offset_of!(Registers, cc), OFF_CC);
}
