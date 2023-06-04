use std::io::{self, Read};

use core::ops::RangeInclusive;
use core::fmt::Debug;
use core::default::Default;
use core::ops::Deref;

pub const STR_A_CHAR_SET: &[u8] = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789_",
    " !\"%&'()*+,-./:;<=>?")
    .as_bytes();

const STR_A_CHAR_SET_BIT_SET: [u8; 16] = build_ascii_bit_set(STR_A_CHAR_SET);

const fn build_ascii_bit_set(alphabet: &[u8]) -> [u8; 16] {
    let mut bitset = [0_u8; 16];
    let mut index = 0;
    let len = alphabet.len();
    while index < len {
        let b = alphabet[index];
        let byte_index = b / 8;
        let bit_index = (8 - (b & 7)).saturating_sub(1);
        bitset[byte_index as usize] |= 1 << bit_index;
        index += 1;
    }
    bitset
}

#[derive(Debug)]
pub struct InvalidChar {
    pub code_point: u8,
    pub alphabet: &'static [u8],
}

pub struct ArrStr<const LEN: usize> {
    bytes: [u8; LEN],
    len: usize,
}

impl<const LEN: usize> Default for ArrStr<LEN> {
    fn default() -> Self {
        Self {
            bytes: [0_u8; LEN],
            len: Default::default()
        }
    }
}

#[derive(Debug)]
pub struct TooBig;

impl<const LEN: usize> TryFrom<&str> for ArrStr<LEN> {
    type Error = TooBig;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() > LEN {
            return Err(TooBig)
        }
        let mut bytes = [0_u8; LEN];
        let len = value.len();
        bytes[..len].copy_from_slice(value.as_bytes());
        Ok(Self {
            bytes,
            len,
        })
    }
}

impl<const LEN: usize> ArrStr<LEN> {
    pub fn raw_bytes(&self) -> &[u8;LEN] {
        &self.bytes
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(&self.bytes[..self.len])
        }
    }

    /// SAFETY: `slice` must be of size LEN
    pub fn from_slice_with_ascii_subset(slice: &[u8], alphabet: &[u8;16]) -> Result<Self, InvalidChar> {
        assert_eq!(slice.len(), LEN, "`slice` must be of size LEN");
        let len = LEN - slice.iter().rev()
            // NB(louis): the standard specifies that strings should be
            // padded with spaces but sometimes they are padded with zeroes
            .take_while(|b| **b == b' ' || **b == 0).count();
        for &b in &slice[..len] {
            let byte_index = b / 8;
            let bit_index = (8 - (b & 7)).saturating_sub(1);
            let bit_mask = 1 << bit_index;
            if bit_mask & alphabet[byte_index as usize] == 0 {
                return Err(InvalidChar {
                    code_point: b,
                    alphabet: STR_A_CHAR_SET,
                })
            }
        }
        unsafe {
            Ok(Self::from_slice_unchecked(slice, len))
        }
    }

    /// # Safety `slice` must be of size LEN and the char set must respect `STR_A_CHAR_SET`
    pub unsafe fn from_slice_unchecked(slice: &[u8], len: usize) -> Self {
        let mut bytes = [0_u8; LEN];
        bytes.copy_from_slice(slice);
        Self {
            bytes,
            len,
        }
    }

}

impl<const LEN: usize> Debug for ArrStr<LEN> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ArrStr")
            .field("bytes", &self.as_str())
            .field("len", &self.len).finish()
    }
}


#[derive(Debug)]
pub struct StrA<const LEN: usize> {
    inner: ArrStr<LEN>
}

impl<const LEN: usize> StrA<LEN> {
    pub fn from_slice(slice: &[u8]) -> Result<Self, InvalidChar> {
        Ok(Self {
            inner: ArrStr::from_slice_with_ascii_subset(slice, &STR_A_CHAR_SET_BIT_SET)?,
        })
    }
}

impl<const LEN: usize> Deref for StrA<LEN> {
    type Target = ArrStr<LEN>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(crate) const STR_D_CHAR_SET: &[u8] = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789_").as_bytes();

const STR_D_CHAR_SET_BIT_SET: [u8; 16] = build_ascii_bit_set(STR_A_CHAR_SET);

#[derive(Debug, Default)]
pub struct StrD<const LEN: usize> {
    inner: ArrStr<LEN>
}

impl<const LEN: usize> Deref for StrD<LEN> {
    type Target = ArrStr<LEN>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const LEN: usize> StrD<LEN> {
    /// SAFETY: `slice` must be of size LEN
    pub fn from_slice(slice: &[u8]) -> Result<Self, InvalidChar> {
        Ok(Self {
            inner: ArrStr::from_slice_with_ascii_subset(slice, &STR_D_CHAR_SET_BIT_SET)?,
        })
    }
}

#[derive(Debug)]
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

impl Default for DecDateTime {
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
        actual: ArrStr<32>,
    },
}

#[cfg(not(feature = "no_std"))]
impl From<io::Error> for DecDateTimeErr {
    fn from(value: io::Error) -> Self {
        DecDateTimeErr::Io(value)
    }
}

impl From<InvalidChar> for DecDateTimeErr {
    fn from(value: InvalidChar) -> Self {
        Self::InvalidChar(value.code_point)
    }
}

impl DecDateTime {
    pub fn try_parse(buffer: &[u8]) -> Result<Option<Self>, DecDateTimeErr> {

        if buffer[16] == 0 && buffer[..16].iter().all(|&b| b == b'0') {
            return Ok(None)
        }

        let year = StrD::<4>::from_slice(&buffer[..4])?;
        if !("1".."9999").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "1"..="9999",
                actual: year.as_str().try_into().unwrap(),
            })
        }

        let month = StrD::<2>::from_slice(&buffer[4..6])?;
        if !("1".."9999").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "1"..="12",
                actual: month.as_str().try_into().unwrap(),
            })
        }

        let day = StrD::<2>::from_slice(&buffer[6..8])?;
        if !("1".."31").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "1"..="31",
                actual: day.as_str().try_into().unwrap(),
            })
        }

        let hour = StrD::<2>::from_slice(&buffer[8..10])?;
        if !("0".."23").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "1"..="23",
                actual: hour.as_str().try_into().unwrap(),
            })
        }

        let minute = StrD::<2>::from_slice(&buffer[10..12])?;
        if !("0".."59").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "0"..="59",
                actual: minute.as_str().try_into().unwrap(),
            })
        }

        let second = StrD::<2>::from_slice(&buffer[12..14])?;
        if !("0".."59").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "0"..="59",
                actual: second.as_str().try_into().unwrap(),
            })
        }

        let centi_sec = StrD::<2>::from_slice(&buffer[14..16])?;
        if !("0".."99").contains(&year.as_str()) {
            return Err(DecDateTimeErr::InvalidDate {
                range: "0"..="99",
                actual: centi_sec.as_str().try_into().unwrap(),
            })
        }

        let time_zone = buffer[16];
        Ok(Some(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            centi_sec,
            time_zone,
        }))
    }
}

pub mod double_endian {
    use core::mem::size_of;

    pub fn i16(slice: &[u8]) -> i16 {
        const SIZE: usize = size_of::<i16>();
        let mut buffer = [0_u8; SIZE];

        if cfg!(target_endian = "little") {
            buffer.copy_from_slice(&slice[..SIZE]);
        } else {
            buffer.copy_from_slice(&slice[SIZE..(SIZE*2)]);
        }

        i16::from_ne_bytes(buffer)
    }

    pub fn u16(slice: &[u8]) -> u16 {
        const SIZE: usize = size_of::<u16>();
        let mut buffer = [0_u8; SIZE];

        if cfg!(target_endian = "little") {
            buffer.copy_from_slice(&slice[..SIZE]);
        } else {
            buffer.copy_from_slice(&slice[SIZE..(SIZE*2)]);
        }

        u16::from_ne_bytes(buffer)
    }

    pub fn i32(slice: &[u8]) -> i32 {
        const SIZE: usize = size_of::<i32>();
        let mut buffer = [0_u8; SIZE];

        if cfg!(target_endian = "little") {
            buffer.copy_from_slice(&slice[..SIZE]);
        } else {
            buffer.copy_from_slice(&slice[SIZE..(SIZE*2)]);
        }

        i32::from_ne_bytes(buffer)
    }

    pub fn u32(slice: &[u8]) -> u32 {
        const SIZE: usize = size_of::<u32>();
        let mut buffer = [0_u8; SIZE];

        if cfg!(target_endian = "little") {
            buffer.copy_from_slice(&slice[..SIZE]);
        } else {
            buffer.copy_from_slice(&slice[SIZE..(SIZE*2)]);
        }

        u32::from_ne_bytes(buffer)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bit_mask() {
        let s = StrA::<83>::from_slice(STR_A_CHAR_SET).unwrap();
        // make sure space is present in char set A
        assert!(STR_A_CHAR_SET.contains(&32));
    }
}
