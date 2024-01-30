.PHONY: build opt

build:
	cargo wasi build -r

opt: build
	wizer --allow-wasi --wasm-bulk-memory true -o ./javy-demo.wasi.wasm ./target/wasm32-wasi/release/javy-demo.wasm

test-input:
	echo '{"id":"1","name":"李四"}' | wasmtime  .\target\wasm32-wasi\release\javy-demo.wasm

	echo '{"body": "{\"desc\":\"input is default\"}", "js_content": "const handler = (input, {dayjs, Big, moment, env}) => { console.log(\"input\", input1);return {  env  };};"}' | wasmtime  target/wasm32-wasi/release/javy-demo.wasm
