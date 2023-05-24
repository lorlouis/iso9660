use std::io::{self, Read, Seek};
use std::iter::repeat;
use std::ops::RangeInclusive;

mod iso9660_types;
use iso9660_types::*;

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

#[derive(Debug)]
pub struct VD {
    ty: VDType,
    version: u8,
}

#[derive(Debug)]
pub enum VDErr {
    Io(io::Error),
    UnknownVersion(u8),
    UnknownIdent([u8;5]),
    InvalidAlphabet(u8, &'static [u8]),
    InvalidDate {
        range: RangeInclusive<&'static str>,
        actual: String,
    },
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

impl VD {
    pub fn read_header<R: Read>(mut r: R) -> Result<Self, VDErr> {
        let mut buffer = [0_u8; 7];
        r.read_exact(&mut buffer)?;

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

pub struct PVD {
    sys_ident: StrA<32>,
    vol_ident: StrD<32>,
    vol_space_size: u32,
    vol_set_size: u16,
    vol_seq_num: u16,
    logical_block_size: u16,
    path_table_size: u32,
    path_table_l_location: u32,
    opt_path_table_location: Option<u32>,
    vol_set_ident: StrD<128>,
    publisher_ident: StrA<128>,
    data_prep_ident: StrA<128>,
    app_ident: StrA<128>,
    copyright_file_name: Option<StrD<37>>,
    abstract_file_name: Option<StrD<37>>,
    bibliographic_file_name: Option<StrD<37>>,
    vol_create_date_time: DecDateTime,
    vol_mod_date_time: DecDateTime,
    vol_expiration_date_time: DecDateTime,
    vol_effective_date_time: DecDateTime,
    application_used: StrA<512>,
}

impl PVD {
    pub fn try_parse<R: Read>(mut r: R) -> Result<Self, VDErr> {
        // TODO(louis) read 2k and interpret it

        // substitute the size of the header
        let sector_minus_header = SECTOR_SIZE-6;
        // try not to allocate 2k in the stack
        let mut buffer: Vec<u8> = repeat(0_u8).take(sector_minus_header).collect();
        r.read_exact(&mut buffer)?;

        let sys_ident: StrA<32> = StrA::default();
        let vol_ident: StrD<32> = StrD::default();
        let vol_space_size: u32 = 0;
        let vol_set_size: u16 = 0;
        let vol_seq_num: u16 = 0;
        let logical_block_size: u16 = 0;
        let path_table_size: u32 = 0;
        let path_table_l_location: u32 = 0;
        let opt_path_table_location: Option<u32> = None;
        let vol_set_ident: StrD<128> = StrD::default();
        let publisher_ident: StrA<128> = StrA::default();
        let data_prep_ident: StrA<128> = StrA::default();
        let app_ident: StrA<128> = StrA::default();
        let copyright_file_name: Option<StrD<37>> = None;
        let abstract_file_name: Option<StrD<37>> = None;
        let bibliographic_file_name: Option<StrD<37>> None;
        let vol_create_date_time: DecDateTime,
        let vol_mod_date_time: DecDateTime,
        let vol_expiration_date_time: DecDateTime,
        let vol_effective_date_time: DecDateTime,
        let application_used: StrA<512>,


        todo!()
    }
}

pub struct IsoReader<R> {
    reader: R,
}

impl<R: Read + Seek> IsoReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
