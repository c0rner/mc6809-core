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

//! Opcode dispatch and cycle tables for the 6809.

mod page0;
mod page1;
mod page2;

use crate::cpu::Cpu;
use crate::memory::Memory;

/// Returns the base cycle count for a 6809 instruction.
///
/// Pass the raw instruction bytes starting at the opcode byte. The function
/// inspects `bytes[0]` to detect the page prefix (0x10 = page 1, 0x11 = page 2)
/// and dispatches to the appropriate cycle table.
///
/// Repeated page-prefix chaining is intentionally unsupported: only the first
/// leading `0x10` or `0x11` is recognised as a page selector.
///
/// Returns `0` for an empty slice or an unrecognised sub-opcode.
pub fn instruction_cycles(bytes: &[u8]) -> u8 {
    match bytes.first().copied() {
        Some(0x10) => bytes.get(1).map_or(1, |&sub| page1::cycles(sub)),
        Some(0x11) => bytes.get(1).map_or(1, |&sub| page2::cycles(sub)),
        Some(op) => page0::cycles(op),
        None => 0,
    }
}

/// Execute a single opcode (already fetched).
///
/// Repeated page-prefix chaining is intentionally unsupported: if a page
/// prefix fetches another prefix as its sub-opcode, that second prefix is
/// handled as the page-local opcode byte rather than being discarded.
impl Cpu {
    pub(crate) fn execute(&mut self, mem: &mut impl Memory, opcode: u8) {
        match opcode {
            0x10 => {
                let op2 = self.fetch_byte(mem);
                page1::execute(self, mem, op2);
            }
            0x11 => {
                let op2 = self.fetch_byte(mem);
                page2::execute(self, mem, op2);
            }
            _ => {
                page0::execute(self, mem, opcode);
            }
        }
    }
}
