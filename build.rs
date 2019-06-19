use cbindgen::{Builder, Language};
use std::{env, fs, path::PathBuf};

fn main() {
    let target = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let package_name = env::var("CARGO_PKG_NAME").unwrap().replace("-", "_");
    let static_lib_name = format!("lib{}.a", package_name);

    let output_file = target.join(format!("{}.h", package_name));

    Builder::new()
        .with_no_includes()
        .with_language(Language::C)
        .with_crate(&target)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&output_file);


    let lib_before = target.join("target").join(env::var("PROFILE").unwrap()).join(&static_lib_name);
    let lib_after = target.join(static_lib_name);
    println!("{:?}", lib_before);
    println!("{:?}", lib_after);
    fs::copy(lib_before, lib_after).unwrap();



}
