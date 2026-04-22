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

use mc6809_core::{Cpu, Memory};

/// Write to this address to signal all tests passed (value = last test number).
pub const PASS_REG: u16 = 0xFF00;
/// Write to this address to signal a failing test (value = test number).
pub const FAIL_REG: u16 = 0xFF01;
/// Write non-zero to assert the IRQ line; write 0 to deassert.
const TRIGGER_IRQ: u16 = 0xFF02;
/// Write non-zero to assert the FIRQ line; write 0 to deassert.
const TRIGGER_FIRQ: u16 = 0xFF03;
/// Write any value to fire a single NMI pulse (edge-triggered).
const TRIGGER_NMI: u16 = 0xFF04;

const MAX_CYCLES: u64 = 10_000_000;

/// Reason the emulated test program stopped executing.
#[derive(Debug, PartialEq)]
pub enum HaltReason {
    /// All tests passed; value is whatever was written to PASS_REG.
    Pass(u8),
    /// A test failed; value is the failing test number written to FAIL_REG.
    Fail(u8),
    /// Cycle budget exhausted before the program signalled pass or fail.
    CycleLimit,
}

/// A 64 KB flat-RAM bus with memory-mapped I/O registers for test signalling
/// and interrupt line control.
///
/// - Write to `PASS_REG` ($FF00): signals all tests passed.
/// - Write to `FAIL_REG` ($FF01): signals that the written test number failed.
/// - Write non-zero to `TRIGGER_IRQ`  ($FF02): assert IRQ line; write 0 to deassert.
/// - Write non-zero to `TRIGGER_FIRQ` ($FF03): assert FIRQ line; write 0 to deassert.
/// - Write any value to `TRIGGER_NMI` ($FF04): fire one NMI pulse (edge-triggered).
///
/// All other addresses are plain RAM, so assembly code can write interrupt
/// vectors anywhere in the 64 KB address space.
pub struct TestHarness {
    mem: Box<[u8; 65536]>,
    halted: bool,
    halt_reason: Option<HaltReason>,
    irq_asserted: bool,
    firq_asserted: bool,
    nmi_pulse: bool,
}

impl TestHarness {
    pub fn new() -> Self {
        Self {
            mem: Box::new([0u8; 65536]),
            halted: false,
            halt_reason: None,
            irq_asserted: false,
            firq_asserted: false,
            nmi_pulse: false,
        }
    }

    /// Copy `data` into RAM starting at `base`.
    pub fn load(&mut self, data: &[u8], base: u16) {
        let start = base as usize;
        let end = start + data.len();
        assert!(end <= 65536, "binary exceeds 64 KB address space");
        self.mem[start..end].copy_from_slice(data);
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory for TestHarness {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            PASS_REG => {
                self.halt_reason = Some(HaltReason::Pass(val));
                self.halted = true;
            }
            FAIL_REG => {
                self.halt_reason = Some(HaltReason::Fail(val));
                self.halted = true;
            }
            TRIGGER_IRQ => {
                self.irq_asserted = val != 0;
            }
            TRIGGER_FIRQ => {
                self.firq_asserted = val != 0;
            }
            TRIGGER_NMI => {
                self.nmi_pulse = true;
            }
            _ => {
                self.mem[addr as usize] = val;
            }
        }
    }
}

/// Run the CPU until it signals pass/fail, hits an illegal opcode, or
/// exhausts the cycle budget.  Before each instruction, interrupt-line
/// state written by the test program via the trigger registers is applied
/// to the CPU so the 6809 sees them on the very next step.
pub fn run_to_halt(cpu: &mut Cpu, system: &mut TestHarness) -> HaltReason {
    while cpu.cycles < MAX_CYCLES {
        // Drive interrupt lines from the bus trigger registers.
        cpu.set_irq(system.irq_asserted);
        cpu.set_firq(system.firq_asserted);
        if system.nmi_pulse {
            system.nmi_pulse = false;
            cpu.trigger_nmi();
        }

        cpu.step(system);
        if system.halted {
            return system.halt_reason.take().unwrap();
        }
    }
    HaltReason::CycleLimit
}
