#![feature(lazy_cell)]

use std::{env, fs};
use std::ops::Deref;
use std::sync::LazyLock;
use std::time::Instant;

use wasmtime::{component, Config, Engine, Linker, Module, Store};
use wasmtime::component::Component;
use wasmtime_wasi::{ambient_authority, Dir, preview2};
use wasmtime_wasi::preview2::{DirPerms, FilePerms, WasiCtxBuilder};
use crate::errors::RuntimeError;

mod errors;

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
    }else {
        Err(RuntimeError::InvalidWrapper)
    };
    module_or_component.unwrap()
}


// #[async_std::main]
#[tokio::main]
async fn main() {
    println!("Hello, world!");
    for _i in 0..10 {
        run().await;
    }
    // drop(store);
}


pub async fn run() {
    let now = Instant::now();
    let engine = ENGINE.deref();

    // let mut path_buf = env::current_dir().unwrap();
    // path_buf.push("javy-demo.wasm");
    // println!("{:#?}", &path_buf);
    // let bytes = fs::read(path_buf).unwrap();
    //
    // let first_end = now.elapsed().as_millis();
    // println!("read file cost:{:?}ms", first_end);
    // let now = Instant::now();
    // let component = Component::from_binary(engine, &bytes).expect("load component error");

    let wasi_ctx = WasiCtxBuilder::new()
        // .envs()
        .inherit_stdin()
        .inherit_stdout()
        .inherit_stderr()
        .preopened_dir(
            Dir::open_ambient_dir(env::current_dir().unwrap(),
                                  ambient_authority()).unwrap(),
            DirPerms::all(),
            FilePerms::all(),
            ".",
        )
        .build();

    let mut store = Store::new(&engine, WasiHostCtx {
        preview2_ctx: wasi_ctx,
        preview2_table: preview2::Table::default(),
        preview1_adapter: preview2::preview1::WasiPreview1Adapter::new(),
    });

    let module_or_component = parse_module_or_component("javy-demo.wasm");

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

    let first_end = now.elapsed().as_millis();
    println!("init cost:{:?}ms", first_end);
    let now = Instant::now();

    // let bytes = include_bytes!("../javy-demo.wasm").to_vec();


    let first_end = now.elapsed().as_millis();
    println!("init 1 cost:{:?}ms", first_end);
    let now = Instant::now();
}
