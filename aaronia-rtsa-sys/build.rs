use std::env;
use std::path::PathBuf;

const LIB: &str = "AaroniaRTSAAPI";
const LIB_NAME: &str = "libAaroniaRTSAAPI.so";
const HEADER_NAME: &str = "aaroniartsaapi.h";

fn search() -> Option<PathBuf> {
    let paths = env::var_os("RTSA_DIR")
        .unwrap_or(concat!(env!("HOME"), "/Aaronia/RTSA/Aaronia-RTSA-Suite-PRO").into());

    for dir in env::split_paths(&paths) {
        let lib_path = dir.join(LIB_NAME);
        let inc_path = dir.join(HEADER_NAME);
        if lib_path.is_file() && inc_path.is_file() {
            return Some(dir);
        }
    }
    None
}

fn main() {
    let dir = search().unwrap();

    println!("cargo:rustc-link-search={}", dir.to_str().unwrap());
    println!("cargo:rustc-link-lib={LIB}");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++14")
        .clang_arg(format!("-I{}", dir.to_str().unwrap()))
        .header("wrapper.h")
        .allowlist_function("AARTSAAPI.*")
        .allowlist_var("AARTSAAPI.*")
        .allowlist_type("AARTSAAPI.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
