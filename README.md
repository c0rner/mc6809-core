# mc6809-core

mc6809-core is a small, focused Rust library implementing the Motorola 6809 CPU for use in emulators, tools, and testing harnesses. It provides a `Cpu` implementation capable of executing 6809 instructions against any memory system that implements the `Memory` trait.

Features
- Accurate 6809 instruction execution and addressing modes
- A `Memory` trait for pluggable memory and I/O backends
- A `Clocked` trait for peripheral timing and interrupt signal delivery, kept separate from memory access
- Lightweight API suitable for embedding in emulators, disassemblers, and debuggers

Quick example
```rust
use mc6809_core::{Cpu, Memory};

struct FlatRam([u8; 65536]);

impl Memory for FlatRam {
    fn read(&mut self, addr: u16) -> u8 { self.0[addr as usize] }
    fn write(&mut self, addr: u16, val: u8) { self.0[addr as usize] = val; }
}

let mut mem = FlatRam([0; 65536]);
// Place a reset vector pointing to 0x0400
mem.0[0xFFFE] = 0x04;
mem.0[0xFFFF] = 0x00;
// Place a NOP at 0x0400
mem.0[0x0400] = 0x12;

let mut cpu = Cpu::new();
cpu.reset(&mut mem);
assert_eq!(cpu.registers().pc, 0x0400);
cpu.step(&mut mem);
assert_eq!(cpu.registers().pc, 0x0401);
```

Systems with peripherals implement both traits on the same type. The `Memory` trait is
passed to the CPU, while `Clocked::tick` is called separately by the host loop.
The preferred way to feed signals into the CPU is `Cpu::apply_signals`, which handles
NMI edge detection internally:

```rust
let mut prev_signals = BusSignals::default();

loop {
    let cycles = cpu.step(&mut system);
    let signals = system.tick(cycles);

    // RESET is handled before apply_signals so a held-RESET pin keeps the
    // CPU quiescent and is not confused with a regular interrupt transition.
    if signals.contains(BusSignals::RESET) {
        cpu.reset(&mut system);
        prev_signals = BusSignals::default();
        continue;
    }

    // Only call into the CPU when something actually changed on the bus.
    if signals != prev_signals {
        cpu.apply_signals(signals, prev_signals);
        prev_signals = signals;
    }

    if cpu.halted() { break; }
    if cpu.illegal() {
        // Optional host policy: stop, log, or ignore.
    }
}
```

The individual `cpu.set_irq()`, `cpu.set_firq()`, and `cpu.trigger_nmi()` methods are
still available for simpler setups where NMI edge detection is handled by the caller.

Behavior notes
- Illegal opcodes set `Cpu::illegal()` but do not halt the CPU. This matches the default 6809-style execution model and leaves trap/stop policy to the host.
- Repeated page-prefix chaining (`0x10`/`0x11` after an initial page prefix) is intentionally not implemented. Only a single leading page prefix is recognised.

Building and testing
- Build: `cargo build` (run in the workspace or this crate)
- Test: `cargo test`

Contributing
- Contributions, bug reports and improvements are welcome — open an issue or pull request in the main repository.
