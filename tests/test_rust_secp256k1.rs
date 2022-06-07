//#![cfg(test)]

extern crate secp256k1 as test_secp256k1;

use ecc_secp256k1::{HashTrait, PrivateKey, PublicKey, Signature};
use test_secp256k1::rand::{thread_rng, Rng, RngCore};
use test_secp256k1::{
    ecdsa::Signature as TestSignature, Message as TestMessage, PublicKey as TestPublicKey, SecretKey as TestPrivateKey, SECP256K1,
};

#[test]
fn test_cmp_sign_der() {
    let mut rng = thread_rng();

    let key: [u8; 32] = rng.gen();
    let priv_key = PrivateKey::from_serialized(&key);
    let msg = get_rand_msg();

    let orig_sig = priv_key.sign(&msg, true);
    let pubkey = priv_key.generate_pubkey().compressed();

    // Verify with rust-secp256k1

    let pubkey = TestPublicKey::from_slice(&pubkey).unwrap();
    let msg = TestMessage::from_slice(&msg.hash_digest()).unwrap();
    let sig = TestSignature::from_der(&orig_sig.serialize_der()).unwrap();

    assert_eq!(&sig.serialize_der()[..], &orig_sig.serialize_der()[..]);
    assert!(SECP256K1.verify_ecdsa(&msg, &sig, &pubkey).is_ok())
}

#[test]
fn test_cmp_sign_compact() {
    let mut rng = thread_rng();

    let key: [u8; 32] = rng.gen();
    let priv_key = PrivateKey::from_serialized(&key);
    let msg = get_rand_msg();

    let orig_sig = priv_key.sign(&msg, true).serialize();
    let pubkey = priv_key.generate_pubkey().compressed();

    // Verify with rust-secp256k1

    let pubkey = TestPublicKey::from_slice(&pubkey).unwrap();
    let msg = TestMessage::from_slice(&msg.hash_digest()).unwrap();
    let sig = TestSignature::from_compact(&orig_sig).unwrap();
    assert_eq!(&sig.serialize_compact()[..], &orig_sig[..]);
    assert!(SECP256K1.verify_ecdsa(&msg, &sig, &pubkey).is_ok())
}

fn get_rand_msg() -> Vec<u8> {
    let mut rng = thread_rng();
    let msg_len: usize = rng.gen_range(1, 1024);
    let mut msg = vec![0u8; msg_len];
    rng.fill_bytes(&mut msg[..]);
    msg
}

#[test]
fn test_cmp_verify_compact_uncompressed() {
    // Sign with rust-secp256k1
    let orig_msg = get_rand_msg();
    let msg = TestMessage::from_slice(&orig_msg.hash_digest()).unwrap();
    let privkey = TestPrivateKey::new(&mut thread_rng());
    let sig = SECP256K1.sign_ecdsa(&msg, &privkey).serialize_compact();
    let pubkey = TestPublicKey::from_secret_key(&SECP256K1, &privkey).serialize_uncompressed();

    // Verify with This library

    let sig = Signature::parse(sig);
    let pubkey = PublicKey::from_uncompressed(&pubkey);
    assert!(pubkey.verify(&orig_msg, sig, true));
}

#[test]
fn test_cmp_verify_der_compressed() {
    // Sign with rust-secp256k1
    let orig_msg = get_rand_msg();
    let msg = TestMessage::from_slice(&orig_msg.hash_digest()).unwrap();
    let privkey = TestPrivateKey::new(&mut thread_rng());
    let sig = SECP256K1.sign_ecdsa(&msg, &privkey);
    let pubkey = TestPublicKey::from_secret_key(&SECP256K1, &privkey).serialize();

    // Verify with This library

    let sig = Signature::parse_der(&sig.serialize_der());
    let pubkey = PublicKey::from_compressed(&pubkey).unwrap();
    assert!(pubkey.verify(&orig_msg, sig, true));
}

#[test]
fn test_compare_sigs() {
    let orig_msg = get_rand_msg();

    let test_msg = TestMessage::from_slice(&orig_msg.hash_digest()).unwrap();
    let test_privkey = TestPrivateKey::new(&mut thread_rng());
    let test_sig = SECP256K1.sign_ecdsa(&test_msg, &test_privkey).serialize_compact();

    let my_privkey = PrivateKey::from_serialized(&test_privkey[..]);
    let my_sig = my_privkey.sign(&orig_msg, true).serialize();

    assert_eq!(&my_sig[..], &test_sig[..]);
}
