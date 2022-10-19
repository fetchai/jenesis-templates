# Oracle Client Contract

This contract requests an oracle value from a specified oracle contract and pays a fee for it.

To test the contract:
``` bash
cargo test --lib
```

To compile the contract:
``` bash
RUSTFLAGS='-C link-arg=-s' cargo wasm
```
