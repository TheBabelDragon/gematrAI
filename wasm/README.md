# WASM artifacts

Built by CI from `gematra-wasm` via `wasm-bindgen`.

```bash
cd rust
cargo build -p gematra-wasm --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/gematra_wasm.wasm \
  --out-dir ../wasm --target web --no-typescript
```
