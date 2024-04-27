#![feature(lazy_cell)]

use std::ops::Deref;
use std::sync::LazyLock;
use std::time::Instant;
use std::{env, fs};

use serde::Deserialize;
use serde_json::Value;
use wasmtime::component::Component;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::{self};

use crate::errors::RuntimeError;
use crate::io::{WasmInput, WasmOutput};
use crate::runtime::CtxBuilder;
use crate::stdio::Stdio;

mod errors;
mod io;
mod runtime;
mod stdio;

static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
    let mut config = Config::default();
    config
        .async_support(true)
        .wasm_component_model(true)
        .wasm_multi_memory(true)
        .wasm_threads(true)
        .cache_config_load_default()
        .unwrap();
    Engine::new(&config).unwrap()
});

pub enum ModuleOrComponent {
    Module(Module),
    Component(Component),
}

fn parse_module_or_component(url: &str) -> ModuleOrComponent {
    let mut path_buf = env::current_dir().unwrap();
    path_buf.push(url);
    println!("{:#?}", &path_buf);
    let bytes = fs::read(path_buf).unwrap();
    let engine = ENGINE.deref();
    let module_or_component = if wasmparser::Parser::is_component(&bytes) {
        Ok(ModuleOrComponent::Component(
            Component::from_binary(ENGINE.deref(), &bytes).expect("load component error"),
        ))
    } else if wasmparser::Parser::is_core_wasm(&bytes) {
        Ok(ModuleOrComponent::Module(
            Module::from_binary(engine, &bytes).expect("load module error"),
        ))
    } else {
        Err(RuntimeError::CannotReadModule)
    };
    module_or_component.unwrap()
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
    for _i in 0..10 {
        run(handler_str, json).await;
    }
    // drop(store);
}

pub fn prepare_wasi_context(wasi_builder: &mut CtxBuilder) -> anyhow::Result<()> {
    match wasi_builder {
        CtxBuilder::Preview2(_wasi_builder) => {
            // wasi_builder
            //     .preopened_dir(
            //         Dir::open_ambient_dir(env::current_dir().unwrap(),
            //                               ambient_authority()).unwrap(),
            //         DirPerms::all(),
            //         FilePerms::all(),
            //         ".",
            //     );
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponse {
    pub output: Value,
    pub log: Vec<Value>,
}

pub async fn run(js_content: &str, json: &str) {
    let now = Instant::now();
    let engine = ENGINE.deref();
    let input = serde_json::to_vec(&WasmInput::new(js_content, json)).unwrap();

    // let mut ctx_builder = CtxBuilder::Preview2(wasmtime_wasi::WasiCtxBuilder::new());

    let stdio = Stdio::new(input);

    // let ctx_builder = wasmtime_wasi::WasiCtxBuilder::new()
    //     .stdin(wasmtime_wasi::pipe::MemoryInputPipe::new(
    //         stdio.stdin.to_vec(),
    //     ))
    //     .stdout(stdio.stdout.clone())
    //     .stderr(stdio.stderr.clone());

    //设置in out error
    // let ctx_builder = stdio.configure_wasi_ctx(ctx_builder);

    // let wasi_host_ctx = match ctx_builder {
    //     CtxBuilder::Preview2(mut wasi_builder) => WasiHostCtx {
    //         preview2_ctx: wasi_builder.build(),
    //         preview2_table: wasmtime_wasi::ResourceTable::new(),
    //     },
    // };

    let module_or_component = parse_module_or_component("js.opt.wasm");

    let wasm_output = {
        match &module_or_component {
            ModuleOrComponent::Component(_) => {}
            ModuleOrComponent::Module(module) => {
                // wasi_common::pipe::ReadPipe::peek(buf)
                let wasi_host_ctx: WasiP1Ctx = wasmtime_wasi::WasiCtxBuilder::new()
                    // let wasi_host_ctx: WasiCtx = wasi_common::sync::WasiCtxBuilder::new()
                    .stdin(wasmtime_wasi::pipe::MemoryInputPipe::new(
                        stdio.stdin.to_vec(),
                    ))
                    .stdout(stdio.stdout.clone())
                    .stderr(stdio.stderr.clone())
                    .build_p1();
                // .build();
                let mut store: Store<WasiP1Ctx> = Store::new(&engine, wasi_host_ctx);

                let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
                preview1::add_to_linker_async(&mut linker, |t| t).unwrap();

                let func = linker
                    .module_async(&mut store, "", &module)
                    .await
                    .unwrap()
                    .get_default(&mut store, "")
                    .unwrap()
                    .typed::<(), ()>(&store)
                    .unwrap();

                // Invoke the WASI program default function.
                func.call_async(&mut store, ()).await.unwrap();
            }
        }
        if stdio.stdout.contents().is_empty() {
            WasmOutput::new(false, stdio.stderr.contents().to_vec())
        } else {
            WasmOutput::new(true, stdio.stdout.contents().to_vec())
        }
    };
    println!(
        "result: success: \n{:#?} \nbody:\n{:#?}",
        wasm_output.success,
        String::from_utf8_lossy(&wasm_output.data)
    );
    // let evaluate_response: EvaluateResponse = serde_json::from_slice(wasm_output.data.as_slice()).unwrap();
    // println!("evaluate_response:{:#?}", evaluate_response);
    // println!("result: success: \n{:#?} \nbody:\n{:#?}", wasm_output.success, String::from_utf8(wasm_output.data).unwrap());
    let first_end = now.elapsed().as_millis();
    println!("init cost:{:?}ms", first_end);
}
