# Fetch Oracle Contract

This contract stores an oracle value and allows other contrasts to request the value in exchange for a fee.

To test the contract:
``` bash
cargo test --lib
```

To compile the contract:
``` bash
RUSTFLAGS='-C link-arg=-s' cargo wasm
```
