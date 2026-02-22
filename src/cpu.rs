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

use crate::bus::Bus;
use crate::registers::Registers;

mod opcodes;

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
    pub reg: Registers,
    /// Total elapsed cycles since reset.
    pub cycles: u64,
    /// CPU is halted (hit illegal opcode or RESET instruction).
    pub halted: bool,
    /// CPU encountered an illegal opcode (invalid in current state).
    pub illegal: bool,

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
    pub fn reset(&mut self, bus: &impl Bus) {
        self.reg = Registers::new();
        self.reg.cc.set_irq_inhibit(true);
        self.reg.cc.set_firq_inhibit(true);
        self.reg.pc = bus.read_word(VEC_RESET);
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
    pub fn step(&mut self, bus: &mut impl Bus) -> u64 {
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
        if self.check_interrupts(bus) {
            return self.cycles - start_cycles;
        }

        // Fetch and execute one instruction
        let opcode = self.fetch_byte(bus);
        self.execute(bus, opcode);

        self.cycles - start_cycles
    }

    /// Run until at least `cycle_budget` cycles have been consumed.
    pub fn run(&mut self, bus: &mut impl Bus, cycle_budget: u64) -> u64 {
        let target = self.cycles + cycle_budget;
        while self.cycles < target && !self.halted {
            self.step(bus);
        }
        self.cycles - (target - cycle_budget)
    }

    // ---- interrupt logic ----

    fn check_interrupts(&mut self, bus: &mut impl Bus) -> bool {
        // NMI (edge-triggered, highest priority)
        if self.nmi_pending {
            self.nmi_pending = false;
            if !self.cwai {
                self.reg.cc.set_entire(true);
                self.push_entire_state(bus);
            }
            self.cwai = false;
            self.reg.cc.set_irq_inhibit(true);
            self.reg.cc.set_firq_inhibit(true);
            self.reg.pc = bus.read_word(VEC_NMI);
            self.cycles += 19;
            return true;
        }

        // FIRQ (level-triggered)
        if self.firq_line && !self.reg.cc.firq_inhibit() {
            if !self.cwai {
                self.reg.cc.set_entire(false);
                self.push_word_s(bus, self.reg.pc);
                self.push_byte_s(bus, self.reg.cc.to_byte());
            }
            self.cwai = false;
            self.reg.cc.set_irq_inhibit(true);
            self.reg.cc.set_firq_inhibit(true);
            self.reg.pc = bus.read_word(VEC_FIRQ);
            self.cycles += 10;
            return true;
        }

        // IRQ (level-triggered)
        if self.irq_line && !self.reg.cc.irq_inhibit() {
            if !self.cwai {
                self.reg.cc.set_entire(true);
                self.push_entire_state(bus);
            }
            self.cwai = false;
            self.reg.cc.set_irq_inhibit(true);
            self.reg.pc = bus.read_word(VEC_IRQ);
            self.cycles += 19;
            return true;
        }

        false
    }

    // ---- stack helpers ----

    /// Push a byte onto the hardware stack (S).
    pub(crate) fn push_byte_s(&mut self, bus: &mut impl Bus, val: u8) {
        self.reg.s = self.reg.s.wrapping_sub(1);
        bus.write(self.reg.s, val);
    }

    /// Push a 16-bit word onto the hardware stack (S), high byte first.
    pub(crate) fn push_word_s(&mut self, bus: &mut impl Bus, val: u16) {
        self.push_byte_s(bus, val as u8); // low byte pushed first (ends at higher address)
        self.push_byte_s(bus, (val >> 8) as u8);
    }

    /// Pull a byte from the hardware stack (S).
    pub(crate) fn pull_byte_s(&mut self, bus: &impl Bus) -> u8 {
        let val = bus.read(self.reg.s);
        self.reg.s = self.reg.s.wrapping_add(1);
        val
    }

    /// Pull a 16-bit word from the hardware stack (S).
    pub(crate) fn pull_word_s(&mut self, bus: &impl Bus) -> u16 {
        let hi = self.pull_byte_s(bus) as u16;
        let lo = self.pull_byte_s(bus) as u16;
        (hi << 8) | lo
    }

    /// Push a byte onto the user stack (U).
    pub(crate) fn push_byte_u(&mut self, bus: &mut impl Bus, val: u8) {
        self.reg.u = self.reg.u.wrapping_sub(1);
        bus.write(self.reg.u, val);
    }

    /// Push a 16-bit word onto the user stack (U).
    pub(crate) fn push_word_u(&mut self, bus: &mut impl Bus, val: u16) {
        self.push_byte_u(bus, val as u8);
        self.push_byte_u(bus, (val >> 8) as u8);
    }

    /// Pull a byte from the user stack (U).
    pub(crate) fn pull_byte_u(&mut self, bus: &impl Bus) -> u8 {
        let val = bus.read(self.reg.u);
        self.reg.u = self.reg.u.wrapping_add(1);
        val
    }

    /// Pull a 16-bit word from the user stack (U).
    pub(crate) fn pull_word_u(&mut self, bus: &impl Bus) -> u16 {
        let hi = self.pull_byte_u(bus) as u16;
        let lo = self.pull_byte_u(bus) as u16;
        (hi << 8) | lo
    }

    /// Push the entire register state onto S (used by NMI, IRQ, SWI).
    /// Order: CC, A, B, DP, X, Y, U, PC (PC pushed first = highest address).
    pub(crate) fn push_entire_state(&mut self, bus: &mut impl Bus) {
        self.push_word_s(bus, self.reg.pc);
        self.push_word_s(bus, self.reg.u);
        self.push_word_s(bus, self.reg.y);
        self.push_word_s(bus, self.reg.x);
        self.push_byte_s(bus, self.reg.dp);
        self.push_byte_s(bus, self.reg.b());
        self.push_byte_s(bus, self.reg.a());
        self.push_byte_s(bus, self.reg.cc.to_byte());
    }

    /// Pull the entire register state from S (E flag was set).
    #[allow(dead_code)]
    pub(crate) fn pull_entire_state(&mut self, bus: &impl Bus) {
        let cc = self.pull_byte_s(bus);
        self.reg.cc = crate::registers::ConditionCodes::from_byte(cc);
        let a = self.pull_byte_s(bus);
        self.reg.set_a(a);
        let b = self.pull_byte_s(bus);
        self.reg.set_b(b);
        self.reg.dp = self.pull_byte_s(bus);
        self.reg.x = self.pull_word_s(bus);
        self.reg.y = self.pull_word_s(bus);
        self.reg.u = self.pull_word_s(bus);
        self.reg.pc = self.pull_word_s(bus);
    }

    // ---- instruction fetch helpers ----

    /// Fetch a byte from [PC] and advance PC.
    pub(crate) fn fetch_byte(&mut self, bus: &impl Bus) -> u8 {
        let val = bus.read(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        val
    }

    /// Fetch a big-endian 16-bit word from [PC] and advance PC by 2.
    pub(crate) fn fetch_word(&mut self, bus: &impl Bus) -> u16 {
        let hi = self.fetch_byte(bus) as u16;
        let lo = self.fetch_byte(bus) as u16;
        (hi << 8) | lo
    }

    // ---- addressing mode helpers ----

    /// Direct addressing: DP:fetch_byte → effective address.
    pub(crate) fn addr_direct(&mut self, bus: &impl Bus) -> u16 {
        let lo = self.fetch_byte(bus) as u16;
        ((self.reg.dp as u16) << 8) | lo
    }

    /// Extended addressing: fetch 16-bit absolute address.
    pub(crate) fn addr_extended(&mut self, bus: &impl Bus) -> u16 {
        self.fetch_word(bus)
    }

    /// Indexed addressing: decode post-byte and return (effective_address, extra_cycles).
    pub(crate) fn addr_indexed(&mut self, bus: &impl Bus) -> (u16, u8) {
        crate::addressing::indexed(self, bus)
    }

    /// Relative 8-bit: signed offset from current PC.
    pub(crate) fn addr_relative8(&mut self, bus: &impl Bus) -> u16 {
        let offset = self.fetch_byte(bus) as i8 as i16 as u16;
        self.reg.pc.wrapping_add(offset)
    }

    /// Relative 16-bit: signed offset from current PC.
    pub(crate) fn addr_relative16(&mut self, bus: &impl Bus) -> u16 {
        let offset = self.fetch_word(bus);
        self.reg.pc.wrapping_add(offset)
    }

    /// Arm the NMI (called when S is first written to).
    pub(crate) fn arm_nmi(&mut self) {
        self.nmi_armed = true;
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} cyc={}", self.reg, self.cycles)
    }
}

use std::fmt;
