// yeah i would probably document this if understood anything going on here
// check out this if you'd like to learn more though: https://github.com/fsphil/ssdv

use log::debug;
use tinyvec::ArrayVec;

use crate::{JpegMarker, PacketType, Quality};

const PACKET_SIZE: usize = 256;
const HEADER_SIZE: usize = 16;
const CRC_SIZE: usize = 4;
const FEC_SIZE: usize = 32;
const PAYLOAD_SIZE: usize = PACKET_SIZE - HEADER_SIZE - CRC_SIZE - CRC_SIZE;
const CRCDATA_SIZE: usize = HEADER_SIZE + PAYLOAD_SIZE - 1;

/// APP0 header data
const APP0: [u8; 14] = [
    0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01, 0x00, 0x48, 0x00, 0x48, 0x00, 0x00,
];

/// SOS header data
const SOS: [u8; 10] = [0x03, 0x01, 0x00, 0x02, 0x11, 0x03, 0x11, 0x00, 0x3F, 0x00];

const STD_DQT0: [u8; 65] = [
    0x00, 0x10, 0x0C, 0x0C, 0x0E, 0x0C, 0x0A, 0x10, 0x0E, 0x0E, 0x0E, 0x12, 0x12, 0x10, 0x14, 0x18,
    0x28, 0x1A, 0x18, 0x16, 0x16, 0x18, 0x32, 0x24, 0x26, 0x1E, 0x28, 0x3A, 0x34, 0x3E, 0x3C, 0x3A,
    0x34, 0x38, 0x38, 0x40, 0x48, 0x5C, 0x4E, 0x40, 0x44, 0x58, 0x46, 0x38, 0x38, 0x50, 0x6E, 0x52,
    0x58, 0x60, 0x62, 0x68, 0x68, 0x68, 0x3E, 0x4E, 0x72, 0x7A, 0x70, 0x64, 0x78, 0x5C, 0x66, 0x68,
    0x64,
];

const STD_DQT1: [u8; 65] = [
    0x01, 0x12, 0x12, 0x12, 0x16, 0x16, 0x16, 0x30, 0x1A, 0x1A, 0x30, 0x64, 0x42, 0x38, 0x42, 0x64,
    0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64,
    0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64,
    0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64, 0x64,
    0x64,
];

/* Standard Huffman tables */
const STD_DHT00: [u8; 29] = [
    0x00, 0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
];

const STD_DHT01: [u8; 29] = [
    0x01, 0x00, 0x03, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
];

const STD_DHT10: [u8; 179] = [
    0x10, 0x00, 0x02, 0x01, 0x03, 0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01,
    0x7D, 0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61,
    0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1,
    0xF0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27,
    0x28, 0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
    0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
    0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88,
    0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6,
    0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4,
    0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1,
    0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7,
    0xF8, 0xF9, 0xFA,
];

const STD_DHT11: [u8; 179] = [
    0x11, 0x00, 0x02, 0x01, 0x02, 0x04, 0x04, 0x03, 0x04, 0x07, 0x05, 0x04, 0x04, 0x00, 0x01, 0x02,
    0x77, 0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61,
    0x71, 0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xA1, 0xB1, 0xC1, 0x09, 0x23, 0x33, 0x52,
    0xF0, 0x15, 0x62, 0x72, 0xD1, 0x0A, 0x16, 0x24, 0x34, 0xE1, 0x25, 0xF1, 0x17, 0x18, 0x19, 0x1A,
    0x26, 0x27, 0x28, 0x29, 0x2A, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47,
    0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67,
    0x68, 0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x82, 0x83, 0x84, 0x85, 0x86,
    0x87, 0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4,
    0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2,
    0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9,
    0xDA, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7,
    0xF8, 0xF9, 0xFA,
];

pub struct Encoder {
    state: State,
    callsign: u32,
    image_id: u8,
    quality: Quality,
    image: Box<dyn Iterator<Item = u8>>,
    dtbl0: [u8; 65],
    dtbl1: [u8; 65],
    outbits: u16,
    outlen: u8,
    out: ArrayVec<[u8; 256]>,
    out_stuff: bool,
    skip: usize,
    marker: u16,
    marker_len: u16,
    marker_data: Vec<u8>,
    dc: [isize; 3],
    adc: [isize; 3],
    acpart: u8,
    acrle: u8,
    accrle: u8,
    mcupart: u8,
    grayscale: bool,
    component: u8,
    ycparts: u8,
    workbits: u32,
    worklen: u8,
    needbits: u8,
    width: u16,
    height: u16,
    mcu_mode: u8,
    mcu_id: u16,
    mcu_count: u16,
}

impl Encoder {
    pub fn new<C, I>(callsign: C, image_id: u8, quality: Quality, image: I) -> Self
    where
        C: Into<ArrayVec<[u8; 6]>>,
        I: IntoIterator<Item = u8>,
        <I as IntoIterator>::IntoIter: 'static,
    {
        let dtbl0 = Self::load_standard_dqt(&STD_DQT0, quality);
        let dtbl1 = Self::load_standard_dqt(&STD_DQT1, quality);

        Self {
            state: State::Marker,
            callsign: Encoder::encode_callsign(&callsign.into()),
            image_id,
            quality,
            image: Box::new(image.into_iter()),
            dtbl0,
            dtbl1,
            outbits: 0,
            outlen: 0,
            out: ArrayVec::new(),
            out_stuff: false,
            skip: 0,
            marker: 0,
            marker_len: 0,
            marker_data: Vec::new(),
            dc: [0; 3],
            adc: [0; 3],
            acpart: 0,
            acrle: 0,
            accrle: 0,
            mcupart: 0,
            grayscale: false,
            component: 0,
            ycparts: 0,
            workbits: 0,
            worklen: 0,
            needbits: 0,
            width: 0,
            height: 0,
            mcu_count: 0,
            mcu_id: 0,
            mcu_mode: 0,
        }
    }

    fn encode_callsign(callsign: &[u8]) -> u32 {
        let mut x: u32 = 0;

        for c in callsign.into_iter().rev() {
            x *= 40;
            if *c >= b'A' && *c <= b'Z' {
                x += (c - b'A' + 14) as u32;
            } else if *c >= b'a' && *c <= b'z' {
                x += (c - b'a' + 14) as u32;
            } else if *c >= b'0' && *c <= b'9' {
                x += (c - b'0' + 1) as u32;
            }
        }

        return x;
    }

    fn load_standard_dqt(table: &[u8; 65], quality: Quality) -> [u8; 65] {
        let scale_factor = quality.scale_factor();
        let mut out: [u8; 65] = [0; 65];

        out[0] = table[0];
        for (i, b) in table.iter().skip(1).copied().enumerate() {
            let mut byte: u32 = (b as u32 * scale_factor as u32 + 50) / 100;
            byte = byte.clamp(1, 255);

            out[i] = byte as u8;
        }

        return out;
    }

    fn outbits(&mut self, bits: u16, len: u8) {
        if len > 0 {
            self.outbits <<= len;
            self.outbits |= bits & ((1 << len) - 1);
            self.outlen += len;
        }

        while self.outlen >= 8 && self.out.len() > 0 {
            let b = self.outbits >> (self.outlen - 8);

            self.out.push(b as u8);
            self.outlen -= 8;

            if self.out_stuff && b == 0xFF {
                self.outbits &= (1 << self.outlen) - 1;
                self.outlen += 1;
            }
        }
    }

    fn have_marker(&mut self) -> Result<(), EncodeError> {
        use JpegMarker as J;

        match self.marker.into() {
            J::Sof0 | J::Sos | J::Dri | J::Dht | J::Dqt => {
                self.marker_data.clear();
                self.state = State::MarkerData;
            }
            J::Sof2 => return Err(EncodeError::Progressive),
            J::Eoi => self.state = State::Eoi,
            J::Rst0 | J::Rst1 | J::Rst2 | J::Rst3 | J::Rst4 | J::Rst5 | J::Rst6 | J::Rst7 => {
                self.dc.fill(0);
                self.mcupart = 0;
                self.acpart = 0;
                self.component = 0;
                self.acrle = 0;
                self.accrle = 0;
                self.workbits = 0;
                self.worklen = 0;
                self.state = State::Huff;
            }
            _ => {
                self.skip = self.marker_len as usize;
                self.state = State::Marker;
            }
        }

        Ok(())
    }

    fn have_marker_data(&mut self) -> Result<(), EncodeError> {
        use JpegMarker as J;

        match self.marker.into() {
            J::Sof0 => {
                self.width = ((self.marker_data[3] as u16) << 8) | self.marker_data[4] as u16;
                self.height = ((self.marker_data[1] as u16) << 8) | self.marker_data[2] as u16;

                debug!("Precision: {}", self.marker_data[0]);
                debug!("Resolution: {}x{}", self.width, self.height);
                debug!("Components: {}", self.marker_data[5]);

                if self.marker_data[0] != 8 {
                    return Err(EncodeError::Precision);
                }

                if self.marker_data[5] != 1 && self.marker_data[5] != 3 {
                    return Err(EncodeError::Components);
                }

                if self.width > 4080 || self.height > 4080 {
                    return Err(EncodeError::TooLarge);
                }

                if (self.width & 0x0F != 0) || self.width & 0x0F != 0 {
                    return Err(EncodeError::InvalidResolution);
                }

                for i in 0..self.marker_data[5] {
                    let dq = &self.marker_data[(i as usize * 3 + 6)..];

                    debug!("DQT table for component {}: {}, Sampling factor: {}x{}", dq[0], dq[2], dq[1] & 0x0F, dq[1] >> 4);

                    // The first (Y) component must have a factor of 2x2, 2x1, 1x2, or 1x1
                    if i == 0 {
                        match dq[1] {
                            0x22 => {
                                self.mcu_mode = 0;
                                self.ycparts = 4;
                            }
                            0x12 => {
                                self.mcu_mode = 1;
                                self.ycparts = 2;
                            }
                            0x21 => {
                                self.mcu_mode = 2;
                                self.ycparts = 2;
                            }
                            0x11 => {
                                self.mcu_mode = 3;
                                self.ycparts = 1;
                            }
                            _ => return Err(EncodeError::SamplingFactor),
                        }
                    } else if dq[1] != 0x11 {
                        return Err(EncodeError::SamplingFactor);
                    }
                }

                if self.marker_data[5] == 1 {
                    self.grayscale = true;
                    self.mcu_mode = 2;
                    self.ycparts = 2;
                }

                let blocks: usize = match self.mcu_mode {
                    0 => (self.width >> 4) * (self.height >> 4),
                    1 => (self.width >> 4) * (self.height >> 3),
                    2 => (self.width >> 3) * (self.height >> 4),
                    3 => (self.width >> 3) * (self.height >> 3),
                    _ => unreachable!(),
                } as usize;

                debug!("MCU blocks: {blocks}");

                if blocks > 0xFFFF {
                    return Err(EncodeError::Blocks);
                }

                self.mcu_count = blocks as u16;
            }
            J::Sos => {
                debug!("Components: {}", self.marker_data[0]);

                if self.marker_data[0] != 1 && self.marker_data[0] != 3 {
                    return Err(EncodeError::Components);
                }

                for i in 0.. self.marker_data[0] {
                    let dh = &self.marker_data[i as usize * 2 + 1..];
                    debug!("Component {} DHT: {}", dh[0], dh[1]);
                }

                // Verify all of the DQT and DHT tables were loaded
                if self.sdq
            }
        }

        self.state = State::Marker;
        Ok(())
    }
}

impl Iterator for Encoder {
    type Item = Result<[u8; PACKET_SIZE], EncodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(b) = self.image.next() {
            if self.skip > 0 {
                self.skip -= 1;
                continue;
            }

            match self.state {
                State::Marker => {
                    self.marker = (self.marker << 8) | b as u16;

                    if self.marker == JpegMarker::Tem || (self.marker >= JpegMarker::Rst0 && self.marker <= JpegMarker::Eoi) {
                        self.marker_len = 0;
                        if let Err(err) = self.have_marker() {
                            return Some(Err(err))
                        }
                    }
                },
                State::MarkerLen => {
                    self.marker_len = (self.marker_len << 8) | b as u16;
                    self.needbits -= 8;

                    if self.needbits == 0 {
                        self.marker_len -= 2;
                        if let Err(err) = self.have_marker() {
                            return Some(Err(err))
                        }
                    }
                },
                State::MarkerData => {
                    self.marker_data.push(b);
                    if self.marker_data.len() == self.marker_len {
                        if let Err(err) = self.have_marker_data() {
                            return Some(Err(err));
                        }
                    }
                }
                State::Eoi => return None,
            }
        }

        return Some(Ok(self.out.into_inner()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum State {
    Marker,
    MarkerLen,
    MarkerData,
    Huff,
    Int,
    Eoi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodeError {
    Progressive,
    /// The image must have a precision of 8
    Precision,
    /// The image must have 1 or 3 components (Y'Cb'Cr)
    Components,
    /// Maximum image is 4080x4080
    TooLarge,
    /// The image dimensions mus be a multiple of 16
    InvalidResolution,
    // Component's sampling factor is not supported
    SamplingFactor,
    /// Maximum number of MCU blocks is 65535
    Blocks,
}
