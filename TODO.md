## Bug fixes
- [x] Write the low byte first on 16-bit pushes to either stack (`S` or `U`).
  Debuggers and other `Memory` implementations observe the order of writes.
  Regression case: `PSHS X` with `X = $1234` and `S = $1000` must write `$34`
  to `$0FFF` first, then `$12` to `$0FFE`.

## Minor features
- [ ] Add counter for missed interrupts (IRQ, FIRQ)

## Implement undocumented op codes
Source: https://github.com/hoglet67/6809Decoder/wiki/Undocumented-6809-Behaviours

** Undocumented Base-page Opcodes **
- [x] XHCF
- [x] X18
- [x] NOP
- [x] XANDCC
- [x] XRES
- [x] NEG
- [x] XNC
- [x] LSR
- [x] XDEC
- [x] XCLRA/B
- [ ] Store Immediate
- [ ] Flags for Unprefixed Store Immediate
** Undocumented Page-2 Opcodes **
- [x] XLBRA
- [x] XSWI2
- [x] XADDD
- [ ] Page-2 Store Immediate
** Undocumented Page-3 Opcodes **
- [x] XFIRQ
- [x] XADDU
- [ ] Page-3 Store Immediate
- [ ] Undefined values of the Half-Carry Flag
- [ ] Undefined values of the Overflow Flag
- [ ] Multiple Prefixes (this one is fun)