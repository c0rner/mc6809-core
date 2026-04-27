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

use crate::memory::Memory;
use crate::registers::Registers;

mod opcodes;

pub use opcodes::instruction_cycles;

// ---------------------------------------------------------------------------
// Interrupt vector addresses
// ---------------------------------------------------------------------------

pub const VEC_RESET: u16 = 0xFFFE;
pub const VEC_NMI: u16 = 0xFFFC;
pub const VEC_SWI: u16 = 0xFFFA;
pub const VEC_IRQ: u16 = 0xFFF8;
pub const VEC_FIRQ: u16 = 0xFFF6;
pub const VEC_SWI2: u16 = 0xFFF4;
pub const VEC_SWI3: u16 = 0xFFF2;

// ---------------------------------------------------------------------------
// CPU state
// ---------------------------------------------------------------------------

/// Motorola 6809 CPU emulator.
pub struct Cpu {
    /// Programmer-visible registers.
    reg: Registers,
    /// Total elapsed cycles since reset.
    cycles: u64,
    /// CPU execution has been explicitly halted by an instruction.
    halted: bool,
    /// Sticky status bit set when an illegal opcode is executed.
    illegal: bool,

    // ---- interrupt state ----
    /// NMI is armed (becomes true after first write to S).
    nmi_armed: bool,
    /// NMI pending (edge-triggered).
    nmi_pending: bool,
    /// FIRQ line asserted (level-triggered).
    firq_line: bool,
    /// IRQ line asserted (level-triggered).
    irq_line: bool,
    /// CWAI: entire state already pushed, waiting for interrupt.
    cwai: bool,
    /// SYNC: waiting for any interrupt edge.
    sync: bool,
}

impl Cpu {
    /// Create a new CPU with all state zeroed.
    pub fn new() -> Self {
        Self {
            reg: Registers::new(),
            cycles: 0,
            halted: false,
            illegal: false,
            nmi_armed: false,
            nmi_pending: false,
            firq_line: false,
            irq_line: false,
            cwai: false,
            sync: false,
        }
    }

    /// Hardware reset: read PC from reset vector, set I+F, clear state.
    pub fn reset(&mut self, mem: &mut impl Memory) {
        self.reg = Registers::new();
        self.reg.cc.set_irq_inhibit(true);
        self.reg.cc.set_firq_inhibit(true);
        self.reg.pc = mem.read_word(VEC_RESET);
        self.cycles = 0;
        self.halted = false;
        self.illegal = false;
        self.nmi_armed = false;
        self.nmi_pending = false;
        self.firq_line = false;
        self.irq_line = false;
        self.cwai = false;
        self.sync = false;
    }

    /// Read-only access to the programmer-visible registers.
    pub fn registers(&self) -> &Registers {
        &self.reg
    }

    /// Mutable access to the programmer-visible registers via an RAII guard.
    ///
    /// The guard implements [`std::ops::Deref`] and [`std::ops::DerefMut`] for
    /// [`Registers`], giving transparent read/write access to all fields. On drop
    /// it checks whether the hardware stack pointer (S) changed and, if so, arms
    /// the NMI — matching the real 6809 behaviour where the first write to S
    /// enables edge-triggered NMI.
    ///
    /// Note: the guard detects S changes by comparing the value on entry with the
    /// value on drop. Writing S to the value it already holds will not arm NMI, but
    /// because `nmi_armed` is sticky (never cleared) this is inconsequential in
    /// practice.
    ///
    /// # Example
    /// ```ignore
    /// cpu.registers_mut().s = 0x8000; // arms NMI
    /// {
    ///     let mut r = cpu.registers_mut();
    ///     r.s -= 2;
    ///     mem[r.s as usize] = lo;
    /// } // NMI armed here via Drop
    /// ```
    pub fn registers_mut(&mut self) -> RegistersMut<'_> {
        let prev_s = self.reg.s;
        RegistersMut { cpu: self, prev_s }
    }

    /// Total elapsed cycles since the last [`Self::reset`].
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    /// `true` if the CPU has been halted by a halt instruction.
    ///
    /// Illegal opcodes do not set this flag; they only set [`Self::illegal`]
    /// so the host can decide whether to keep running or stop.
    pub fn halted(&self) -> bool {
        self.halted
    }

    /// Assert or de-assert the halted state.
    pub fn set_halted(&mut self, active: bool) {
        self.halted = active;
    }

    /// Sticky flag set when an illegal opcode is executed.
    ///
    /// The 6809 keeps running after undefined opcodes, so this flag does not
    /// halt the CPU by itself. Hosts that want trap-like behaviour can check
    /// this flag after each [`Self::step`] and stop on their own policy.
    pub fn illegal(&self) -> bool {
        self.illegal
    }

    /// Clear the illegal opcode flag.
    pub fn clear_illegal(&mut self) {
        self.illegal = false;
    }

    /// Assert or de-assert the IRQ line (level-triggered).
    pub fn set_irq(&mut self, active: bool) {
        self.irq_line = active;
    }

    /// Assert or de-assert the FIRQ line (level-triggered).
    pub fn set_firq(&mut self, active: bool) {
        self.firq_line = active;
    }

    /// Trigger an NMI (edge-triggered). Only effective if NMI is armed.
    pub fn trigger_nmi(&mut self) {
        if self.nmi_armed {
            self.nmi_pending = true;
        }
    }

    /// Execute a single instruction (or handle a pending interrupt).
    /// Returns the number of cycles consumed.
    ///
    /// If the decoded instruction is illegal, the CPU records that in
    /// [`Self::illegal`] and continues execution unless the caller chooses to
    /// stop.
    pub fn step(&mut self, mem: &mut impl Memory) -> u64 {
        if self.halted {
            return 1;
        }

        let start_cycles = self.cycles;

        // Handle SYNC state: wait for any interrupt edge
        if self.sync {
            if self.nmi_pending || self.firq_line || self.irq_line {
                self.sync = false;
            } else {
                self.cycles += 1;
                return 1;
            }
        }

        // Check pending interrupts (priority: NMI > FIRQ > IRQ)
        if self.check_interrupts(mem) {
            return self.cycles - start_cycles;
        }

        // Fetch and execute one instruction
        let opcode = self.fetch_byte(mem);
        self.execute(mem, opcode);

        self.cycles - start_cycles
    }

    /// Run until at least `cycle_budget` cycles have been consumed.
    ///
    /// This method stops only when the cycle budget is exhausted or
    /// [`Self::halted`] becomes true. Illegal opcodes do not stop `run`; check
    /// [`Self::illegal`] in the host loop if that policy is desired.
    pub fn run(&mut self, mem: &mut impl Memory, cycle_budget: u64) -> u64 {
        let start_cycles = self.cycles;
        let target = self.cycles + cycle_budget;
        while self.cycles < target && !self.halted {
            self.step(mem);
        }
        self.cycles - start_cycles
    }

    // ---- interrupt logic ----

    fn check_interrupts(&mut self, mem: &mut impl Memory) -> bool {
        // NMI (edge-triggered, highest priority)
        if self.nmi_pending {
            self.nmi_pending = false;
            if !self.cwai {
                self.reg.cc.set_entire(true);
                self.push_entire_state(mem);
            }
            self.cwai = false;
            self.reg.cc.set_irq_inhibit(true);
            self.reg.cc.set_firq_inhibit(true);
            self.reg.pc = mem.read_word(VEC_NMI);
            self.cycles += 19;
            return true;
        }

        // FIRQ (level-triggered)
        if self.firq_line && !self.reg.cc.firq_inhibit() {
            if !self.cwai {
                self.reg.cc.set_entire(false);
                self.push_word_s(mem, self.reg.pc);
                self.push_byte_s(mem, self.reg.cc.to_byte());
            }
            self.cwai = false;
            self.reg.cc.set_irq_inhibit(true);
            self.reg.cc.set_firq_inhibit(true);
            self.reg.pc = mem.read_word(VEC_FIRQ);
            self.cycles += 10;
            return true;
        }

        // IRQ (level-triggered)
        if self.irq_line && !self.reg.cc.irq_inhibit() {
            if !self.cwai {
                self.reg.cc.set_entire(true);
                self.push_entire_state(mem);
            }
            self.cwai = false;
            self.reg.cc.set_irq_inhibit(true);
            self.reg.pc = mem.read_word(VEC_IRQ);
            self.cycles += 19;
            return true;
        }

        false
    }

    // ---- stack helpers ----

    /// Push a byte onto the hardware stack (S).
    pub(super) fn push_byte_s(&mut self, mem: &mut impl Memory, val: u8) {
        self.reg.s = self.reg.s.wrapping_sub(1);
        mem.write(self.reg.s, val);
    }

    /// Push a 16-bit word onto the hardware stack (S), low byte first.
    pub(super) fn push_word_s(&mut self, mem: &mut impl Memory, val: u16) {
        self.reg.s = self.reg.s.wrapping_sub(2);
        mem.write_word(self.reg.s, val);
    }

    /// Pull a byte from the hardware stack (S).
    pub(super) fn pull_byte_s(&mut self, mem: &mut impl Memory) -> u8 {
        let val = mem.read(self.reg.s);
        self.reg.s = self.reg.s.wrapping_add(1);
        val
    }

    /// Pull a 16-bit word from the hardware stack (S).
    pub(super) fn pull_word_s(&mut self, mem: &mut impl Memory) -> u16 {
        let val = mem.read_word(self.reg.s);
        self.reg.s = self.reg.s.wrapping_add(2);
        val
    }

    /// Push a byte onto the user stack (U).
    pub(super) fn push_byte_u(&mut self, mem: &mut impl Memory, val: u8) {
        self.reg.u = self.reg.u.wrapping_sub(1);
        mem.write(self.reg.u, val);
    }

    /// Push a 16-bit word onto the user stack (U).
    pub(super) fn push_word_u(&mut self, mem: &mut impl Memory, val: u16) {
        self.reg.u = self.reg.u.wrapping_sub(2);
        mem.write_word(self.reg.u, val);
    }

    /// Pull a byte from the user stack (U).
    pub(super) fn pull_byte_u(&mut self, mem: &mut impl Memory) -> u8 {
        let val = mem.read(self.reg.u);
        self.reg.u = self.reg.u.wrapping_add(1);
        val
    }

    /// Pull a 16-bit word from the user stack (U).
    pub(super) fn pull_word_u(&mut self, mem: &mut impl Memory) -> u16 {
        let val = mem.read_word(self.reg.u);
        self.reg.u = self.reg.u.wrapping_add(2);
        val
    }

    /// Push the entire register state onto S (used by NMI, IRQ, SWI).
    /// Order: CC, A, B, DP, X, Y, U, PC (PC pushed first = highest address).
    pub(super) fn push_entire_state(&mut self, mem: &mut impl Memory) {
        self.push_word_s(mem, self.reg.pc);
        self.push_word_s(mem, self.reg.u);
        self.push_word_s(mem, self.reg.y);
        self.push_word_s(mem, self.reg.x);
        self.push_byte_s(mem, self.reg.dp);
        self.push_byte_s(mem, self.reg.b());
        self.push_byte_s(mem, self.reg.a());
        self.push_byte_s(mem, self.reg.cc.to_byte());
    }

    // ---- instruction fetch helpers ----

    /// Fetch a byte from [PC] and advance PC.
    pub(super) fn fetch_byte(&mut self, mem: &mut impl Memory) -> u8 {
        let val = mem.read(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        val
    }

    /// Fetch a big-endian 16-bit word from [PC] and advance PC by 2.
    pub(super) fn fetch_word(&mut self, mem: &mut impl Memory) -> u16 {
        let val = mem.read_word(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(2);
        val
    }

    // ---- addressing mode helpers ----

    /// Direct addressing: DP:fetch_byte → effective address.
    pub(super) fn addr_direct(&mut self, mem: &mut impl Memory) -> u16 {
        let lo = self.fetch_byte(mem) as u16;
        ((self.reg.dp as u16) << 8) | lo
    }

    /// Extended addressing: fetch 16-bit absolute address.
    pub(super) fn addr_extended(&mut self, mem: &mut impl Memory) -> u16 {
        self.fetch_word(mem)
    }

    /// Indexed addressing: decode post-byte and return (effective_address, extra_cycles).
    pub(super) fn addr_indexed(&mut self, mem: &mut impl Memory) -> (u16, u8) {
        crate::addressing::indexed(self, mem)
    }

    /// Relative 8-bit: signed offset from current PC.
    pub(super) fn addr_relative8(&mut self, mem: &mut impl Memory) -> u16 {
        let offset = self.fetch_byte(mem) as i8 as i16 as u16;
        self.reg.pc.wrapping_add(offset)
    }

    /// Relative 16-bit: signed offset from current PC.
    pub(super) fn addr_relative16(&mut self, mem: &mut impl Memory) -> u16 {
        let offset = self.fetch_word(mem);
        self.reg.pc.wrapping_add(offset)
    }

    /// Arm the NMI (called when S is first written to).
    pub(super) fn arm_nmi(&mut self) {
        self.nmi_armed = true;
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// RegistersMut — RAII guard for mutable register access
// ---------------------------------------------------------------------------

/// RAII guard returned by [`Cpu::registers_mut`].
///
/// Dereferences to [`Registers`], giving full read/write access to all
/// programmer-visible registers. On drop the guard arms the NMI if the
/// hardware stack pointer (S) changed during the guard's lifetime.
pub struct RegistersMut<'a> {
    cpu: &'a mut Cpu,
    prev_s: u16,
}

impl std::ops::Deref for RegistersMut<'_> {
    type Target = Registers;
    fn deref(&self) -> &Registers {
        &self.cpu.reg
    }
}

impl std::ops::DerefMut for RegistersMut<'_> {
    fn deref_mut(&mut self) -> &mut Registers {
        &mut self.cpu.reg
    }
}

impl Drop for RegistersMut<'_> {
    fn drop(&mut self) {
        if self.cpu.reg.s != self.prev_s {
            self.cpu.nmi_armed = true;
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} cyc={}", self.reg, self.cycles)
    }
}

use std::fmt;
