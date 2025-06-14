extern crate bindgen;
extern crate semver;

use semver::Version;
use std::{env, fs};

const MIN_VERSION: Version = Version::new(14, 2, 0);

fn main() {
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    let link_search_path = "cargo:rustc-link-search=native";
    let link_lib = "cargo:rustc-link-lib=static";

    let dkp = env::var("DEVKITPRO").expect("Please provided DEVKITPRO via env variables");
    let ppc = env::var("DEVKITPPC").expect("Please provided DEVKITPPC via env variables");

    let gcc_dir = format!("{ppc}/lib/gcc/powerpc-eabi");
    let version = fs::read_dir(&gcc_dir)
        .unwrap_or_else(|_| panic!("Failed to read directory: {gcc_dir}"))
        .filter_map(|entry| {
            entry
                .ok()?
                .file_name()
                .to_str()
                .and_then(|name| Version::parse(name).ok())
                .filter(|version| version >= &MIN_VERSION)
        })
        .max()
        .expect(&format!(
            "No valid versions >= {MIN_VERSION} found in {gcc_dir} directory"
        ));

    println!("{link_search_path}={ppc}/powerpc-eabi/lib",);
    println!("{link_search_path}={dkp}/wums/lib");

    println!("{link_lib}=notifications");
    println!("{link_lib}=stdc++");

    let bindings = bindgen::Builder::default()
        .use_core()
        .header("src/wrapper.h")
        .emit_builtins()
        .generate_cstr(true)
        .generate_comments(false)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .prepend_enum_name(false)
        .layout_tests(false)
        .derive_default(true)
        .merge_extern_blocks(true)
        .clang_args(vec![
            "--target=powerpc-none-eabi",
            &format!("--sysroot={ppc}/powerpc-eabi"),
            "-xc++",
            "-m32",
            "-mfloat-abi=hard",
            &format!("-I{dkp}/wums/include/notifications"),
            &format!("-I{dkp}/wut/include"),
            &format!("-I{ppc}/powerpc-eabi/include"),
            &format!("-I{ppc}/powerpc-eabi/include/c++/{version}"),
            &format!("-I{ppc}/powerpc-eabi/include/c++/{version}/powerpc-eabi"),
        ])
        .allowlist_file(".*/wums/include/notifications/.*.h")
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .generate()
        .expect("Unable to generate bindings");

    let out = std::path::PathBuf::from("./src/bindings.rs");
    bindings
        .write_to_file(&out)
        .expect("Unable to write bindings to file");
}
