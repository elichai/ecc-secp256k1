// TODO Should I receive a length and hash the message myself?.
// TODO: More flags?
pub mod ecdsa {
    use crate::{PrivateKey, PublicKey, Signature};
    use std::os::raw::{c_int, c_uchar};
    use std::{ptr, slice};

    #[no_mangle]
    /// Sign an ECDSA Signature
    /// The message should be a hashed 32 bytes.
    ///
    /// Input: msg -> pointer to 32 bytes message.
    ///        privkey -> pointer to 32 bytes private key.
    /// Output: sig_out -> pointer to a 64 bytes buffer.
    ///
    /// Returns:
    /// 1 - Finished successfully.
    /// 0 - Failed.
    ///
    ///
    pub unsafe extern "C" fn ecc_secp256k1_ecdsa_sign(sig_out: *mut c_uchar, msg: *const c_uchar, privkey: *const c_uchar) -> c_int {
        if sig_out.is_null() || msg.is_null() || privkey.is_null() {
            return -1;
        }
        let privkey = slice::from_raw_parts(privkey as *const u8, 32);
        let msg = slice::from_raw_parts(msg as *const u8, 32);
        let key = PrivateKey::from_serialized(privkey);
        let sig = key.sign(msg, false).serialize();
        ptr::copy_nonoverlapping(sig.as_ptr(), sig_out, sig.len());
        1
    }

    #[no_mangle]
    /// Verify a ECDSA Signature
    /// Accepts either compressed(33 btes) or uncompressed(65 bytes) public key. using the flag (1==compressed, 0==uncompressed).
    ///
    /// Input: sig -> pointer to 64 bytes signature.
    ///        msg -> 32 bytes result of a hash. (***Make Sure you hash the message yourself! otherwise it's easily broken***)
    ///        pubkey -> pointer to 33 or 65 bytes pubkey depending on the compressed flag.
    ///        compressed -> 1 for compressed, 0 for uncompressed.
    ///
    /// Returns:
    /// 1 - The signature is valid.
    /// 0 - Signature is not valid.
    /// -1 - Some other problem.
    ///
    pub unsafe extern "C" fn ecc_secp256k1_ecdsa_verify(
        sig: *const c_uchar,
        msg: *const c_uchar,
        pubkey: *const c_uchar,
        compressed: c_int,
    ) -> c_int {
        if sig.is_null() || msg.is_null() || pubkey.is_null() {
            return -1;
        }
        let pubkey_res = if compressed == 1 {
            let key = slice::from_raw_parts(pubkey as *const u8, 33);
            PublicKey::from_compressed(key)
        } else if compressed == 0 {
            let key = slice::from_raw_parts(pubkey as *const u8, 65);
            Ok(PublicKey::from_uncompressed(key))
        } else {
            return -1;
        };
        let pubkey = match pubkey_res {
            Ok(k) => k,
            Err(e) => {
                println!("ecc_secp256k1 Err: {}", e);
                return -1;
            }
        };

        let msg = slice::from_raw_parts(msg, 32);
        let sig = slice::from_raw_parts(sig, 64);
        let sig = Signature::parse_slice(sig);
        if pubkey.verify(msg, sig, false) {
            return 1;
        } else {
            0
        }
    }
}

pub mod schnorr {
    use crate::{PrivateKey, PublicKey, SchnorrSignature};
    use std::os::raw::{c_int, c_uchar};
    use std::{ptr, slice};

    #[no_mangle]
    /// Sign a Schnorr Signature
    /// The message should be a hashed 32 bytes.
    ///
    /// Input: msg -> pointer to 32 bytes message.
    ///        privkey -> pointer to 32 bytes private key.
    /// Output: sig_out -> pointer to a 64 bytes buffer.
    ///
    /// Returns:
    /// 1 - Finished successfully.
    /// 0 - Failed.
    ///
    ///
    pub unsafe extern "C" fn ecc_secp256k1_schnorr_sign(sig_out: *mut c_uchar, msg: *const c_uchar, privkey: *const c_uchar) -> c_int {
        if sig_out.is_null() || msg.is_null() || privkey.is_null() {
            return -1;
        }
        let privkey = slice::from_raw_parts(privkey as *const u8, 32);
        let msg = slice::from_raw_parts(msg as *const u8, 32);
        let key = PrivateKey::from_serialized(privkey);
        let sig = key.sign_schnorr(msg, false).serialize();
        ptr::copy_nonoverlapping(sig.as_ptr(), sig_out, sig.len());
        1
    }

    #[no_mangle]
    /// Verify a Schnorr Signature
    /// Accepts either compressed(33 btes) or uncompressed(65 bytes) public key. using the flag (1==compressed, 0==uncompressed).
    ///
    /// Input: sig -> pointer to 64 bytes signature.
    ///        msg -> 32 bytes result of a hash. (***Make Sure you hash the message yourself! otherwise it's easily broken***)
    ///        pubkey -> pointer to 33 or 65 bytes pubkey depending on the compressed flag.
    ///        compressed -> 1 for compressed, 0 for uncompressed.
    ///
    /// Returns:
    /// 1 - The signature is valid.
    /// 0 - Signature is not valid.
    /// -1 - Some other problem.
    ///
    pub unsafe extern "C" fn ecc_secp256k1_schnorr_verify(
        sig: *const c_uchar,
        msg: *const c_uchar,
        pubkey: *const c_uchar,
        compressed: c_int,
    ) -> c_int {
        if sig.is_null() || msg.is_null() || pubkey.is_null() {
            return -1;
        }
        let pubkey_res = if compressed == 1 {
            let key = slice::from_raw_parts(pubkey as *const u8, 33);
            PublicKey::from_compressed(key)
        } else if compressed == 0 {
            let key = slice::from_raw_parts(pubkey as *const u8, 64);
            Ok(PublicKey::from_uncompressed(key))
        } else {
            return -1;
        };
        let pubkey = match pubkey_res {
            Ok(k) => k,
            Err(e) => {
                println!("ecc_secp256k1 Err: {}", e);
                return -1;
            }
        };

        let msg = slice::from_raw_parts(msg, 32);
        let sig = slice::from_raw_parts(sig, 64);
        let sig = SchnorrSignature::parse_slice(sig);
        if pubkey.verify_schnorr(msg, sig, false) {
            return 1;
        } else {
            0
        }
    }
}
