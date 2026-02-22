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

//! Unit tests for ALU operations.

use crate::alu;
use crate::registers::ConditionCodes;

#[test]
fn add8_basic() {
    let mut cc = ConditionCodes::new();
    let r = alu::add8(0x10, 0x20, &mut cc);
    assert_eq!(r, 0x30);
    assert!(!cc.carry());
    assert!(!cc.zero());
    assert!(!cc.negative());
    assert!(!cc.overflow());
}

#[test]
fn add8_carry() {
    let mut cc = ConditionCodes::new();
    let r = alu::add8(0xFF, 0x01, &mut cc);
    assert_eq!(r, 0x00);
    assert!(cc.carry());
    assert!(cc.zero());
    assert!(!cc.negative());
}

#[test]
fn add8_overflow_positive() {
    // 0x7F + 0x01 = 0x80 → positive overflow
    let mut cc = ConditionCodes::new();
    let r = alu::add8(0x7F, 0x01, &mut cc);
    assert_eq!(r, 0x80);
    assert!(cc.overflow());
    assert!(cc.negative());
    assert!(!cc.carry());
}

#[test]
fn add8_half_carry() {
    let mut cc = ConditionCodes::new();
    alu::add8(0x0F, 0x01, &mut cc);
    assert!(cc.half_carry());
}

#[test]
fn sub8_basic() {
    let mut cc = ConditionCodes::new();
    let r = alu::sub8(0x30, 0x10, &mut cc);
    assert_eq!(r, 0x20);
    assert!(!cc.carry());
    assert!(!cc.zero());
    assert!(!cc.negative());
}

#[test]
fn sub8_borrow() {
    let mut cc = ConditionCodes::new();
    let r = alu::sub8(0x00, 0x01, &mut cc);
    assert_eq!(r, 0xFF);
    assert!(cc.carry()); // borrow
    assert!(cc.negative());
}

#[test]
fn sub8_overflow() {
    // 0x80 - 0x01 = 0x7F → overflow (negative to positive)
    let mut cc = ConditionCodes::new();
    let r = alu::sub8(0x80, 0x01, &mut cc);
    assert_eq!(r, 0x7F);
    assert!(cc.overflow());
    assert!(!cc.negative());
}

#[test]
fn neg8_zero() {
    let mut cc = ConditionCodes::new();
    let r = alu::neg8(0x00, &mut cc);
    assert_eq!(r, 0x00);
    assert!(!cc.carry());
    assert!(cc.zero());
}

#[test]
fn neg8_0x80() {
    let mut cc = ConditionCodes::new();
    let r = alu::neg8(0x80, &mut cc);
    assert_eq!(r, 0x80); // -128 negated wraps
    assert!(cc.overflow());
    assert!(cc.carry());
}

#[test]
fn neg8_positive() {
    let mut cc = ConditionCodes::new();
    let r = alu::neg8(0x01, &mut cc);
    assert_eq!(r, 0xFF);
    assert!(cc.carry());
    assert!(cc.negative());
}

#[test]
fn com8_basic() {
    let mut cc = ConditionCodes::new();
    let r = alu::com8(0x55, &mut cc);
    assert_eq!(r, 0xAA);
    assert!(cc.carry()); // COM always sets C
    assert!(!cc.overflow()); // COM always clears V
    assert!(cc.negative());
}

#[test]
fn inc8_overflow() {
    let mut cc = ConditionCodes::new();
    let r = alu::inc8(0x7F, &mut cc);
    assert_eq!(r, 0x80);
    assert!(cc.overflow());
    assert!(cc.negative());
}

#[test]
fn dec8_overflow() {
    let mut cc = ConditionCodes::new();
    let r = alu::dec8(0x80, &mut cc);
    assert_eq!(r, 0x7F);
    assert!(cc.overflow());
    assert!(!cc.negative());
}

#[test]
fn clr8_flags() {
    let mut cc = ConditionCodes::new();
    cc.set_carry(true);
    cc.set_negative(true);
    let r = alu::clr8(&mut cc);
    assert_eq!(r, 0x00);
    assert!(cc.zero());
    assert!(!cc.carry());
    assert!(!cc.negative());
    assert!(!cc.overflow());
}

#[test]
fn tst8_zero() {
    let mut cc = ConditionCodes::new();
    cc.set_carry(true);
    alu::tst8(0x00, &mut cc);
    assert!(cc.zero());
    assert!(!cc.negative());
    assert!(!cc.overflow());
    assert!(cc.carry()); // TST does not affect C
}

#[test]
fn tst8_negative() {
    let mut cc = ConditionCodes::new();
    alu::tst8(0x80, &mut cc);
    assert!(cc.negative());
    assert!(!cc.zero());
}

#[test]
fn asl8_carry() {
    let mut cc = ConditionCodes::new();
    let r = alu::asl8(0x80, &mut cc);
    assert_eq!(r, 0x00);
    assert!(cc.carry());
    assert!(cc.zero());
}

#[test]
fn lsr8_carry() {
    let mut cc = ConditionCodes::new();
    let r = alu::lsr8(0x01, &mut cc);
    assert_eq!(r, 0x00);
    assert!(cc.carry());
    assert!(!cc.negative()); // LSR always clears N
}

#[test]
fn asr8_preserves_sign() {
    let mut cc = ConditionCodes::new();
    let r = alu::asr8(0x80, &mut cc);
    assert_eq!(r, 0xC0); // bit 7 preserved
    assert!(!cc.carry());
    assert!(cc.negative());
}

#[test]
fn rol8_through_carry() {
    let mut cc = ConditionCodes::new();
    cc.set_carry(true);
    let r = alu::rol8(0x00, &mut cc);
    assert_eq!(r, 0x01); // old carry rotated in
    assert!(!cc.carry()); // bit 7 was 0
}

#[test]
fn ror8_through_carry() {
    let mut cc = ConditionCodes::new();
    cc.set_carry(true);
    let r = alu::ror8(0x00, &mut cc);
    assert_eq!(r, 0x80); // old carry → bit 7
    assert!(!cc.carry());
    assert!(cc.negative());
}

#[test]
fn and8_basic() {
    let mut cc = ConditionCodes::new();
    let r = alu::and8(0xFF, 0x0F, &mut cc);
    assert_eq!(r, 0x0F);
    assert!(!cc.overflow());
}

#[test]
fn or8_basic() {
    let mut cc = ConditionCodes::new();
    let r = alu::or8(0xF0, 0x0F, &mut cc);
    assert_eq!(r, 0xFF);
    assert!(cc.negative());
}

#[test]
fn eor8_basic() {
    let mut cc = ConditionCodes::new();
    let r = alu::eor8(0xFF, 0xFF, &mut cc);
    assert_eq!(r, 0x00);
    assert!(cc.zero());
    assert!(!cc.overflow());
}

#[test]
fn add16_carry() {
    let mut cc = ConditionCodes::new();
    let r = alu::add16(0xFFFF, 0x0001, &mut cc);
    assert_eq!(r, 0x0000);
    assert!(cc.carry());
    assert!(cc.zero());
}

#[test]
fn sub16_borrow() {
    let mut cc = ConditionCodes::new();
    let r = alu::sub16(0x0000, 0x0001, &mut cc);
    assert_eq!(r, 0xFFFF);
    assert!(cc.carry());
    assert!(cc.negative());
}

#[test]
fn mul_basic() {
    let mut cc = ConditionCodes::new();
    let r = alu::mul(10, 20, &mut cc);
    assert_eq!(r, 200);
    assert!(!cc.zero());
    assert!(cc.carry()); // bit 7 of low byte: 200 = 0xC8, bit 7 set
}

#[test]
fn mul_zero() {
    let mut cc = ConditionCodes::new();
    let r = alu::mul(0, 100, &mut cc);
    assert_eq!(r, 0);
    assert!(cc.zero());
    assert!(!cc.carry());
}

#[test]
fn sex_positive() {
    let mut cc = ConditionCodes::new();
    let d = alu::sex(0x42, &mut cc);
    assert_eq!(d, 0x0042);
    assert!(!cc.negative());
    assert!(!cc.zero());
}

#[test]
fn sex_negative() {
    let mut cc = ConditionCodes::new();
    let d = alu::sex(0x80, &mut cc);
    assert_eq!(d, 0xFF80);
    assert!(cc.negative());
    assert!(!cc.zero());
}

#[test]
fn sex_zero() {
    let mut cc = ConditionCodes::new();
    let d = alu::sex(0x00, &mut cc);
    assert_eq!(d, 0x0000);
    assert!(!cc.negative());
    assert!(cc.zero());
}

#[test]
fn daa_basic() {
    // Simulate BCD: 0x15 + 0x27 = 0x3C in hex, DAA corrects to 0x42
    let mut cc = ConditionCodes::new();
    // First do the binary add
    let r = alu::add8(0x15, 0x27, &mut cc);
    assert_eq!(r, 0x3C);
    // Now apply DAA
    let r = alu::daa(r, &mut cc);
    assert_eq!(r, 0x42);
}

#[test]
fn adc8_with_carry() {
    let mut cc = ConditionCodes::new();
    cc.set_carry(true);
    let r = alu::adc8(0x10, 0x20, &mut cc);
    assert_eq!(r, 0x31); // 0x10 + 0x20 + 1
    assert!(!cc.carry());
}

#[test]
fn sbc8_with_carry() {
    let mut cc = ConditionCodes::new();
    cc.set_carry(true);
    let r = alu::sbc8(0x20, 0x10, &mut cc);
    assert_eq!(r, 0x0F); // 0x20 - 0x10 - 1
    assert!(!cc.carry());
}
