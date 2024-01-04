#![feature(lazy_cell)]

mod runtime;
mod errors;
mod modules;

use std::{env, fs};
use std::ops::Deref;
use std::sync::LazyLock;
use std::time::Instant;

use wasmtime::{component, Config, Engine, Store};
use wasmtime::component::Component;
use wasmtime_wasi::{ambient_authority, Dir, preview2};
use wasmtime_wasi::preview2::{DirPerms, FilePerms, Table, WasiCtx, WasiCtxBuilder};

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

static COMPONENT: LazyLock<Component> = LazyLock::new(|| {
    let mut path_buf = env::current_dir().unwrap();
    path_buf.push("javy-demo.wasm");
    println!("{:#?}", &path_buf);
    let bytes = fs::read(path_buf).unwrap();
    Component::from_binary(ENGINE.deref(), &bytes).expect("load component error")
});

// #[derive(Default)]
struct Host {
    pub wasi_preview2_ctx: WasiCtx,
    wasi_preview2_table: Table,
}

impl preview2::WasiView for Host {
    fn table(&self) -> &Table {
        &self.wasi_preview2_table
    }

    fn table_mut(&mut self) -> &mut Table {
        &mut self.wasi_preview2_table
    }

    fn ctx(&self) -> &WasiCtx {
        &self.wasi_preview2_ctx
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi_preview2_ctx
    }
    // fn table(&self) -> &Table {
    //     &self.wasi_preview2_table
    // }
    //
    // fn table_mut(&mut self) -> &mut Table {
    //     Arc::get_mut(&mut self.wasi_preview2_table)
    //         .expect("preview2 is not compatiable with threads")
    // }
    //
    // fn ctx(&self) -> &WasiCtx {
    //     self.wasi_preview2_ctx.as_ref().unwrap()
    // }
    //
    // fn ctx_mut(&mut self) -> &mut WasiCtx {
    //     let ctx = self.wasi_preview2_ctx.as_mut().unwrap();
    //     Arc::get_mut(ctx).expect("preview2 is not compatiable with threads")
    // }
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
    let component = COMPONENT.deref();

    let first_end = now.elapsed().as_millis();
    println!("init cost:{:?}ms", first_end);
    let now = Instant::now();

    // let bytes = include_bytes!("../javy-demo.wasm").to_vec();

    let mut component_linker = component::Linker::new(&engine);

    let first_end = now.elapsed().as_millis();
    println!("init 1 cost:{:?}ms", first_end);
    let now = Instant::now();

    preview2::command::add_to_linker(&mut component_linker).unwrap();

    let first_end = now.elapsed().as_millis();
    println!("init 2 cost:{:?}ms", first_end);
    let now = Instant::now();

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
    let table = Table::default();

    let mut store = Store::new(&engine, Host {
        wasi_preview2_ctx: wasi_ctx,
        wasi_preview2_table: table,
    });


    let first_end = now.elapsed().as_millis();
    println!("init 3 cost:{:?}ms", first_end);

    let now = Instant::now();

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

    let second_end = now.elapsed().as_millis();
    println!("end cost:{:?}ms", second_end);
}
