src/main.wasm: src/main.rs
	rustc -C opt-level=1 --target wasm32-unknown-unknown src/main.rs -o $@
