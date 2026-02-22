# mc6809-core

mc6809-core is a small, focused Rust library implementing the Motorola 6809 CPU for use in emulators, tools, and testing harnesses. It provides a `Cpu` implementation capable of executing 6809 instructions against any memory system that implements the `Bus` trait.

Features
- Accurate 6809 instruction execution and addressing modes
- Modular design: separate `alu`, `addressing`, `bus`, `registers` modules
- A `Bus` trait for pluggable memory and I/O backends
- Lightweight API suitable for embedding in emulators, disassemblers, and debuggers

Public modules
- `addressing` — addressing mode helpers
- `alu` — arithmetic and logic operations
- `bus` — `Bus` trait for memory access
- `registers` — CPU register and status types

Quick example

```rust
use mc6809_core::{Cpu, Bus};

struct FlatRam([u8; 65536]);

impl Bus for FlatRam {
	fn read(&self, addr: u16) -> u8 { self.0[addr as usize] }
	fn write(&mut self, addr: u16, val: u8) { self.0[addr as usize] = val; }
}

let mut bus = FlatRam([0; 65536]);
// Place a reset vector pointing to 0x0400
bus.0[0xFFFE] = 0x04;
bus.0[0xFFFF] = 0x00;
// Place a NOP at 0x0400
bus.0[0x0400] = 0x12;

let mut cpu = Cpu::new();
cpu.reset(&mut bus);
assert_eq!(cpu.reg.pc, 0x0400);
cpu.step(&mut bus);
assert_eq!(cpu.reg.pc, 0x0401);
```

Building and testing
- Build: `cargo build` (run in the workspace or this crate)
- Test: `cargo test`

Contributing
- Contributions, bug reports and improvements are welcome — open an issue or pull request in the main repository.
