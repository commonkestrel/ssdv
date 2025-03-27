mod decoder;
mod encoder;

pub use decoder::Decoder;
pub use encoder::Encoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub(crate) enum JpegMarker {
    Invalid = 0x0000,
    Tem = 0xFF01,
    Sof0 = 0xFFC0,
    Sof1,
    Sof2,
    Sof3,
    Dht,
    Sof5,
    Sof6,
    Sof7,
    Jpg,
    Sof9,
    Sof10,
    Sof11,
    Dac,
    Sof13,
    Sof14,
    Sof15,
    Rst0,
    Rst1,
    Rst2,
    Rst3,
    Rst4,
    Rst5,
    Rst6,
    Rst7,
    Soi,
    Eoi,
    Sos,
    Dqt,
    Dnl,
    Dri,
    Dhp,
    Exp,
    App0,
    App1,
    App2,
    App3,
    App4,
    App5,
    App6,
    App7,
    App8,
    App9,
    App10,
    App11,
    App12,
    App13,
    App14,
    App15,
    Jpg0,
    Jpg1,
    Jpg2,
    Jpg3,
    Jpg4,
    Jpg5,
    Jpg6,
    Sof48,
    Lse,
    Jpg9,
    Jpg10,
    Jpg11,
    Jpg12,
    Jpg13,
    Com,
}

impl PartialEq<u16> for JpegMarker {
    fn eq(&self, other: &u16) -> bool {
        return *self as u16 == *other;
    }
}

impl PartialEq<JpegMarker> for u16 {
    fn eq(&self, other: &JpegMarker) -> bool {
        return *self == *other as u16;
    }
}

impl PartialOrd<u16> for JpegMarker {
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        return (*self as u16).partial_cmp(other);
    }
}

impl PartialOrd<JpegMarker> for u16 {
    fn partial_cmp(&self, other: &JpegMarker) -> Option<std::cmp::Ordering> {
        return self.partial_cmp(&(*other as u16));
    }
}

impl From<u16> for JpegMarker {
    fn from(value: u16) -> Self {
        if value != JpegMarker::Tem && (value < JpegMarker::Sof0 || value > JpegMarker::Com) {
            return JpegMarker::Invalid;
        }

        return unsafe { std::mem::transmute(value) };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PacketType {
    /// Normal mode (224 byte packet + 32 byte FEC)
    Normal,
    /// No-FEC mode (256 byte packet)
    NoFEC,
    Padding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Quality {
    Q0 = 0,
    Q1,
    Q2,
    Q3,
    Q4,
    Q5,
    Q6,
    Q7,
}

impl Quality {
    /// Quantisation table scaling factors for each quality level 0-7
    const DQT_SCALES: [u16; 8] = [5000, 357, 172, 116, 100, 58, 28, 0];

    pub fn scale_factor(&self) -> u16 {
        let num = self.num();
        return Self::DQT_SCALES[num as usize];
    }

    pub fn num(&self) -> u8 {
        unsafe { std::mem::transmute_copy(self) }
    }
}

#[cfg(test)]
mod tests {}
