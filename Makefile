.PHONY: build opt

build:
	cargo wasi build -r

opt: build
	wizer --allow-wasi --wasm-bulk-memory true -o ./javy-demo.wasm ./target/wasm32-wasi/release/javy-demo.wasm

test-input:
	echo '{"id":"1","name":"李四"}' | wasmtime  .\target\wasm32-wasi\release\javy-demo.wasm
