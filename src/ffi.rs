use std::ops::{Deref, DerefMut};
use crate::{Signature, SchnorrSignature};
#[repr(C)]
#[derive(Clone)]
pub struct SchnorrSig([u8; 64]);
#[repr(C)]
#[derive(Clone)]
pub struct EcdsaSig([u8; 64]);


// TODO Should I receive a length and hash the message myself?.
// TODO: More flags?
pub mod ecdsa {
    use std::os::raw::{c_int, c_uchar};
    use std::slice;
    use super::EcdsaSig;
    use crate::{PrivateKey, PublicKey};


    #[no_mangle]
    /// Sign an ECDSA Signature
    /// The message should be a hashed 32 bytes.
    ///
    /// Returns:
    /// 1 - Finished successfully.
    /// 0 - Failed.
    ///
    ///
    pub unsafe extern "C" fn ecc_secp256k1_ecdsa_sign(sig_out: *mut EcdsaSig, msg: *const c_uchar, privkey: *const c_uchar) -> c_int {
        let privkey = slice::from_raw_parts(privkey as *const u8, 32);
        let msg = slice::from_raw_parts(msg as *const u8, 32);
        let key = PrivateKey::from_serialized(privkey);
        let sig = key.sign(msg, false);
        (*sig_out).0 = sig.serialize();
        1
    }


    #[no_mangle]
    /// Verify a ECDSA Signature
    /// Accepts either compressed(33) or uncompressed(64) public key. using the flag (1==compressed, 0==uncompressed).
    ///
    /// The message should be a hashed 32 bytes.  (***Make Sure you hash the message yourself! otherwise it's easily broken***)
    /// Returns:
    /// 1 - The signature is valid.
    /// 0 - Signature is not valid.
    /// -1 - Some other problem.
    ///
    pub unsafe extern "C" fn secp256k1_ec_ecdsa_verify(sig: *const EcdsaSig, msg: *const c_uchar, pubkey: *const c_uchar, compressed: c_int) -> c_int {
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
        if pubkey.verify(msg, (*sig).clone().into(), false) {
            return 1
        } else {
            0
        }
    }
}


pub mod schnorr {
    use std::os::raw::{c_int, c_uchar};
    use super::SchnorrSig;
    use crate::{PrivateKey, PublicKey};
    use std::slice;


    #[no_mangle]
    /// Sign a Schnorr Signature
    /// The message should be a hashed 32 bytes.
    ///
    /// Returns:
    /// 1 - Finished successfully.
    /// 0 - Failed.
    ///
    ///
    pub unsafe extern "C" fn ecc_secp256k1_schnorr_sign(sig_out: *mut SchnorrSig, msg: *const c_uchar, privkey: *const c_uchar) -> c_int {
        let privkey = slice::from_raw_parts(privkey as *const u8, 32);
        let msg = slice::from_raw_parts(msg as *const u8, 32);
        let key = PrivateKey::from_serialized(privkey);
        let sig = key.sign_schnorr(msg, false);
        (*sig_out).0 = sig.serialize();
        1
    }

    #[no_mangle]
    /// Verify a Schnorr Signature
    /// Accepts either compressed(33) or uncompressed(64) public key. using the flag (1==compressed, 0==uncompressed).
    ///
    /// The message should be a hashed 32 bytes.  (***Make Sure you hash the message yourself! otherwise it's easily broken***)
    /// Returns:
    /// 1 - The signature is valid.
    /// 0 - Signature is not valid.
    /// -1 - Some other problem.
    ///
    pub unsafe extern "C" fn secp256k1_ec_schnorr_verify(sig: *const SchnorrSig, msg: *const c_uchar, pubkey: *const c_uchar, compressed: c_int) -> c_int {
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
        if pubkey.verify_schnorr(msg, (*sig).clone().into(), false) {
            return 1
        } else {
            0
        }
    }
}



impl From<EcdsaSig> for Signature {
    fn from(sig: EcdsaSig) -> Signature {
        Signature::parse(sig.0)
    }
}

impl From<SchnorrSig> for SchnorrSignature {
    fn from(sig: SchnorrSig) -> SchnorrSignature {
        SchnorrSignature::parse(sig.0)
    }
}

impl Deref for EcdsaSig {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EcdsaSig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for SchnorrSig {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SchnorrSig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}