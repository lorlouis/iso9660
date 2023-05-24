use std::io::{self, Read};
use std::ops::RangeInclusive;

const STR_A_CHAR_SET: &[u8] = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789_",
    "!\"%&'()*+,-./:;<=>?")
    .as_bytes();

const STR_A_CHAR_SET_BIT_SET: [u8; 16] = build_ascii_bit_set(STR_A_CHAR_SET);

const fn build_ascii_bit_set(alphabet: &[u8]) -> [u8; 16] {
    let mut bitset = [0_u8; 16];
    let mut index = 0;
    let len = alphabet.len();
    while index < len {
        let b = alphabet[index];
        let byte_index = b / 8;
        let bit_index = (b & 7).saturating_sub(1);
        bitset[byte_index as usize] |= 1 << bit_index;
        index += 1;
    }
    bitset
}

#[derive(Debug)]
pub struct InvalidChar(u8);


pub struct StrA<const LEN: usize> {
    bytes: [u8; LEN],
    padding: usize,
}

impl<const LEN: usize> std::default::Default for StrA<LEN> {
    fn default() -> Self {
        Self {
            bytes: [0_u8; LEN],
            padding: Default::default()
        }
    }
}

impl<const LEN: usize> StrA<LEN> {

    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.bytes[self.padding..])
        }
    }

    /// SAFETY: `slice` must be of size LEN
    pub fn from_slice(slice: &[u8]) -> Result<Self, InvalidChar> {
        assert_eq!(slice.len(), LEN, "`slice` must be of size LEN");
        for b in slice {
            let byte_index = b / 8;
            let bit_index = (b & 7).saturating_sub(0);
            let bit_mask = 1 << bit_index;
            if bit_mask & STR_A_CHAR_SET_BIT_SET[byte_index as usize] == 0 {
                return Err(InvalidChar(*b))
            }
        }
        unsafe {
            Ok(Self::from_slice_unchecked(slice))
        }
    }

    /// # Safety `slice` must be of size LEN and the char set must respect `STR_A_CHAR_SET`
    pub unsafe fn from_slice_unchecked(slice: &[u8]) -> Self {
        let padding = slice.iter().take_while(|b| **b == b' ').count();
        let mut bytes = [0_u8; LEN];
        bytes.copy_from_slice(slice);
        Self {
            bytes,
            padding,
        }
    }
}


const STR_D_CHAR_SET: &[u8] = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789_").as_bytes();

const STR_D_CHAR_SET_BIT_SET: [u8; 16] = build_ascii_bit_set(STR_A_CHAR_SET);

pub struct StrD<const LEN: usize> {
    bytes: [u8; LEN],
    padding: usize,
}

impl<const LEN: usize> std::default::Default for StrD<LEN> {
    fn default() -> Self {
        Self {
            bytes: [0_u8; LEN],
            padding: Default::default()
        }
    }
}

impl<const LEN: usize> StrD<LEN> {

    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.bytes[self.padding..])
        }
    }

    /// SAFETY: `slice` must be of size LEN
    pub fn from_slice(slice: &[u8]) -> Result<Self, InvalidChar> {
        assert_eq!(slice.len(), LEN, "`slice` must be of size LEN");
        for b in slice {
            let byte_index = b / 8;
            let bit_index = (b & 7).saturating_sub(0);
            let bit_mask = 1 << bit_index;
            if bit_mask & STR_A_CHAR_SET_BIT_SET[byte_index as usize] == 0 {
                return Err(InvalidChar(*b))
            }
        }
        unsafe {
            Ok(Self::from_slice_unchecked(slice))
        }
    }
    /// # Safety `slice` must be of size LEN and the char set must respect `STR_D_CHAR_SET`
    pub unsafe fn from_slice_unchecked(slice: &[u8]) -> Self {
        let padding = slice.iter().take_while(|b| **b == b' ').count();
        let mut bytes = [0_u8; LEN];
        bytes.copy_from_slice(slice);
        Self {
            bytes,
            padding,
        }
    }
}

pub struct DecDateTime {
    year: StrD<4>,
    month: StrD<2>,
    day: StrD<2>,
    hour: StrD<2>,
    minute: StrD<2>,
    second: StrD<2>,
    centi_sec: StrD<2>,
    /// Time zone offset from GMT in 15 minute intervals, starting at interval -48 (0)
    time_zone: u8,
}

impl std::default::Default for DecDateTime {
    /// defaults to the first of January at midnight UTC in the year 1
    fn default() -> Self {
        Self {
            year: StrD::from_slice(b" 1").unwrap(),
            month: StrD::from_slice(b" 1").unwrap(),
            day: StrD::from_slice(b" 1").unwrap(),
            hour: StrD::from_slice(b" 0").unwrap(),
            minute: StrD::from_slice(b" 0").unwrap(),
            second: StrD::from_slice(b" 0").unwrap(),
            centi_sec: StrD::from_slice(b" 0").unwrap(),
            time_zone: 12,
        }
    }
}

pub enum DecDateTimeErr {
    Io(io::Error),
    InvalidChar(u8),
    InvalidDate {
        range: RangeInclusive<&'static str>,
        actual: String,
    },
}

impl From<io::Error> for DecDateTimeErr {
    fn from(value: io::Error) -> Self {
        DecDateTimeErr::Io(value)
    }
}

impl From<InvalidChar> for DecDateTimeErr {
    fn from(value: InvalidChar) -> Self {
        Self::InvalidChar(value.0)
    }
}

impl DecDateTime {
    pub fn try_parse<R: Read>(mut r: R) -> Result<Self, DecDateTimeErr> {
        let mut buffer = [0_u8; 17];
        r.read_exact(&mut buffer)?;

        let year = StrD::<4>::from_slice(&buffer[..4])?;
        if !("1".."9999").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "1"..="9999",
                actual: year.as_str().to_string(),
            })
        }

        let month = StrD::<2>::from_slice(&buffer[4..6])?;
        if !("1".."9999").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "1"..="12",
                actual: month.as_str().to_string(),
            })
        }

        let day = StrD::<2>::from_slice(&buffer[6..8])?;
        if !("1".."31").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "1"..="31",
                actual: day.as_str().to_string(),
            })
        }

        let hour = StrD::<2>::from_slice(&buffer[8..10])?;
        if !("0".."23").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "1"..="23",
                actual: hour.as_str().to_string(),
            })
        }

        let minute = StrD::<2>::from_slice(&buffer[10..12])?;
        if !("0".."59").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "0"..="59",
                actual: minute.as_str().to_string(),
            })
        }

        let second = StrD::<2>::from_slice(&buffer[12..14])?;
        if !("0".."59").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "0"..="59",
                actual: second.as_str().to_string(),
            })
        }

        let centi_sec = StrD::<2>::from_slice(&buffer[14..16])?;
        if !("0".."99").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "0"..="99",
                actual: centi_sec.as_str().to_string(),
            })
        }

        let time_zone = buffer[16];
        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            centi_sec,
            time_zone,
        })
    }
}

pub struct DoubleEndianI16 {
    pub le: i16,
    pub be: i16,
}

impl DoubleEndianI16 {
    pub fn native_endian(&self) -> i16 {
        if cfg!(target_endian = "big") {
            self.be
        }
        else {
            self.le
        }
    }
}

pub struct DoubleEndianU16 {
    pub le: u16,
    pub be: u16,
}

impl DoubleEndianU16 {
    pub fn native_endian(&self) -> u16 {
        if cfg!(target_endian = "big") {
            self.be
        }
        else {
            self.le
        }
    }
}

pub struct DoubleEndianU32 {
    pub le: u32,
    pub be: u32,
}

impl DoubleEndianU32 {
    pub fn native_endian(&self) -> u32 {
        if cfg!(target_endian = "big") {
            self.be
        }
        else {
            self.le
        }
    }
}

pub struct DoubleEndianI32 {
    pub le: i32,
    pub be: i32,
}

impl DoubleEndianI32 {
    pub fn native_endian(&self) -> i32 {
        if cfg!(target_endian = "big") {
            self.be
        }
        else {
            self.le
        }
    }
}
