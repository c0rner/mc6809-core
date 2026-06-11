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

mod common;
use common::{HaltReason, TestHarness, run_to_halt};

use mc6809_core::Cpu;

/// Pre-assembled test binary.  Rebuild with:
///   asm6809 -B -o asm/mc6809_test.bin asm/mc6809_test.asm
const BINARY: &[u8] = include_bytes!("../asm/mc6809_test.bin");

/// Load the integration-test binary into a fresh bus, run to completion,
/// and assert that every test passed.
#[test]
fn mc6809_integration() {
    let mut system = TestHarness::new();
    system.load(BINARY, 0x0000);

    let mut cpu = Cpu::new();
    cpu.reset(&mut system);

    match run_to_halt(&mut cpu, &mut system) {
        HaltReason::Pass(_) => {}
        HaltReason::Fail(test_num) => {
            panic!(
                "Assembly test {:02} FAILED  (PC={:#06X}, cycles={})",
                test_num,
                cpu.registers().pc,
                cpu.cycles(),
            );
        }
        HaltReason::CycleLimit => {
            panic!(
                "Cycle limit exceeded without pass/fail signal  (PC={:#06X})",
                cpu.registers().pc,
            );
        }
    }
}

#[test]
fn halt_on_invalid_opcode() {
    let mut system = TestHarness::new();
    // 0x87 is an illegal opcode
    system.load(&[0x87], 0x1000);
    // Set reset vector to 0x1000
    system.load(&[0x10, 0x00], 0xFFFE);

    // Run without halt_on_invalid
    let mut cpu = Cpu::new();
    cpu.reset(&mut system);
    cpu.step(&mut system);
    assert!(cpu.illegal());
    assert!(!cpu.halted());

    // Run with halt_on_invalid
    let mut cpu2 = Cpu::new().with_halt_on_invalid(true);
    cpu2.reset(&mut system);
    cpu2.step(&mut system);
    assert!(cpu2.illegal());
    assert!(cpu2.halted());
}
