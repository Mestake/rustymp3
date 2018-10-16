#![allow(unused)]

use std::ops::*;

use result::*;
use utils::*;

/////////////////////////////////
//// START PUBLIC INTERFACE

/// Main API entry
pub struct Decoder<'a> {
    raw: &'a [u8],
}

/// Decoded (logical) frame
pub struct Frame {
    /// The decoded audio held by this frame. Channels are interleaved.
    pub data: Vec<i16>,
    /// This frame's sample rate in hertz.
    pub sample_rate: i32,
    /// The number of channels in this frame.
    pub channels: usize,
    /// MPEG layer used by this file.
    pub layer: usize,
    /// Current bitrate_kbps as of this frame, in kb/s.
    pub bitrate_kbps: i32,
}

impl<'a> Decoder<'a> {
    pub fn next_frame(&mut self) -> Option<Result<Frame>> {
        let header = self.read_header()?;
        // self.decode_frame(&Header);
        unimplemented!()
    }
}

impl<'a> From<&'a [u8]> for Decoder<'a> {
    fn from(raw: &'a [u8]) -> Self {
        Decoder { raw }
    }
}

//// ENG PUBLIC INTERFACE
/////////////////////////////////


#[derive(PartialEq)]
#[repr(u8)]
enum Mode {
    Stereo = 0,
    Joint,
    Dual,
    Single,
}

#[derive(PartialEq)]
#[repr(u8)]
enum Layer {
    Reserved,
    L1,
    L2,
    L3,
}

#[derive(PartialEq)]
#[repr(u8)]
#[allow(non_camel_case_types)]
enum Emphasis {
    None,
    Ms_50_15,
    Reserved,
    CCIT_J_17
}

/// Frame header
#[derive(Debug)]
pub struct Header(u32);

impl Header {
    fn from_raw(raw: u32) -> Result<Self> {
        use self::Error::*;

        let hdr = Header(raw);

        if hdr.is_mpeg25() {
            Err(UnsupportedFormat("MPEG-2.5"))
        }
        else if hdr.is_mpeg2() {
            Err(UnsupportedFormat("MPEG-2"))
        }
        else if hdr.layer() == Layer::Reserved {
            Err(InvalidHeader("layer 0x00 is reserved"))
        }
        else if hdr.bitrate_index() == 0b1111 {
            Err(InvalidHeader("bitrate index 0b1111 is invalid"))
        }
        else if hdr.sampling_rate_index() == 0b11 {
            Err(InvalidHeader("sampling rate index 0b11 is reserved"))
        }
        else if hdr.emphasis() == Emphasis::Reserved {
            Err(InvalidHeader("emphasis 0b10 is reserved"))
        }
        else {
            Ok(hdr)
        }
    }

    fn is_mpeg25(&self) -> bool {
        (self.0.bit_range(20..21)) == 0
    }

    fn is_mpeg2(&self) -> bool {
        self.0.bit_range(19..20) == 1
    }

    fn is_mpeg1(&self) -> bool {
        !self.is_mpeg25() && !self.is_mpeg2()
    }

    fn layer(&self) -> Layer {
        use self::Layer::*;

        match self.0.bit_range(17..19) {
            0 => Reserved,
            1 => L3,
            2 => L2,
            3 => L1,
            _ => unreachable!(),
        }
    }

    fn is_protected(&self) -> bool {
        self.0.bit_range(16..17) == 0
    }

    fn bitrate_index(&self) -> usize {
        self.0.bit_range(12..16) as usize
    }

    fn bitrate_kbps(&self) -> u16 {
        static BITRATES: [[u16; 15]; 3] = [
            // L1
            [
                0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320,
                352, 384, 416, 448,
            ],
            // L2
            [
                0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224,
                256, 320, 384,
            ],
            // L3
            [
                0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192,
                224, 256, 320,
            ],
        ];

        let bitrate_idx = self.bitrate_index();
        let layer_idx = self.layer() as usize - 1;

        BITRATES[layer_idx][bitrate_idx]
    }

    fn sampling_rate_index(&self) -> usize {
        // TODO: support MPEG 2 and 2.5
        debug_assert!(self.is_mpeg1());

        self.0.bit_range(10..12) as usize
    }

    fn sampling_rate_hz(&self) -> u32 {
        static RATES: [u32; 3] = [44100, 48000, 32000];

        RATES[self.sampling_rate_index()]
    }

    fn padding(&self) -> bool {
        self.0.bit_range(9..10) != 0
    }

    #[allow(unused)]
    fn private_bit(&self) -> bool {
        self.0.bit_range(8..9) != 0
    }

    fn channel_mode(&self) -> Mode {
        match self.0.bit_range(6..8) {
            0b00 => Mode::Stereo,
            0b01 => Mode::Joint,
            0b10 => Mode::Dual,
            0b11 => Mode::Single,
            _ => unreachable!()
        }
    }

    // TODO: use an enum
    fn mode_ext(&self) -> u8 {
        self.0.bit_range(4..6) as u8
    }

    fn copyright_bit(&self) -> bool {
        self.0.bit_range(3..4) != 0
    }

    fn original_bit(&self) -> bool {
        self.0.bit_range(2..3) != 0
    }

    fn emphasis(&self) -> Emphasis {
        match self.0.bit_range(0..2) {
            0b00 => Emphasis::None,
            0b01 => Emphasis::Ms_50_15,
            0b10 => Emphasis::Reserved,
            0b11 => Emphasis::CCIT_J_17,
            _ => unreachable!()
        }
    }
}

// private methods
impl<'a> Decoder<'a> {
    fn read_header(&mut self) -> Option<Result<Header>> {
        if self.raw.len() < 4 {
            return None;
        }

        let mut raw = 
            (self.raw[0] as u32) << 24
                | (self.raw[1] as u32) << 16
                | (self.raw[2] as u32) << 8
                | (self.raw[3] as u32) << 0;

        self.raw.advance(4);

        fn starts_with_syncword(raw: u32) -> bool {
            (raw.bit_range(20..32) | 1) as u16 == 0xfff0
        }

        if !starts_with_syncword(raw) {
            for byte in self.raw.iter().map(|b| *b) {
                raw <<= 8;
                raw |= byte as u32;

                if starts_with_syncword(raw) {
                    break;
                }
            }

            if !starts_with_syncword(raw) {
                return None;
            }
        }

        match Header::from_raw(raw) {
            Err(e) => Some(Err(e)),
            Ok(hdr) => {
                if hdr.is_protected() {
                    if self.raw.len() < 2 {
                        return None;
                    }

                    let crc = (self.raw[0] as u16) << 8
                        | (self.raw[1] as u16) << 0;

                    self.raw.advance(2);

                    // TODO: CRC check
                }

                Some(Ok(hdr))
            }
        }
    }
}




