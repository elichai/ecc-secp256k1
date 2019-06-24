use cbindgen::{Builder, Language};
use std::{env, path::PathBuf};

fn main() {
    let target = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let package_name = env::var("CARGO_PKG_NAME").unwrap().replace("-", "_");
    let output_file = target.join(format!("{}.h", package_name));

    Builder::new()
        .with_no_includes()
        .with_language(Language::C)
        .with_crate(&target)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&output_file);
}
