#![feature(lazy_cell)]

use std::{env, fs};
use std::ops::Deref;
use std::sync::LazyLock;
use std::time::Instant;
use serde::Deserialize;
use serde_json::Value;

use wasmtime::{component, Config, Engine, Linker, Module, Store};
use wasmtime::component::Component;
use wasmtime_wasi::{ambient_authority, Dir, preview2};
use wasmtime_wasi::preview2::{DirPerms, FilePerms};

use crate::errors::RuntimeError;
use crate::io::{WasmInput, WasmOutput};
use crate::runtime::CtxBuilder;
use crate::stdio::Stdio;

mod errors;
mod stdio;
mod io;
mod runtime;

static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
    let mut config = Config::default();
    config.async_support(true)
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


// #[derive(Default)]
struct WasiHostCtx {
    preview2_ctx: preview2::WasiCtx,
    preview2_table: preview2::Table,
    preview1_adapter: preview2::preview1::WasiPreview1Adapter,
}

impl preview2::WasiView for WasiHostCtx {
    fn table(&self) -> &preview2::Table {
        &self.preview2_table
    }

    fn table_mut(&mut self) -> &mut preview2::Table {
        &mut self.preview2_table
    }

    fn ctx(&self) -> &preview2::WasiCtx {
        &self.preview2_ctx
    }

    fn ctx_mut(&mut self) -> &mut preview2::WasiCtx {
        &mut self.preview2_ctx
    }
}

impl preview2::preview1::WasiPreview1View for WasiHostCtx {
    fn adapter(&self) -> &preview2::preview1::WasiPreview1Adapter {
        &self.preview1_adapter
    }

    fn adapter_mut(&mut self) -> &mut preview2::preview1::WasiPreview1Adapter {
        &mut self.preview1_adapter
    }
}


fn parse_module_or_component(url: &str) -> ModuleOrComponent {
    let mut path_buf = env::current_dir().unwrap();
    path_buf.push(url);
    println!("{:#?}", &path_buf);
    let bytes = fs::read(path_buf).unwrap();
    let engine = ENGINE.deref();
    let module_or_component = if wasmparser::Parser::is_component(&bytes) {
        Ok(ModuleOrComponent::Component(
            Component::from_binary(ENGINE.deref(), &bytes).expect("load component error")
        ))
    } else if wasmparser::Parser::is_core_wasm(&bytes) {
        Ok(ModuleOrComponent::Module(
            Module::from_binary(engine, &bytes).expect("load module error")
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
        CtxBuilder::Preview2(wasi_builder) => {
            wasi_builder
                .preopened_dir(
                    Dir::open_ambient_dir(env::current_dir().unwrap(),
                                          ambient_authority()).unwrap(),
                    DirPerms::all(),
                    FilePerms::all(),
                    ".",
                );
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


    let mut ctx_builder = CtxBuilder::Preview2(preview2::WasiCtxBuilder::new());

    let _ = prepare_wasi_context(&mut ctx_builder);

    let stdio = Stdio::new(input);

    //设置in out error
    let ctx_builder = stdio.configure_wasi_ctx(ctx_builder);

    let wasi_host_ctx = match ctx_builder {
        CtxBuilder::Preview2(mut wasi_builder) => {
            WasiHostCtx {
                preview2_ctx: wasi_builder.build(),
                preview2_table: preview2::Table::default(),
                preview1_adapter: preview2::preview1::WasiPreview1Adapter::new(),
            }
        }
    };

    let mut store = Store::new(&engine, wasi_host_ctx);

    let module_or_component = parse_module_or_component("js.opt.wasm");

    let wasm_output = {
        match &module_or_component {
            ModuleOrComponent::Component(component) => {
                let mut component_linker = component::Linker::new(&engine);
                preview2::command::add_to_linker(&mut component_linker).unwrap();
                let (comand, _instance) = preview2::command::Command::instantiate_async(
                    &mut store,
                    component,
                    &component_linker,
                ).await.unwrap();
                let _ = comand
                    .wasi_cli_run()
                    .call_run(&mut store)
                    .await
                    .unwrap();
            }
            ModuleOrComponent::Module(module) => {
                let mut linker: Linker<WasiHostCtx> = Linker::new(&engine);
                preview2::preview1::add_to_linker_async(&mut linker).unwrap();
                let func = linker
                    .module_async(&mut store, "", &module)
                    .await.unwrap()
                    .get_default(&mut store, "").unwrap()
                    .typed::<(), ()>(&store).unwrap();

                // Invoke the WASI program default function.
                func.call_async(&mut store, ()).await.unwrap();
            }
        }
        drop(store);
        if stdio.stderr.contents().is_empty() {
            WasmOutput::new(false, stdio.stderr.contents().to_vec())
        } else {
            WasmOutput::new(true, stdio.stdout.contents().to_vec())
        }
    };
    let evaluate_response: EvaluateResponse = serde_json::from_slice(wasm_output.data.as_slice()).unwrap();
    println!("evaluate_response:{:#?}", evaluate_response);
    // println!("result: success: \n{:#?} \nbody:\n{:#?}", wasm_output.success, String::from_utf8(wasm_output.data).unwrap());
    let first_end = now.elapsed().as_millis();
    println!("init cost:{:?}ms", first_end);
}
