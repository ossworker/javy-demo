extern crate core;
use std::env;
use std::ffi::{c_void, CStr, CString};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use wamr_rust_sdk::function::Function;
use wamr_rust_sdk::instance::Instance;
use wamr_rust_sdk::module::Module;
use wamr_rust_sdk::runtime::Runtime;
use wamr_rust_sdk::sys::wasm_runtime_module_malloc;
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
extern "C" fn test_extra() -> i32 {
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




pub async fn run(js_content: &str, json: &str) -> anyhow::Result<()> {
    let now = Instant::now();

    let runtime = Runtime::builder()
        .use_system_allocator()
        .register_host_function("extra", extra as *mut c_void)
        .build().unwrap();

    let x = test_extra as *mut c_void;
    println!("{:#?}",x);

    let wamr_runtime = &runtime;

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // d.push("add_extra_wasm32_wasi.wasm");
    d.push("wasi-demo-app.wasm");

    let mut module = Module::from_file(wamr_runtime, d.as_path())?;

    let wasi_ctx = WasiCtxBuilder::new()
        .set_pre_open_path(vec!["."],vec![])
        // .set_arguments(vec!["wasi-demo-app.wasm","daemon","hello"])
        .set_arguments(vec!["wasi-demo-app.wasm","write","1.txt",js_content])
        .set_env_vars(vec!["id=1","name=2"])
        .build();

    module.set_wasi_context(wasi_ctx);

    let instance = Instance::new_with_args(wamr_runtime,&module,1024 * 64,1024 * 64)?;


    // let function  = Function::find_export_func(&instance, "add")?;

    let function  = Function::find_export_func(&instance, "_start")?;

    let params: Vec<WasmValue> = vec![WasmValue::I32(92222222), WasmValue::I32(2122222222)];

    let result = function.call(&instance, &vec![WasmValue::I32(22)])?;

    let range = String::from("{\"code\":11}").as_bytes().to_vec();


    // let result = &result.encode()[0];

    println!("output:{:#?}", result);


    // let evaluate_response: EvaluateResponse = serde_json::from_slice(wasm_output.data.as_slice()).unwrap();
    // println!("evaluate_response:{:#?}", evaluate_response);
    // println!("result: success: \n{:#?} \nbody:\n{:#?}", wasm_output.success, String::from_utf8(wasm_output.data).unwrap());
    let first_end = now.elapsed().as_millis();
    println!("init cost:{:?}ms", first_end);

    let c_string = CString::new(&*js_content.as_bytes().to_vec())?;

    let c_char = c_string.as_ptr();

    println!("cstring ptr {:#?}", &c_char);

    let cstr = unsafe { CStr::from_ptr(c_char) };
    let string = String::from_utf8_lossy(cstr.to_bytes()).to_string();

    println!("cstring {:#?}", string);

    Ok(())

}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::time::Instant;
    use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
    use wamr_rust_sdk::runtime::Runtime;
    use wamr_rust_sdk::sys::{wasm_runtime_addr_app_to_native, wasm_runtime_free, wasm_runtime_malloc, wasm_runtime_module_malloc};

    #[test]
    fn test_runtime_builder_interpreter() {
        let runtime = Runtime::builder()
            .run_as_interpreter()
            .use_system_allocator()
            .build();
        assert!(runtime.is_ok());



        let small_buf = unsafe { wasm_runtime_malloc(16) };
        println!("{:#?}", &small_buf);
        assert!(!small_buf.is_null());
        // unsafe { wasm_runtime_free(small_buf) };

        let small_buf1 = unsafe { wasm_runtime_malloc(16) };

        unsafe { wasm_runtime_addr_app_to_native(small_buf, small_buf1)};

        let align = std::mem::align_of::<usize>();
        let layout = unsafe { std::alloc::Layout::from_size_align_unchecked(usize, align) };

        std::alloc::alloc(layout);

        unsafe { wasm_runtime_free(small_buf) };

        println!("{:#?}", &small_buf1);
        assert!(!small_buf1.is_null());

        unsafe { wasm_runtime_free(small_buf1) };

        // 0x0000600002f14010
        // 0x0000600002f14020
        // 0x0000600001760010
        // 0x0000600001760020
        // 0x000060000284c9a0
        // 0x000060000284c9b0
    }

    /*#[test]
    fn test_http(){
        let mut config = Config::new();
        config.timeouts = Timeouts::default();
        config.tls_config.provider = TlsProvider::default();

        let agent: Agent = config.new_agent();
        let start_time = Instant::now();
        for _i in 0..100 {
            // Reuses the connection from previous request.
            let _response: String = agent.post("https://httpbin.org/post")
                .header("Content-Type", "text/plain")
                .send("this is body").unwrap()
                .body_mut()
                .read_to_string().unwrap();
        }
        let end_time = Instant::now();
        let duration = end_time.duration_since(start_time);
        println!("耗时: {:?}", duration.as_millis());
    }*/

}
