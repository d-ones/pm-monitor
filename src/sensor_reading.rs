use core::clone::Clone;
use core::iter::Iterator;
use core::marker::Copy;
use core::option::Option::{self, None, Some};
use core::prelude::rust_2024::derive;
use zerocopy::{byteorder::network_endian::U16, FromBytes};

// Zero copy struct used for converting sensor readings to Rust type

#[derive(FromBytes, Clone, Copy)]
#[repr(C, packed)]
pub struct PlantowerFrame {
    pub header: [u8; 2],
    pub length: U16,
    pub pm1_0_std: U16,
    pub pm2_5_std: U16,
    pub pm10_std: U16,
    pub pm1_0_atm: U16,
    pub pm2_5_atm: U16, // Data 5 High/Low
    pub pm10_atm: U16,
    pub counts_0_3: U16,
    pub counts_0_5: U16,
    pub counts_1_0: U16,
    pub counts_2_5: U16,
    pub counts_5_0: U16,
    pub counts_10_0: U16,
    pub reserved: [u8; 2],
    pub checksum: U16,
}

impl PlantowerFrame {
    /// returns a Frame only if the header and checksum are perfect.
    pub fn parse(buffer: &[u8; 32]) -> Option<Self> {
        // overlay the struct onto the bytes
        let frame = Self::read_from_bytes(buffer).ok()?;

        // Validate proprietary header ("BM")
        if frame.header != [0x42, 0x4D] {
            return None;
        }

        //  Validate Checksum
        let calc_sum = buffer
            .chunks_exact(30)
            .next() // Get the first (and only) chunk of 30
            .unwrap_or(&[])
            .iter()
            .map(|&b| b as u16)
            .sum::<u16>();

        if calc_sum == frame.checksum.get() {
            Some(frame)
        } else {
            None
        }
    }
}
