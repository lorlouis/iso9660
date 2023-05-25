use std::io::{self, Read};
use std::ops::RangeInclusive;

pub const STR_A_CHAR_SET: &[u8] = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
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
pub struct InvalidChar {
    pub code_point: u8,
    pub alphabet: &'static [u8],
}


pub struct StrA<const LEN: usize> {
    bytes: [u8; LEN],
    len: usize,
}

impl<const LEN: usize> std::fmt::Debug for StrA<LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StrA")
            .field("bytes", &self.as_str())
            .field("len", &self.len).finish()
    }
}

impl<const LEN: usize> std::default::Default for StrA<LEN> {
    fn default() -> Self {
        Self {
            bytes: [0_u8; LEN],
            len: Default::default()
        }
    }
}

impl<const LEN: usize> StrA<LEN> {

    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.bytes[..self.len])
        }
    }

    /// SAFETY: `slice` must be of size LEN
    pub fn from_slice(slice: &[u8]) -> Result<Self, InvalidChar> {
        assert_eq!(slice.len(), LEN, "`slice` must be of size LEN");
        for b in slice {
            let byte_index = b / 8;
            let bit_index = (b & 7).saturating_sub(1);
            let bit_mask = 1 << bit_index;
            if bit_mask & STR_A_CHAR_SET_BIT_SET[byte_index as usize] == 0 {
                return Err(InvalidChar {
                    code_point: *b,
                    alphabet: STR_A_CHAR_SET,
                })
            }
        }
        unsafe {
            Ok(Self::from_slice_unchecked(slice))
        }
    }

    /// # Safety `slice` must be of size LEN and the char set must respect `STR_A_CHAR_SET`
    pub unsafe fn from_slice_unchecked(slice: &[u8]) -> Self {
        let len = LEN - slice.iter().rev().take_while(|b| **b == b' ').count();
        let mut bytes = [0_u8; LEN];
        bytes.copy_from_slice(slice);
        Self {
            bytes,
            len,
        }
    }
}


pub(crate) const STR_D_CHAR_SET: &[u8] = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789_").as_bytes();

const STR_D_CHAR_SET_BIT_SET: [u8; 16] = build_ascii_bit_set(STR_A_CHAR_SET);

pub struct StrD<const LEN: usize> {
    bytes: [u8; LEN],
    len: usize,
}


impl<const LEN: usize> std::fmt::Debug for StrD<LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StrD")
            .field("bytes", &self.as_str())
            .field("len", &self.len).finish()
    }
}

impl<const LEN: usize> std::default::Default for StrD<LEN> {
    fn default() -> Self {
        Self {
            bytes: [0_u8; LEN],
            len: Default::default()
        }
    }
}

impl<const LEN: usize> StrD<LEN> {

    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.bytes[..self.len])
        }
    }

    /// SAFETY: `slice` must be of size LEN
    pub fn from_slice(slice: &[u8]) -> Result<Self, InvalidChar> {
        assert_eq!(slice.len(), LEN, "`slice` must be of size LEN");
        for b in slice {
            let byte_index = b / 8;
            let bit_index = (b & 7).saturating_sub(1);
            let bit_mask = 1 << bit_index;
            if bit_mask & STR_A_CHAR_SET_BIT_SET[byte_index as usize] == 0 {
                return Err(InvalidChar {
                    code_point: *b,
                    alphabet: STR_D_CHAR_SET
                })
            }
        }
        unsafe {
            Ok(Self::from_slice_unchecked(slice))
        }
    }
    /// # Safety `slice` must be of size LEN and the char set must respect `STR_D_CHAR_SET`
    pub unsafe fn from_slice_unchecked(slice: &[u8]) -> Self {
        let len = LEN - slice.iter().rev().take_while(|b| **b == b' ').count();
        let mut bytes = [0_u8; LEN];
        bytes.copy_from_slice(slice);
        Self {
            bytes,
            len,
        }
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
        Self::InvalidChar(value.code_point)
    }
}

impl DecDateTime {
    pub fn try_parse<R: Read>(mut r: R) -> Result<Option<Self>, DecDateTimeErr> {
        let mut buffer = [0_u8; 17];

        r.read_exact(&mut buffer)?;

        if buffer[16] == 0 && buffer[..16].iter().all(|&b| b == b'0') {
            return Ok(None)
        }

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

    pub fn i16(slice: &[u8]) -> i16 {
        const SIZE: usize = std::mem::size_of::<i16>();
        let mut buffer = [0_u8; SIZE];

        if cfg!(target_endian = "little") {
            buffer.copy_from_slice(&slice[..SIZE]);
        } else {
            buffer.copy_from_slice(&slice[SIZE..(SIZE*2)]);
        }

        i16::from_ne_bytes(buffer)
    }

    pub fn u16(slice: &[u8]) -> u16 {
        const SIZE: usize = std::mem::size_of::<u16>();
        let mut buffer = [0_u8; SIZE];

        if cfg!(target_endian = "little") {
            buffer.copy_from_slice(&slice[..SIZE]);
        } else {
            buffer.copy_from_slice(&slice[SIZE..(SIZE*2)]);
        }

        u16::from_ne_bytes(buffer)
    }

    pub fn i32(slice: &[u8]) -> i32 {
        const SIZE: usize = std::mem::size_of::<i32>();
        let mut buffer = [0_u8; SIZE];

        if cfg!(target_endian = "little") {
            buffer.copy_from_slice(&slice[..SIZE]);
        } else {
            buffer.copy_from_slice(&slice[SIZE..(SIZE*2)]);
        }

        i32::from_ne_bytes(buffer)
    }

    pub fn u32(slice: &[u8]) -> u32 {
        const SIZE: usize = std::mem::size_of::<u32>();
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
        let s = StrA::<82>::from_slice(STR_A_CHAR_SET).unwrap();
    }
}
