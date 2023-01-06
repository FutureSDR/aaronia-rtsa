use std::env;
use std::path::PathBuf;


#[cfg(not(windows))]
fn search() -> Option<String> {
    const LIB: &str = "AaroniaRTSAAPI";
    const LIB_NAME: &str = "libAaroniaRTSAAPI.so";
    const HEADER_NAME: &str = "aaroniartsaapi.h";

    println!("cargo:rustc-link-lib={LIB}");

    let paths = env::var_os("RTSA_DIR")
        .unwrap_or(concat!(env!("HOME"), "/Aaronia/RTSA/Aaronia-RTSA-Suite-PRO").into());

    for dir in env::split_paths(&paths) {
        let lib_path = dir.join(LIB_NAME);
        let inc_path = dir.join(HEADER_NAME);
        if lib_path.is_file() && inc_path.is_file() {
            let dir = dir.to_str().expect("sdk path not valid utf-8");
            println!("cargo:rustc-link-search={dir}");
            return Some(dir.to_string());
        }
    }
    None
}

#[cfg(windows)]
fn search() -> Option<String> {
    const LIB: &str = "AaroniaRTSAAPI";
    const LIB_NAME: &str = "AaroniaRTSAAPI.lib";
    const HEADER_NAME: &str = "aaroniartsaapi.h";

    println!("cargo:rustc-link-lib={LIB}");

    let paths = env::var("RTSA_DIR")
        .unwrap_or(r"C:\Program Files\Aaronia AG\Aaronia RTSA-Suite PRO".into());

    for dir in env::split_paths(&paths) {
        let lib_path = dir.join("sdk").join(LIB_NAME);
        let inc_path = dir.join("sdk").join(HEADER_NAME);
        if lib_path.is_file() && inc_path.is_file() {
            let lib_dir = dir.to_str().expect("sdk path not valid utf-8");
            println!("cargo:rustc-link-search={lib_dir}");
            let dir = dir.join("sdk").to_str().expect("sdk path not valid utf-8");
            return Some(dir.to_string());
        }
    }
    None
}

fn main() {
    let dir = search().expect("sdk not found, set RTSA_DIR environment variable");

    println!("cargo:rerun-if-env-changed=RTSA_DIR");

    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++14")
        .clang_arg(format!("-I{dir}"))
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
