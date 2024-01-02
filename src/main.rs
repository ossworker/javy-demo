#![feature(lazy_cell)]

use std::ops::Deref;
use std::sync::{Arc, LazyLock};
use wasmtime::component::Component;
use wasmtime::{component, Config, Engine, Store};
use wasmtime_wasi::{Dir, ambient_authority, preview2};
use wasmtime_wasi::preview2::{DirPerms, FilePerms, Table, WasiCtx, WasiCtxBuilder};

static ENGINE: LazyLock<Engine> = LazyLock::new(||{
    let mut config = Config::default();
    config.async_support(true)
        .wasm_component_model(true)
        .wasm_multi_memory(true)
        .wasm_threads(true);
    Engine::new(&config).unwrap()
});

#[derive(Default)]
struct Host {
    pub wasi_preview2_ctx: Option<Arc<WasiCtx>>,
    wasi_preview2_table: Arc<Table>,
}

impl preview2::WasiView for Host{
    fn table(&self) -> &Table {
        &self.wasi_preview2_table
    }

    fn table_mut(&mut self) -> &mut Table {
        Arc::get_mut(&mut self.wasi_preview2_table)
            .expect("preview2 is not compatiable with threads")
    }

    fn ctx(&self) -> &WasiCtx {
        self.wasi_preview2_ctx.as_ref().unwrap()
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        let ctx = self.wasi_preview2_ctx.as_mut().unwrap();
        Arc::get_mut(ctx).expect("preview2 is not compatiable with threads")
    }
}


#[async_std::main]
async fn main() {
    println!("Hello, world!");
    let engine = ENGINE.deref();

    let bytes = include_bytes!("../javy-demo.wasm").to_vec();
    let component =
    // if wasmparser::Parser::is_core_wasm(&bytes) {
        // println!("is core wasm");
        // wasmtime::Module::from_binary(&engine, &bytes).expect("load module error")
    // }
    // else if wasmparser::Parser::is_component(&bytes) {
    //     println!("is component");
        Component::from_binary(engine, &bytes).expect("load component error");
    // } else {
        // Err("not support")
    // };

    let wasi_ctx = WasiCtxBuilder::new()
        // .envs()
        .inherit_stdin()
        .inherit_stdout()
        .inherit_stderr()
        .preopened_dir(
            Dir::open_ambient_dir(std::env::current_dir().unwrap(),
                                  ambient_authority()).unwrap(),
            DirPerms::all(),
            FilePerms::all(),
            ".",
        ).build();
    let table = Table::default();
    let host = Host {
        wasi_preview2_ctx: Some(Arc::new(wasi_ctx)),
        wasi_preview2_table: Arc::new(table),
    };

    let mut store = Store::new(&engine, host);

    let mut component_linker = component::Linker::new(&engine);

    preview2::command::add_to_linker(&mut component_linker).unwrap();

    let (comand,_instance) = preview2::command::Command::instantiate_async(
        &mut store,
        &component,
        &component_linker,
    ).await.unwrap();

    let _ = comand
        .wasi_cli_run()
        .call_run(&mut store)
        .await
        .unwrap();
    // drop(store);
}
