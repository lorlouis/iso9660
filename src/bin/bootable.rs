use std::io::{self, Read, Write};

use iso9660::*;

fn main () {
    let mut file: Vec<u8> = std::iter::repeat(0).take(SECTOR_SIZE * 3).collect();

    let primary_header = VD {
        ty: VDType::PrimaryVD,
        version: 1,
    };

    // Needed to fool the bios into thinking this buffer is a proper iso9660 disk
    primary_header.dump(&mut file);

    let boot_record = BootRecord::el_torito(18); // the sector right after the boot record
    boot_record.dump(&mut file[SECTOR_SIZE..]);

    // boot catalogue
    let validation = ValidationEntry {
        header_id: 1,
        platform_id: Platform::X86,
        manufacturer_id: None,
    };

    validation.dump(&mut file[SECTOR_SIZE*2..]);

    let initial = InitialEntry {
        boot_indicator: BootIndicator::Bootable,
        boot_media: BootMedia::Floppy1_44,
        load_segment: 0, // ie default value (I know it should be an option)
        sys_type: 0,  // no idea what it's supposed to be, idk it felt right
        sector_count: 4, // hmm intresting
        virtual_disk_addr: 19, // the last segment
    };

    initial.dump(&mut file[SECTOR_SIZE*2+32..]);

    let section_header = SectionHeaderEntry {
        header_indicator: HeaderIndicator::Final,
        platform_id: Platform::X86,
        nb_section_entries: 1,
        id_str: None,
    };

    section_header.dump(&mut file[SECTOR_SIZE*2+64..]);

    let section_entry = SectionEntry {
        boot_indicator: BootIndicator::Bootable,
        boot_media: BootMedia::Floppy1_44,
        has_continuation_entry: false,
        image_contains_atapi_driver: false,
        image_contains_scsi_driver: false,
        load_segment: 0,
        sys_type: 0, // again, no idea
        sector_count: 4,
        virtual_disk_addr: 19,
        selection_criteria: SelectionCriteria::None,
        selection_criteria_bytes: Default::default(),
    };

    section_entry.dump(&mut file[SECTOR_SIZE*2+96..]);

    io::stdout().write_all(&file).unwrap();
}
