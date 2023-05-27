#![feature(allocator_api)]
#![cfg_attr(feature="no_std", no_std)]

#[cfg(not(feature = "no_std"))]
use std::io::{self, Read, Seek};

use core::ops::RangeInclusive;

mod iso9660_types;
use iso9660_types::*;

mod allocator;

const EL_TORITO_SPECIFICATION_STR: &str = "EL TORITO SPECIFICATION";

pub const SECTOR_SIZE: usize = 2 * 1024; // 2K

pub const DATA_START: u64 = 32_768; // 16 sectors

const VD_IDENT: &[u8; 5] = b"CD001";

#[repr(u8)]
#[derive(Debug)]
pub enum VDType {
    BootRecord = 0,
    PrimaryVD = 1,
    EVD = 2,
    PartDes = 3,
    VDEnd = 255,
}

pub struct UnknownVersion(pub u8);

impl TryFrom<u8> for VDType {
    type Error=UnknownVersion;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BootRecord),
            1 => Ok(Self::PrimaryVD),
            2 => Ok(Self::EVD),
            3 => Ok(Self::PartDes),
            255 => Ok(Self::VDEnd),
            _ => Err(UnknownVersion(value)),
        }
    }
}

#[cfg(not(feature = "no_std"))]
pub fn read_sector<R: Read>(mut r: R) -> io::Result<Box<[u8]>> {
    use core::iter::repeat;
    // try not to allocate 2k in the stack
    let mut sector: Vec<u8> = repeat(0_u8).take(SECTOR_SIZE).collect();
    r.read_exact(&mut sector)?;
    Ok(sector.into_boxed_slice())
}


#[derive(Debug)]
pub struct VD {
    pub ty: VDType,
    pub version: u8,
}

#[derive(Debug)]
pub enum VDErr {
    #[cfg(not(feature = "no_std"))]
    Io(io::Error),

    UnknownVersion(u8),
    UnknownIdent([u8;5]),
    InvalidAlphabet {
        code_point: u8,
        alphabet: &'static [u8]
    },
    InvalidDate {
        range: RangeInclusive<&'static str>,
        actual: ArrStr<32>,
    },
}

#[cfg(not(feature = "no_std"))]
impl From<io::Error> for VDErr {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<UnknownVersion> for VDErr {
    fn from(value: UnknownVersion) -> Self {
        Self::UnknownVersion(value.0)
    }
}

impl From<InvalidChar> for VDErr {
    fn from(value: InvalidChar) -> Self {
        Self::InvalidAlphabet {
            code_point: value.code_point,
            alphabet: value.alphabet,
        }
    }
}

impl From<DecDateTimeErr> for VDErr {
    fn from(value: DecDateTimeErr) -> Self {
        match value {
            #[cfg(not(feature = "no_std"))]
            DecDateTimeErr::Io(e) => Self::Io(e),

            DecDateTimeErr::InvalidChar(code_point) => Self::InvalidAlphabet {
                code_point,
                alphabet: STR_D_CHAR_SET,
            },
            DecDateTimeErr::InvalidDate { range, actual } => Self::InvalidDate {
                range,
                actual,
            },
        }
    }
}

impl VD {
    pub fn read_header(buffer: &[u8]) -> Result<Self, VDErr> {
        let ty = VDType::try_from(buffer[0])?;

        let mut ident = [0_u8; 5];
        ident.clone_from_slice(&buffer[1..6]);
        if &ident != VD_IDENT {
            return Err(VDErr::UnknownIdent(ident))
        }

        let version = buffer[6];
        if version != 1 {
            return Err(VDErr::UnknownVersion(version));
        }

        Ok(Self {
            ty,
            version,
        })
    }
}

#[derive(Debug)]
pub struct PVD {
    sys_ident: Option<StrA<32>>,
    vol_ident: Option<StrD<32>>,
    vol_space_size: u32,
    vol_set_size: u16,
    vol_seq_num: u16,
    logical_block_size: u16,
    path_table_size: u32,
    path_table_l_location: u32,
    opt_path_table_l_location: Option<u32>,
    path_table_m_location: u32,
    opt_path_table_m_location: Option<u32>,
    vol_set_ident: Option<StrD<128>>,
    publisher_ident: Option<StrA<127>>,
    data_prep_ident: Option<StrA<127>>,
    app_ident: Option<StrA<127>>,
    copyright_file_name: Option<StrD<37>>,
    abstract_file_name: Option<StrD<37>>,
    bibliographic_file_name: Option<StrD<37>>,
    vol_create_date_time: Option<DecDateTime>,
    vol_mod_date_time: Option<DecDateTime>,
    vol_expiration_date_time: Option<DecDateTime>,
    vol_effective_date_time: Option<DecDateTime>,
    application_used: Option<[u8; 512]>,
}

impl PVD {
    pub fn try_parse(buffer: &[u8]) -> Result<Self, VDErr> {
        let sys_ident: Option<StrA<32>> = {
            let s = StrA::from_slice(&buffer[8..40])?;
            if s.as_str().is_empty() {
                None
            } else {
                Some(s)
            }
        };

        let vol_ident: Option<StrD<32>> = {
            let s = StrD::from_slice(&buffer[40..72])?;
            if s.as_str().is_empty() {
                None
            } else {
                Some(s)
            }
        };

        let vol_space_size = double_endian::u32(&buffer[80..88]);
        let vol_set_size = double_endian::u16(&buffer[120..124]);
        let vol_seq_num = double_endian::u16(&buffer[124..128]);
        let logical_block_size = double_endian::u16(&buffer[128..132]);
        let path_table_size = double_endian::u32(&buffer[132..140]);

        let path_table_l_location = {
            let mut u32_buffer = [0_u8; 4];
            u32_buffer.copy_from_slice(&buffer[140..144]);
            u32::from_le_bytes(u32_buffer)
        };
        let opt_path_table_l_location: Option<u32> = {
            let mut u32_buffer = [0_u8; 4];
            u32_buffer.copy_from_slice(&buffer[144..148]);
            match u32::from_le_bytes(u32_buffer) {
                0 => None,
                v => Some(v),
            }
        };

        let path_table_m_location = {
            let mut u32_buffer = [0_u8; 4];
            u32_buffer.copy_from_slice(&buffer[148..152]);
            u32::from_be_bytes(u32_buffer)
        };
        let opt_path_table_m_location: Option<u32> = {
            let mut u32_buffer = [0_u8; 4];
            u32_buffer.copy_from_slice(&buffer[152..156]);
            match u32::from_be_bytes(u32_buffer) {
                0 => None,
                v => Some(v),
            }
        };

        let vol_set_ident: Option<StrD<128>> = {
            let s = StrD::from_slice(&buffer[190..318])?;
            if s.as_str().is_empty() {
                None
            } else {
                Some(s)
            }
        };

        let publisher_ident: Option<StrA<127>> = if buffer[318] == 0x5f {
            Some(StrA::from_slice(&buffer[319..446])?)
        } else {
            None
        };

        let data_prep_ident: Option<StrA<127>> = if buffer[446] == 0x5f {
            Some(StrA::from_slice(&buffer[445..574])?)
        } else {
            None
        };

        let app_ident: Option<StrA<127>> = if buffer[574] == 0x5f {
            Some(StrA::from_slice(&buffer[575..702])?)
        } else {
            None
        };

        let copyright_file_name: Option<StrD<37>> = {
            let s = StrD::from_slice(&buffer[702..739])?;
            if s.as_str().is_empty() {
                None
            } else {
                Some(s)
            }
        };
        let abstract_file_name: Option<StrD<37>> = {
            let s = StrD::from_slice(&buffer[739..776])?;
            if s.as_str().is_empty() {
                None
            } else {
                Some(s)
            }
        };
        let bibliographic_file_name: Option<StrD<37>> = {
            let s = StrD::from_slice(&buffer[776..813])?;
            if s.as_str().is_empty() {
                None
            } else {
                Some(s)
            }
        };

        let vol_create_date_time: Option<DecDateTime> = DecDateTime::try_parse(&buffer[813..830])?;
        let vol_mod_date_time: Option<DecDateTime> = DecDateTime::try_parse(&buffer[830..847])?;
        let vol_expiration_date_time: Option<DecDateTime> = DecDateTime::try_parse(&buffer[847..864])?;
        let vol_effective_date_time: Option<DecDateTime> = DecDateTime::try_parse(&buffer[864..881])?;

        let version = buffer[881];
        if version != 1 {
            return Err(VDErr::UnknownVersion(version))
        }


        let mut application_used = [0_u8; 512];
        application_used.copy_from_slice(&buffer[883..1395]);

        let application_used = if app_ident.is_some() {
            Some(application_used)
        } else {
            None
        };

        Ok(Self {
            sys_ident,
            vol_ident,
            vol_space_size,
            vol_set_size,
            vol_seq_num,
            logical_block_size,
            path_table_size,
            path_table_l_location,
            opt_path_table_l_location,
            path_table_m_location,
            opt_path_table_m_location,
            vol_set_ident,
            publisher_ident,
            data_prep_ident,
            app_ident,
            copyright_file_name,
            abstract_file_name,
            bibliographic_file_name,
            vol_create_date_time,
            vol_mod_date_time,
            vol_expiration_date_time,
            vol_effective_date_time,
            application_used,
        })

    }
}


#[derive(Debug)]
pub struct BootRecord {
    pub boot_sys_ident: Option<StrA<32>>,
    pub boot_ident: Option<StrA<32>>,
}


impl BootRecord {
    pub fn try_parse(buffer: &[u8]) -> Result<Self, VDErr> {
        let boot_sys_ident: Option<StrA<32>> = {
            let s = StrA::from_slice(&buffer[7..39])?;
            if s.as_str().is_empty() {
                None
            } else {
                Some(s)
            }
        };

        let boot_ident: Option<StrA<32>> = {
            let s = StrA::from_slice(&buffer[39..71])?;
            if s.as_str().is_empty() {
                None
            } else {
                Some(s)
            }
        };

        Ok(Self {
            boot_sys_ident,
            boot_ident,
        })
    }

    pub fn read_el_torino_boot_catalog_off(buffer: &[u8]) -> u32 {
        let mut buf = [0_u8; 4];
        buf.copy_from_slice(&buffer[71..75]);
        u32::from_le_bytes(buf)
    }

}

pub struct DirectoryRecord {
    size: u8,
    ext_attr_len: u8,
    extent_location: u32,
}
