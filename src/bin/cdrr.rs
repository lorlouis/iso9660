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

    file.seek(SeekFrom::Start(DATA_START)).unwrap();

    let mut off = 0x8000;

    loop {
        let sector = read_sector(&mut file).unwrap();
        let header = VD::read_header(&sector).unwrap();
        println!("header: 0x{:x} {:?}", off, header);

        match header.ty {
            VDType::BootRecord => {
                let record = BootRecord::try_parse(&sector).unwrap();
                println!("{:#?}", record);
                let offset = BootRecord::read_el_torino_boot_catalog_off(&sector);
                println!("boot catalog off: {}", offset);
            },
            VDType::PrimaryVD => {
                let pvd = PVD::try_parse(&sector).unwrap();
                println!("{:#?}", pvd);
            },
            VDType::EVD => (),
            VDType::PartDes => todo!(),
            VDType::VDEnd => break,
        }
        println!();

        off += 2048;
    }

    ExitCode::SUCCESS
}
