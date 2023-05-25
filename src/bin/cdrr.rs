use iso9660::*;
use std::process::ExitCode;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

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

    let header = VD::read_header(&mut file).unwrap();
    println!("header: {:#?}\n", header);

    let pvd = PVD::try_parse(&file).unwrap();
    println!("pvd: {:#?}", pvd);


    ExitCode::SUCCESS
}
