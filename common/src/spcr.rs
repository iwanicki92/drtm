// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The ACPI SPCR table, which names the console UART, decoded into a sink
//! the Secure Loader can write to (_Microsoft Serial Port Console
//! Redirection Table_, and ACPI 6.5 section 5.2.3.2 for its address).

use crate::uart::{Space, Uart};

const INTERFACE_TYPE: usize = 36;
const ADDRESS_SPACE: usize = 40;
const BIT_WIDTH: usize = 41;
const ACCESS_SIZE: usize = 43;
const ADDRESS: usize = 44;
const END: usize = ADDRESS + 8;

/// The 16550 variants, as the DBG2 serial subtypes number them: full, the
/// 16450 subset, and one whose register layout the address describes.
const COMPATIBLE: [u8; 3] = [0x00, 0x01, 0x12];

const SPACE_MEMORY: u8 = 0;
const SPACE_IO: u8 = 1;

/// Why the table's UART is not one the Secure Loader can write to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unusable {
    /// Shorter than the fields read here.
    Truncated,
    /// Not a 16550, by its interface type.
    Interface(u8),
    /// Neither memory nor I/O space.
    Space(u8),
    /// MMIO above 4 GiB, which the Secure Loader does not map.
    Above4GiB(u64),
    /// Registers neither 8 nor 32 bits wide.
    Width(u8),
    /// An I/O port beyond 16 bits.
    Port(u64),
}

/// The UART a whole SPCR table describes.
pub fn decode(table: &[u8]) -> Result<Uart, Unusable> {
    let field = |at: usize| table.get(at).copied().ok_or(Unusable::Truncated);
    let interface = field(INTERFACE_TYPE)?;
    if !COMPATIBLE.contains(&interface) {
        return Err(Unusable::Interface(interface));
    }
    let address = table
        .get(ADDRESS..END)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(Unusable::Truncated)?;
    // The access size says how registers are read, 1 for bytes and 3 for
    // dwords. Older tables leave it 0 and say the same with the bit width.
    let width = match (field(ACCESS_SIZE)?, field(BIT_WIDTH)?) {
        (1, _) | (0, 8) => 1,
        (3, _) | (0, 32) => 4,
        (0, bits) => return Err(Unusable::Width(bits)),
        (size, _) => return Err(Unusable::Width(8 << (size - 1).min(3))),
    };
    let space = match field(ADDRESS_SPACE)? {
        SPACE_MEMORY if address.saturating_add(8 * u64::from(width)) > 1 << 32 => {
            return Err(Unusable::Above4GiB(address));
        }
        SPACE_MEMORY => Space::Mmio,
        SPACE_IO if address > 0xFFFF => return Err(Unusable::Port(address)),
        SPACE_IO => Space::Io,
        other => return Err(Unusable::Space(other)),
    };
    Ok(Uart {
        space,
        base: address,
        width,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(interface: u8, space: u8, bits: u8, access: u8, address: u64) -> [u8; 80] {
        let mut table = [0; 80];
        table[..4].copy_from_slice(b"SPCR");
        table[INTERFACE_TYPE] = interface;
        table[ADDRESS_SPACE] = space;
        table[BIT_WIDTH] = bits;
        table[ACCESS_SIZE] = access;
        table[ADDRESS..END].copy_from_slice(&address.to_le_bytes());
        table
    }

    #[test]
    fn io_port() {
        assert_eq!(
            decode(&table(0, SPACE_IO, 8, 1, 0x2F8)),
            Ok(Uart {
                space: Space::Io,
                base: 0x2F8,
                width: 1
            })
        );
    }

    #[test]
    fn mmio_below_4gib() {
        assert_eq!(
            decode(&table(0x12, SPACE_MEMORY, 32, 3, 0xFEDC_9000)),
            Ok(Uart {
                space: Space::Mmio,
                base: 0xFEDC_9000,
                width: 4
            })
        );
        // Width from the bit width alone, as older tables give it.
        assert_eq!(
            decode(&table(0, SPACE_MEMORY, 8, 0, 0xFEDC_9000)).map(|u| u.width),
            Ok(1)
        );
    }

    #[test]
    fn refused() {
        assert_eq!(
            decode(&table(0, SPACE_MEMORY, 32, 3, 0x1_0000_0000)),
            Err(Unusable::Above4GiB(0x1_0000_0000))
        );
        assert_eq!(
            decode(&table(0, SPACE_MEMORY, 16, 2, 0xFEDC_9000)),
            Err(Unusable::Width(16))
        );
        assert_eq!(
            decode(&table(0, SPACE_MEMORY, 16, 0, 0xFEDC_9000)),
            Err(Unusable::Width(16))
        );
        // A PL011, which is what ARM servers name here.
        assert_eq!(
            decode(&table(0x03, SPACE_MEMORY, 32, 3, 0x900_0000)),
            Err(Unusable::Interface(0x03))
        );
        assert_eq!(decode(&table(0, 2, 8, 1, 0x3F8)), Err(Unusable::Space(2)));
        assert_eq!(
            decode(&table(0, SPACE_IO, 8, 1, 0x1_0000)),
            Err(Unusable::Port(0x1_0000))
        );
        assert_eq!(decode(&[0; 40]), Err(Unusable::Truncated));
    }
}
