
.PHONY: wizer opt

wizer:
	wizer --allow-wasi --wasm-bulk-memory true -o ./javy-demo.wasi.wasm ./javy-demo.wasm

#O：启用默认优化，等同于-Os参数
 #-O0：不进行任何优化
 #-O1：进行一些基本的优化，例如内联函数优化和死代码消除优化
 #-O2：进行更为彻底的优化，例如函数重写、数据重排、内存分配优化等
 #-O3：进行最为彻底的优化，包括一些可能影响程序功能的优化
 #-O4：与 -O3 相同，但会启用更为激进的优化
 #-Os：优化目标是减小代码大小，会进行一些可能影响性能的优化
 #-Oz：与 -Os 相同，但会启用更为激进的优化

opt-min:
	wasm-opt -Oz -o javy-module.min.wasm javy-module.wasm

opt-opt:
	wasm-opt -O4 -o javy-module.opt.wasm javy-module.wasm

	wasm-opt -O4 -o js.opt.wasm javy-demo.wasm

test-input:
	echo '{"id":"1","name":"李四"}' | wasmtime  .\target\wasm32-wasi\release\javy-demo.wasm

	echo '{"body": "{\"desc\":\"input is default\"}", "js_content": "const handler = (input, {dayjs, Big, moment, env}) => { console.log(\"input\", input);return {  env  };};"}' | wasmtime  target/wasm32-wasi/release/javy-demo.wasm

	echo '{"body": "{\"desc\":\"input is default\"}", "js_content": "const handler = (input, {dayjs, Big, moment, env}) => { console.log(\"input\", input);return {  env  };};"}' | wasmtime  javy-module.opt.wasm