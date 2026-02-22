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

/// Memory bus trait for the 6809 CPU.
///
/// Implement this trait to provide the CPU with access to memory and I/O.
/// The 6809 has a 16-bit address bus (64KB address space) and an 8-bit data bus.
pub trait Bus {
    /// Read a byte from the given address.
    fn read(&self, addr: u16) -> u8;

    /// Write a byte to the given address.
    fn write(&mut self, addr: u16, val: u8);

    /// Read a big-endian 16-bit word (high byte at `addr`, low byte at `addr + 1`).
    fn read_word(&self, addr: u16) -> u16 {
        let hi = self.read(addr) as u16;
        let lo = self.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Write a big-endian 16-bit word (high byte at `addr`, low byte at `addr + 1`).
    fn write_word(&mut self, addr: u16, val: u16) {
        self.write(addr, (val >> 8) as u8);
        self.write(addr.wrapping_add(1), val as u8);
    }
}
