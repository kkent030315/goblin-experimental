use std::env;
use std::fs;
use std::path::PathBuf;

use goblin::archive::*;
use goblin_experimental as goblin;
use scroll::Pread;

#[test]
fn parse_file_header() {
    let file_header: [u8; SIZEOF_HEADER] = [
        0x2f, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
        0x20, 0x30, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x30, 0x20,
        0x20, 0x20, 0x20, 0x20, 0x30, 0x20, 0x20, 0x20, 0x20, 0x20, 0x30, 0x20, 0x20, 0x20, 0x20,
        0x20, 0x20, 0x20, 0x38, 0x32, 0x34, 0x34, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x60, 0x0a,
    ];
    let buffer = &file_header[..];
    match buffer.pread::<MemberHeader>(0) {
        Err(e) => panic!("could not read the buffer: {:?}", e),
        Ok(file_header2) => {
            let file_header = MemberHeader {
                identifier: [
                    0x2f, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                    0x20, 0x20, 0x20,
                ],
                timestamp: [48, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32],
                owner_id: [48, 32, 32, 32, 32, 32],
                group_id: [48, 32, 32, 32, 32, 32],
                mode: [48, 32, 32, 32, 32, 32, 32, 32],
                file_size: [56, 50, 52, 52, 32, 32, 32, 32, 32, 32],
                terminator: [96, 10],
            };
            assert_eq!(file_header, file_header2)
        }
    }
}

#[test]
fn parse_archive() {
    let crt1a: Vec<u8> = include!("../etc/crt1a.rs");
    const START: &str = "_start";
    match Archive::parse(&crt1a) {
        Ok(archive) => {
            assert_eq!(archive.member_of_symbol(START), Some("crt1.o"));
            if let Some(member) = archive.get("crt1.o") {
                assert_eq!(member.offset, 194);
                assert_eq!(member.size(), 1928)
            } else {
                panic!("could not get crt1.o");
            }
            assert_eq!(archive.len(), 1);
            assert_eq!(archive.get_at(0).unwrap().extended_name(), "crt1.o");
        }
        Err(err) => panic!("could not parse archive: {:?}", err),
    };
}

fn get_libgoblin_rlib() -> PathBuf {
    // this test lies in `target/<target-triple>/<profile>/deps/`
    let me = env::current_exe().unwrap();
    // back to `<profile>` folder
    let artifact_dir = me.ancestors().nth(2).unwrap();
    artifact_dir.join("libgoblin_experimental.rlib")
}

// Cannot run this on Windows because *.rlib type on Windows is unknown
#[cfg(not(windows))]
#[test]
fn parse_self() {
    let path = get_libgoblin_rlib();
    let buffer = fs::read(path).expect("run `cargo build` first?");

    let archive = Archive::parse(&buffer).expect("parse rlib");

    // check that the archive has a useful symbol table by counting the total number of symbols
    let symbol_count: usize = archive
        .summarize()
        .into_iter()
        .map(|(_member_name, _member_index, ref symbols)| symbols.len())
        .sum();
    assert!(symbol_count > 500);

    let goblin_object_name = archive
        .members()
        .into_iter()
        .find(|member| {
            println!("member: {:?}", member);
            member.ends_with("goblin-archive.o")    // < 1.18
                || (member.starts_with("goblin") && member.ends_with("0.o")) // >= 1.18 && < 1.22
                || (member.starts_with("goblin") && member.ends_with("rust-cgu.o")) // = 1.22
                || (member.starts_with("goblin") && member.ends_with("rcgu.o")) // >= nightly 1.23
        })
        .expect("goblin-<hash>.0.o not found");

    let bytes = archive
        .extract(goblin_object_name, &buffer)
        .expect("extract goblin object");
    match goblin::Object::parse(&bytes).expect("parse object") {
        goblin::Object::Elf(elf) => {
            assert!(elf.entry == 0);
        }
        goblin::Object::Mach(goblin::mach::Mach::Binary(macho)) => {
            assert_eq!(macho.header.filetype, goblin::mach::header::MH_OBJECT);
            assert_eq!(macho.entry, 0);
        }
        other => {
            panic!("unexpected Object::parse result: {:?}", other);
        }
    }
}
