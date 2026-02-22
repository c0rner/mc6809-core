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

use crate::bus::Bus;
use crate::cpu::Cpu;

/// Execute a single opcode (already fetched).
impl Cpu {
    pub(crate) fn execute(&mut self, bus: &mut impl Bus, opcode: u8) {
        match opcode {
            0x10 => {
                let op2 = self.fetch_byte(bus);
                page1::execute(self, bus, op2);
            }
            0x11 => {
                let op2 = self.fetch_byte(bus);
                page2::execute(self, bus, op2);
            }
            _ => {
                page0::execute(self, bus, opcode);
            }
        }
    }
}
