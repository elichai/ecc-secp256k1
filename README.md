# ecc-secp256k1

## This repository is for learning purposes only, please *DO NOT* use in production

This repository aims to create a pure rust ECC library (in the end secp256k1 only).
right now it's *not* optimized, *not* constant time
It provides both low level API (Mul/Add/Div etc.), and Private/Public Keys interface with ECDSA and ECDH.


## TODO:
- [x] ECDSA
- [x] ECDH
- [ ] Implement DER formats.
- [ ] Replace random `k` with deterministic.
- [ ] Remove the usage of GMP library.
- [ ] Look into implementing sha2 myself.
- [ ] Remove all `unimplemented!()` and add checks for the points all over.
- [ ] Add Schnorr support.
- [ ] Bulletproofs?
