extern crate core;
use std::env;
use std::ffi::{c_void, CString};
use std::io::stdin;
use std::path::PathBuf;
use std::time::Instant;
use wamr_rust_sdk::function::Function;
use wamr_rust_sdk::instance::Instance;
use wamr_rust_sdk::module::Module;
use wamr_rust_sdk::runtime::Runtime;
use wamr_rust_sdk::sys::WASMMemoryType;
use wamr_rust_sdk::value::WasmValue;
use wamr_rust_sdk::wasi_context::WasiCtxBuilder;
// static WAMR_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
//
//     let runtime = Runtime::builder()
//         .use_system_allocator()
//         .register_host_function("extra", extra as *mut c_void)
//         .build().unwrap();
//
//    runtime
// });

extern "C" fn extra() -> i32 {
    100
}


// #[async_std::main]
#[tokio::main]
async fn main() {
    let handler_str = "export default {
    async handler(input, {dayjs, Big, moment,env}) {
        console.log('input', input);
        const momentValid = typeof moment === 'function' && Object.keys(moment).includes('isDayjs');
        const dayjsValid = typeof dayjs === 'function' && Object.keys(moment).includes('isDayjs');
        const bigjsValid = typeof Big === 'function';
        return {
            momentValid,
            dayjsValid,
            bigjsValid,
            bigjsTests: [
                Big(0.1).add(0.2).eq(0.3),
                Big(123.12).mul(0.1).round(2).eq(12.31),
            ],
            env
        };
    }
};";

    let json = "{\"id\":\"1\",\"name\":\"张三\"}";
    for _i in 0..1 {
        let _ = run(handler_str, json).await;
    }
    // drop(store);
}



pub fn vec_u32_to_u8(data: &Vec<u32>) -> Vec<u8> {
    // TODO: https://stackoverflow.com/questions/72631065/how-to-convert-a-u32-array-to-a-u8-array-in-place
    // TODO: https://stackoverflow.com/questions/29037033/how-to-slice-a-large-veci32-as-u8
    let capacity = 32/8 * data.len() as usize;  // 32/8 == 4
    let mut output = Vec::<u8>::with_capacity(capacity);
    for &value in data {
        output.push((value >> 24) as u8);  // r
        output.push((value >> 16) as u8);  // g
        output.push((value >>  8) as u8);  // b
        output.push((value >>  0) as u8);  // a
    }
    output
}

pub fn vec_u8_to_u32(data: &Vec<u8>) -> Vec<u32> {
    let capacity = data.len() /4 as usize;  // 32/8 == 4
    let mut output = Vec::<u32>::with_capacity(capacity);
    for &value in data {
        output.push((value as u32) << 24);  // r
        output.push((value as u32) << 16);  // g
        output.push((value as u32) <<  8);  // b
        output.push((value as u32) <<  0);  // a
    }
    output
}



pub async fn run(js_content: &str, json: &str) -> anyhow::Result<()> {
    let now = Instant::now();

    let runtime = Runtime::builder()
        .use_system_allocator()
        .register_host_function("extra", extra as *mut c_void)
        .build().unwrap();

    let wamr_runtime = &runtime;

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("add_extra_wasm32_wasi.wasm");

    let mut module = Module::from_file(wamr_runtime, d.as_path())?;

    let wasi_ctx = WasiCtxBuilder::new()
        .set_pre_open_path(vec!["."],vec![])
        .build();

    module.set_wasi_context(wasi_ctx);

    let instance = Instance::new(wamr_runtime,&module,1024 * 64)?;

    let function  = Function::find_export_func(&instance, "add")?;

    let params: Vec<WasmValue> = vec![WasmValue::I32(92222222), WasmValue::I32(2122222222)];

    let result = function.call(&instance, &params)?;

    let range = String::from("{\"code\":11}").as_bytes().to_vec();

    stdin()


    let result = &result.encode()[0];

    println!("output:{:#?}", result);


    // let evaluate_response: EvaluateResponse = serde_json::from_slice(wasm_output.data.as_slice()).unwrap();
    // println!("evaluate_response:{:#?}", evaluate_response);
    // println!("result: success: \n{:#?} \nbody:\n{:#?}", wasm_output.success, String::from_utf8(wasm_output.data).unwrap());
    let first_end = now.elapsed().as_millis();
    println!("init cost:{:?}ms", first_end);


    Ok(())

}
