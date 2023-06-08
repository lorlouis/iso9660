use iso9660::*;
use std::process::ExitCode;
use std::fs::File;
use std::io::{Seek, SeekFrom};

use std::env;

fn print_usage(prg_name: &str) {
    eprintln!("Usage: {} <file.iso>", prg_name);
}

fn main() -> ExitCode {
    let mut args = env::args();
    let prg_name = args.next().expect("no arg 0?");
    let file_name = match args.next() {
        Some(v) => v,
        None => {
            print_usage(&prg_name);
            return ExitCode::FAILURE
        }
    };

    let mut file = match File::open(&file_name) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("unable to open {}: `{}`", file_name, e);
            return ExitCode::FAILURE
        }
    };

    let off = 0x8800;
    file.seek(SeekFrom::Start(off)).unwrap();


    let sector = read_sector(&mut file).unwrap();
    let header = VD::read_header(&sector).unwrap();
    println!("header: 0x{:x} {:?}", off, header);

    let record = BootRecord::try_parse(&sector).unwrap();
    println!("{:#?}", record);
    let offset = record.boot_catalog_addr.unwrap() * SECTOR_SIZE as u32;
    println!("boot catalog off: {}", offset);

    file.seek(SeekFrom::Start(offset as u64)).unwrap();

    let sector = read_sector(&mut file).unwrap();
    let validation = ValidationEntry::try_parse(&sector).unwrap();
    println!("validation: {:#?}", validation);

    let initial = InitialEntry::try_parse(&sector[32..]).unwrap();
    println!("initial: {:#?}", initial);

    let section_header = SectionHeaderEntry::try_parse(&sector[64..]).unwrap();
    println!("section_header: {:#?}", section_header);

    let section = SectionEntry::try_parse(&sector[96..]).unwrap();
    println!("section: {:#?}", section);

    file.seek(SeekFrom::Start(section.virtual_disk_addr as u64 * 2048)).unwrap();


    ExitCode::SUCCESS
}
