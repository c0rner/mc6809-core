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

//! # emu6809-core
//!
//! A Motorola 6809 CPU emulator core.
//!
//! Provides a [`Cpu`] that executes 6809 instructions against any
//! memory system implementing the [`Bus`] trait.
//!
//! ## Example
//!
//! ```rust
//! use mc6809_core::{Cpu, Bus};
//!
//! struct FlatRam([u8; 65536]);
//!
//! impl Bus for FlatRam {
//!     fn read(&self, addr: u16) -> u8 { self.0[addr as usize] }
//!     fn write(&mut self, addr: u16, val: u8) { self.0[addr as usize] = val; }
//! }
//!
//! let mut bus = FlatRam([0; 65536]);
//! // Place a reset vector pointing to 0x0400
//! bus.0[0xFFFE] = 0x04;
//! bus.0[0xFFFF] = 0x00;
//! // Place a NOP at 0x0400
//! bus.0[0x0400] = 0x12;
//!
//! let mut cpu = Cpu::new();
//! cpu.reset(&mut bus);
//! assert_eq!(cpu.reg.pc, 0x0400);
//! cpu.step(&mut bus);
//! assert_eq!(cpu.reg.pc, 0x0401);
//! ```

pub mod addressing;
pub mod alu;
pub mod bus;
mod cpu;
pub mod registers;

pub use bus::Bus;
pub use cpu::Cpu;
pub use registers::{ConditionCodes, Registers};

#[cfg(test)]
mod tests;
