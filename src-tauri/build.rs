use std::{env, fs, path::PathBuf};

use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};

fn main() {
    build_libraw();
    tauri_build::build()
}

fn build_libraw() {
    const RELEASE: &str = "LibRaw-0.22.3";
    const SHA256: &str = "789f3c33466210a1a0907f747a97c715d21bbf4af712f257fcbbd90111b227e4";
    let archive_path = format!("vendor/libraw/{RELEASE}.tar.gz");
    let bridge = "src/image_processing/raw_bridge.cpp";
    println!("cargo:rerun-if-changed={archive_path}");
    println!("cargo:rerun-if-changed={bridge}");

    let archive = fs::read(&archive_path).expect("read pinned LibRaw source archive");
    assert_eq!(format!("{:x}", Sha256::digest(&archive)), SHA256);
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    tar::Archive::new(GzDecoder::new(archive.as_slice()))
        .unpack(&output)
        .expect("unpack pinned LibRaw source");
    let source = output.join(RELEASE);
    let makefile = fs::read_to_string(source.join("Makefile.am")).expect("read LibRaw source list");
    let source_list = makefile
        .split_once("lib_libraw_a_SOURCES =")
        .and_then(|(_, rest)| rest.split_once("\n\n"))
        .expect("find upstream LibRaw source list")
        .0;
    let mut compiler = cc::Build::new();
    compiler
        .cpp(true)
        .std("c++17")
        .warnings(false)
        .define("LIBRAW_NODLL", None)
        .include(&source)
        .file(bridge)
        .files(
            source_list
                .replace('\\', " ")
                .split_whitespace()
                .map(|file| source.join(file)),
        )
        .compile("photart_libraw");
}
