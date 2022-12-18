use std::path::PathBuf;
use std::env;

fn main() {
    println!("cargo:rustc-link-search=/home/basti/Aaronia/RTSA/Aaronia-RTSA-Suite-PRO/");
    println!("cargo:rustc-link-lib=AaroniaRTSAAPI");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++14")
        .clang_arg("-I/home/basti/Aaronia/RTSA/Aaronia-RTSA-Suite-PRO/")
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

