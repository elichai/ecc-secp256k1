fn main() {
    #[cfg(feature = "generate-ffi")]
    {
        use std::fs;
        use std::process::Command;
        println!("cargo:rerun-if-changed=ecc_secp256k1.h");

        let res = Command::new("cbindgen").output().expect("Faild. do you have `cbindgen` installed?");

        fs::write("./ecc_secp256k1.h", res.stdout).expect("Failed writing the bindings to file");
    }
}
