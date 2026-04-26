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

use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// Interrupt and control signals returned by [`Clocked::tick`].
///
/// Each flag corresponds to a physical input pin on the 6809 CPU.
/// Signals can be combined with `|` and tested with [`contains`](Self::contains).
/// The default is all signals de-asserted.
///
/// # Example
/// ```
/// use mc6809_core::BusSignals;
///
/// let signals = BusSignals::IRQ | BusSignals::NMI;
/// assert!(signals.contains(BusSignals::IRQ));
/// assert!(signals.contains(BusSignals::NMI));
/// ```
#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[must_use]
pub struct BusSignals(u8);

impl BusSignals {
    /// NMI request (edge-triggered — set to trigger once).
    pub const NMI: Self = Self(0x01);
    /// FIRQ line state (active = asserted, level-triggered).
    pub const FIRQ: Self = Self(0x02);
    /// IRQ line state (active = asserted, level-triggered).
    pub const IRQ: Self = Self(0x04);
    /// RESET pin asserted — the host loop should call [`Cpu::reset`](crate::Cpu::reset).
    pub const RESET: Self = Self(0x08);

    /// Returns `true` if all bits in `other` are set in `self`.
    #[inline]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for BusSignals {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for BusSignals {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for BusSignals {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for BusSignals {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitXor for BusSignals {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for BusSignals {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for BusSignals {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl fmt::Debug for BusSignals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const FLAGS: &[(&str, BusSignals)] = &[
            ("NMI", BusSignals::NMI),
            ("FIRQ", BusSignals::FIRQ),
            ("IRQ", BusSignals::IRQ),
            ("RESET", BusSignals::RESET),
        ];
        write!(f, "BusSignals(")?;
        let mut first = true;
        for (name, flag) in FLAGS {
            if self.contains(*flag) {
                if !first {
                    write!(f, " | ")?;
                }
                write!(f, "{name}")?;
                first = false;
            }
        }
        if first {
            write!(f, "empty")?;
        }
        write!(f, ")")
    }
}

///
/// Implement this trait for any peripheral that needs to track CPU cycles and
/// signal interrupts. The host loop calls [`tick`](Clocked::tick) after each CPU
/// step (or batch of steps), then feeds the returned [`BusSignals`] into the
/// CPU via [`Cpu::set_irq`](crate::Cpu::set_irq) etc.
///
/// The trait is intentionally thin so that implementations can be layered.
/// A debug or tracing system can wrap an inner `Clocked` implementation, forwarding
/// `tick()` calls while intercepting or logging signals — without requiring
/// changes to the wrapped implementation or the host loop.
///
/// When multiple peripherals share a bus, OR their signals together:
/// ```ignore
/// let mut signals = BusSignals::default();
/// for p in &mut peripherals {
///     signals |= p.tick(cycles);
/// }
/// ```
///
/// The default implementation is a no-op returning all signals inactive,
/// suitable for simple test systems with no peripherals.
pub trait Clocked {
    fn tick(&mut self, _cycles: u64) -> BusSignals {
        BusSignals::default()
    }
}
