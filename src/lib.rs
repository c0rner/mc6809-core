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
//! Provides a [`Cpu`] that executes 6809 instructions against any memory system
//! implementing the [`Memory`] trait. Peripheral timing and interrupt signals are
//! handled separately via the [`Clocked`] trait, which is called by the host loop
//! independently of the CPU.
//!
//! ## Example
//!
//! ```rust
//! use mc6809_core::{Cpu, Memory};
//!
//! struct FlatRam([u8; 65536]);
//!
//! impl Memory for FlatRam {
//!     fn read(&mut self, addr: u16) -> u8 { self.0[addr as usize] }
//!     fn write(&mut self, addr: u16, val: u8) { self.0[addr as usize] = val; }
//! }
//!
//! let mut mem = FlatRam([0; 65536]);
//! // Place a reset vector pointing to 0x0400
//! mem.0[0xFFFE] = 0x04;
//! mem.0[0xFFFF] = 0x00;
//! // Place a NOP at 0x0400
//! mem.0[0x0400] = 0x12;
//!
//! let mut cpu = Cpu::new();
//! cpu.reset(&mut mem);
//! assert_eq!(cpu.registers().pc, 0x0400);
//! cpu.step(&mut mem);
//! assert_eq!(cpu.registers().pc, 0x0401);
//! ```

pub mod addressing;
pub mod alu;
mod cpu;
pub mod memory;
pub mod peripheral;
pub mod registers;

pub use cpu::{Cpu, RegistersMut, instruction_cycles};
pub use memory::Memory;
pub use peripheral::{BusSignals, Clocked};
pub use registers::{ConditionCodes, Registers};

#[cfg(test)]
mod tests;
