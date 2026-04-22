# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-04-22

### Added
- New test harness allowing for run-time tests in sync with the emulator.

### Changed
- **BREAKING**: New `Memory` trait introduced for address/data bus access (`read`, `write`, `read_word`, `write_word`); `Bus` trait is now solely for peripheral timing via `tick()` and interrupt signal delivery. `Cpu::reset()`, `Cpu::step()`, and `Cpu::run()` now accept `impl Memory` instead of `impl Bus`.
- Migrate away from the `mod.rs` module naming convention

## [0.1.2] - 2026-04-06

### Added
- Guaranteed stable memory layout for CPU registers and flags to support JIT clients
- `instruction_cycles()` function returning the cycle count for instructions
- Tests asserting memory layout contract and `instruction_cycles()` behaviour

## [0.1.1] - 2026-04-02

### Added
- `Bus::tick()` method allowing peripherals to signal the CPU
- Support for most undocumented opcodes (page 0 and page 1)
- Tests for a subset of the undocumented opcodes
- Tests for full `RTI` instruction
- `TODO.md` tracking known gaps and planned work

### Changed
- `Bus::read()` now takes `&mut self` to allow stateful bus implementations
- Collapsed `set_negative()` / `set_zero()` helpers into `set_nz8()` / `set_nz16()`
- Illegal instructions on page 0/1 now set the illegal flag
- Reduced visibility of internal helpers from `pub(crate)` to `pub(super)`

### Fixed
- Fixed `X18` instruction incorrectly advancing the program counter by one extra byte
- Fixed `XADDU` for indexed and extended addressing modes
- Fixed rustdoc warnings

### Documentation
- Renamed example from `example` to `flat_bus`
- Added comments to page 1 undocumented opcodes

## [0.1.0] - 2026-02-22

### Added
- Initial release of the MC6809 CPU emulator core
- `Cpu` struct with `reset()` and `step()` for instruction execution
- `Bus` trait for pluggable memory and I/O backends
- `alu`, `addressing`, `bus`, and `registers` public modules

[Unreleased]: https://github.com/c0rner/mc6809-core/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/c0rner/mc6809-core/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/c0rner/mc6809-core/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/c0rner/mc6809-core/releases/tag/v0.1.0
