
.PHONY: wizer

wizer:
	wizer --allow-wasi --wasm-bulk-memory true -o ./javy-demo.wasi.wasm ./javy-demo.wasm