target/release/host: guest/target/wasm32-unknown-unknown/release/guest.wasm
	cargo build --release --bin=host

guest/target/wasm32-unknown-unknown/release/guest.wasm:
	cd guest && make

bench:
	cargo bench

lua:
	hyperfine 'luajit src/foo.lua'
