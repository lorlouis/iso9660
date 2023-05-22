use std::io::Read;

const SECTOR_SIZE: usize = 2 * 1024; // 2K

const DATA_START: usize = 32_768; // 16 sectors

const VD_IDENT: &[u8; 5] = b"CD001";

#[repr(u8)]
pub enum VDType {
    BootRecord = 0,
    PrimaryVD = 1,
    EVD = 2,
    PartDes = 3,
    VDEnd = 255,
}

struct VD {
    ty: VDType,
    version: u8,
}

impl VD {

}

#[cfg(test)]
mod tests {
    use super::*;
}
