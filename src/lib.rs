#![allow(unused)]
use std::io::{self, Read, Seek};

use core::ops::RangeInclusive;

mod iso9660_types;
use iso9660_types::*;

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
    UnknownPlatformId(u8),
    UnknownBootMedia(u8),
    UnknownBootIndicator(u8),
    UnknownHeaderIndicator(u8),
}

impl From<UnknownHeaderIndicator> for VDErr {
    fn from(value: UnknownHeaderIndicator) -> Self {
        Self::UnknownHeaderIndicator(value.0)
    }
}

impl From<UnknownBootIndicator> for VDErr {
    fn from(value: UnknownBootIndicator) -> Self {
        Self::UnknownBootIndicator(value.0)
    }
}

impl From<UnknownBootMedia> for VDErr {
    fn from(value: UnknownBootMedia) -> Self {
        Self::UnknownBootMedia(value.0)
    }
}

impl From<UnknownPlatformId> for VDErr {
    fn from(value: UnknownPlatformId) -> Self {
        Self::UnknownPlatformId(value.0)
    }
}

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
    pub sys_ident: Option<StrA<32>>,
    pub vol_ident: Option<StrD<32>>,
    pub vol_space_size: u32,
    pub vol_set_size: u16,
    pub vol_seq_num: u16,
    pub logical_block_size: u16,
    pub path_table_size: u32,
    pub path_table_l_location: u32,
    pub opt_path_table_l_location: Option<u32>,
    pub path_table_m_location: u32,
    pub opt_path_table_m_location: Option<u32>,
    pub vol_set_ident: Option<StrD<128>>,
    pub publisher_ident: Option<StrA<127>>,
    pub data_prep_ident: Option<StrA<127>>,
    pub app_ident: Option<StrA<127>>,
    pub copyright_file_name: Option<StrD<37>>,
    pub abstract_file_name: Option<StrD<37>>,
    pub bibliographic_file_name: Option<StrD<37>>,
    pub vol_create_date_time: Option<DecDateTime>,
    pub vol_mod_date_time: Option<DecDateTime>,
    pub vol_expiration_date_time: Option<DecDateTime>,
    pub vol_effective_date_time: Option<DecDateTime>,
    pub application_used: Option<[u8; 512]>,
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

    pub fn dump(&self, boot_record_addr: u32, out: &mut [u8]) {
        out[0] = 0;
        out[1..6].copy_from_slice(VD_IDENT);
        out[6..39].copy_from_slice(EL_TORITO_SPECIFICATION_STR.as_bytes());
        out[39..71].fill(0);
        out[71..75].copy_from_slice(&boot_record_addr.to_le_bytes());
        out[75..2048].fill(0);
    }
}

pub struct DirectoryRecordDate {
    pub year: StrD<4>,
    pub month: StrD<2>,
    pub day: StrD<2>,
    pub hour: StrD<2>,
    pub minute: StrD<2>,
    pub second: StrD<2>,
    /// Time zone offset from GMT in 15 minute intervals, starting at interval -48 (0)
    pub time_zone: u8,
}

impl Default for DirectoryRecordDate {
    /// defaults to the first of January at midnight UTC in the year 1
    fn default() -> Self {
        Self {
            year: StrD::from_slice(b" 1").unwrap(),
            month: StrD::from_slice(b" 1").unwrap(),
            day: StrD::from_slice(b" 1").unwrap(),
            hour: StrD::from_slice(b" 0").unwrap(),
            minute: StrD::from_slice(b" 0").unwrap(),
            second: StrD::from_slice(b" 0").unwrap(),
            time_zone: 12,
        }
    }
}

impl DirectoryRecordDate {
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
            time_zone,
        }))
    }
}

pub mod flags {
    pub const HIDDEN: u8 = 1;
    pub const DIR: u8 = 2;
    pub const ASSOCIATED: u8 = 4;
    pub const EXT_ATTR_FORMAT: u8 = 8;
    pub const EXT_ATTR_PERM: u8 = 16;
    pub const IS_PARTIAL: u8 = 128;
}

pub struct DirectoryRecord {
    pub size: u8,
    pub ext_attr_len: u8,
    pub extent_location: u32,
    pub data_size: u32,
    pub create_date: DirectoryRecordDate,
    pub flags: u8,
    pub interleaved_file_size: Option<u8>,
    pub interleaved_gap_size: Option<u8>,
    pub vol_seq_nul: u16,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Platform {
    X86 = 0,
    PPC = 1,
    Mac = 2, // mac is never used ?
    UEFI = 0xef, // not part of the spec..
}

pub struct UnknownPlatformId(pub u8);

impl TryFrom<u8> for Platform {
    type Error = UnknownPlatformId;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::X86),
            1 => Ok(Self::PPC),
            2 => Ok(Self::Mac),
            0xef => Ok(Self::UEFI),
            _ => Err(UnknownPlatformId(value))
        }
    }
}

#[derive(Debug)]
pub struct ValidationEntry {
    pub header_id: u8,
    pub platform_id: Platform,
    pub manufacturer_id: Option<StrA<24>>,
}

impl ValidationEntry {
    pub fn try_parse(buffer: &[u8]) -> Result<Self, VDErr> {
        let header_id = buffer[0];
        let platform_id = Platform::try_from(buffer[1])?;

        let manufacturer_id = {
            let s = StrA::from_slice(&buffer[4..28])?;
            match s.as_str().len() {
                0 => None,
                _ => Some(s)
            }
        };

        Ok(Self {
            header_id,
            platform_id,
            manufacturer_id,
        })
    }

    pub fn dump(&self, out: &mut [u8]) {
        out[0] = self.header_id;
        out[1] = self.platform_id as u8;
        out[2..4].fill(0);
        match self.manufacturer_id {
            Some(ref v) => out[4..28].copy_from_slice(v.raw_bytes()),
            None => out[4..28].fill(b' '),
        }
        out[28..30].fill(0);
        out[30] = 0x55;
        out[31] = 0xAA;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BootIndicator {
    NotBootable = 0,
    Bootable = 0x88,
}

pub struct UnknownBootIndicator(pub u8);

impl TryFrom<u8> for BootIndicator {
    type Error = UnknownBootIndicator;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NotBootable),
            0x88 => Ok(Self::Bootable),
            _ => Err(UnknownBootIndicator(value)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BootMedia {
    NoEmulation = 0,
    Floppy1_2 = 1,
    Floppy1_44 = 2,
    Floppy2_88 = 3,
    HardDrive = 4,
}

pub struct UnknownBootMedia(pub u8);

impl TryFrom<u8> for BootMedia {
    type Error = UnknownBootMedia;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoEmulation),
            1 => Ok(Self::Floppy1_2),
            2 => Ok(Self::Floppy1_44),
            3 => Ok(Self::Floppy2_88),
            4 => Ok(Self::HardDrive),
            _ => Err(UnknownBootMedia(value)),
        }
    }
}

#[derive(Debug)]
pub struct InitialEntry {
    pub boot_indicator: BootIndicator,
    pub boot_media: BootMedia,
    pub load_segment: u16,
    pub sys_type: u8,
    pub sector_count: u16,
    pub virtual_disk_addr: u32,
}

impl InitialEntry {
    pub fn try_parse(buffer: &[u8]) -> Result<Self, VDErr> {
        let boot_indicator = BootIndicator::try_from(buffer[0])?;
        let boot_media = BootMedia::try_from(buffer[1])?;

        let mut u16_buffer = [0_u8; 2];
        u16_buffer.copy_from_slice(&buffer[2..4]);
        let load_segment = u16::from_le_bytes(u16_buffer);

        let sys_type = buffer[4];

        u16_buffer.copy_from_slice(&buffer[6..8]);
        let sector_count = u16::from_le_bytes(u16_buffer);

        let mut u32_buffer = [0_u8; 4];
        u32_buffer.copy_from_slice(&buffer[8..12]);
        let virtual_disk_addr = u32::from_le_bytes(u32_buffer);

        Ok(Self {
            boot_indicator,
            boot_media,
            load_segment,
            sys_type,
            sector_count,
            virtual_disk_addr,
        })
    }

    pub fn dump(&self, out: &mut [u8]) {
        out[0] = self.boot_indicator as u8;
        out[1] = self.boot_media as u8;
        out[2..4].copy_from_slice(&self.load_segment.to_le_bytes());
        out[4] = self.sys_type;
        out[5] = 0;
        out[6..8].copy_from_slice(&self.sector_count.to_le_bytes());
        out[6..12].copy_from_slice(&self.virtual_disk_addr.to_le_bytes());
        out[12..32].fill(0); // might not be necessary
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum HeaderIndicator {
    Partial = 0x90,
    Final = 0x91,
}

#[derive(Debug)]
pub struct UnknownHeaderIndicator(pub u8);

impl TryFrom<u8> for HeaderIndicator {
    type Error = UnknownHeaderIndicator;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x90 => Ok(Self::Partial),
            0x91 => Ok(Self::Final),
            _ => Err(UnknownHeaderIndicator(value)),
        }
    }
}

#[derive(Debug)]
pub struct SectionHeaderEntry {
    pub header_indicator: HeaderIndicator,
    pub platform_id: Platform,
    pub nb_section_entries: u16,
    pub id_str: Option<StrA<28>>,
}

impl SectionHeaderEntry {
    pub fn try_parse(buffer: &[u8]) -> Result<Self, VDErr> {
        let header_indicator = HeaderIndicator::try_from(buffer[0])?;

        let platform_id = Platform::try_from(buffer[1])?;

        let mut u16_buffer = [0_u8; 2];
        u16_buffer.copy_from_slice(&buffer[2..4]);
        let nb_section_entries = u16::from_le_bytes(u16_buffer);

        let id_str: Option<StrA<28>> = {
            let s = StrA::from_slice(&buffer[4..32])?;
            if s.as_str().is_empty() {
                None
            } else {
                Some(s)
            }
        };

        Ok(Self {
            header_indicator,
            platform_id,
            nb_section_entries,
            id_str,
        })
    }

    pub fn dump(&self, out: &mut [u8]) {
        out[0] = self.header_indicator as u8;
        out[1] = self.platform_id as u8;
        out[2..4].copy_from_slice(&self.nb_section_entries.to_le_bytes());
        match self.id_str {
            Some(ref s) => out[4..32].copy_from_slice(s.raw_bytes()),
            None => out[4..32].fill(b' '),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SelectionCriteria {
    None = 0,
    LanguageAndVersion = 1,
    Unknown(u8),
}

impl From<SelectionCriteria> for u8 {
    fn from(value: SelectionCriteria) -> u8 {
        match value {
            SelectionCriteria::None => 0,
            SelectionCriteria::LanguageAndVersion => 1,
            SelectionCriteria::Unknown(v) => v,
        }
    }
}

impl From<u8> for SelectionCriteria {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::LanguageAndVersion,
            _ => Self::Unknown(value)
        }
    }
}

#[derive(Debug)]
pub struct SectionEntry {
    pub boot_indicator: BootIndicator,
    pub boot_media: BootMedia,
    pub has_continuation_entry: bool,
    pub image_contains_atapi_driver: bool,
    pub image_contains_scsi_driver: bool,
    pub load_segment: u16,
    pub sys_type: u8,
    pub sector_count: u16,
    pub virtual_disk_addr: u32,
    pub selection_criteria: SelectionCriteria,
    pub selection_criteria_bytes: [u8; 19],
}

impl SectionEntry {
    pub fn try_parse(buffer: &[u8]) -> Result<Self, VDErr> {
        let boot_indicator = BootIndicator::try_from(buffer[0])?;

        // the first 3 bits denote the media type
        let boot_media_bits = buffer[1] & 3;
        let boot_media = BootMedia::try_from(boot_media_bits)?;

        // the last 3 bits are used as a bitfield
        let has_continuation_entry = buffer[1] & (1 << 5) != 0;
        let image_contains_atapi_driver = buffer[1] & (1 << 6) != 0;
        let image_contains_scsi_driver = buffer[1] & (1 << 7) != 0;;

        let mut u16_bytes = [0_u8; 2];
        u16_bytes.copy_from_slice(&buffer[2..4]);
        let load_segment = u16::from_le_bytes(u16_bytes);

        let sys_type = buffer[4];

        u16_bytes.copy_from_slice(&buffer[6..8]);
        let sector_count = u16::from_le_bytes(u16_bytes);

        let mut u32_bytes = [0_u8; 4];
        u32_bytes.copy_from_slice(&buffer[8..12]);
        let virtual_disk_addr = u32::from_le_bytes(u32_bytes);

        let selection_criteria = SelectionCriteria::from(buffer[12]);

        let mut selection_criteria_bytes = [0_u8; 19];
        selection_criteria_bytes.copy_from_slice(&buffer[13..32]);

        Ok(Self {
            boot_indicator,
            boot_media,
            has_continuation_entry,
            image_contains_atapi_driver,
            image_contains_scsi_driver,
            load_segment,
            sys_type,
            sector_count,
            virtual_disk_addr,
            selection_criteria,
            selection_criteria_bytes,
        })

    }

    pub fn dump(&self, out: &mut [u8]) {
        out[0] = self.boot_indicator as u8;

        let mut second_bit = self.boot_media as u8;
        second_bit |= self.has_continuation_entry as u8 >> 5;
        second_bit |= self.image_contains_atapi_driver as u8 >> 6;
        second_bit |= self.image_contains_scsi_driver as u8 >> 7;
        out[1] = second_bit;

        out[2..4].copy_from_slice(&self.load_segment.to_le_bytes());
        out[4] = self.sys_type;
        out[5] = 0; // reserved but must be 0
        out[6..8].copy_from_slice(&self.sector_count.to_le_bytes());
        out[8..12].copy_from_slice(&self.virtual_disk_addr.to_le_bytes());
        out[12] = self.selection_criteria.into();
        out[13..32].copy_from_slice(&self.selection_criteria_bytes);
    }
}
