// use wasmtime::{Config, Engine};

fn main() {
    println!("Hello, world!");
    // let mut config = Config::default()
    //     .async_support(true)
    //     .wasm_component_model(true);
    //
    // let engine = Engine::new(config).unwrap();

    let _bytes = include_bytes!("../javy-demo.wasm").to_vec();

    // if wasmparser::Parser::is_core_wasm(&bytes) {
    //     println!("is core wasm");
    //     wasmtime::Module::from_binary(&engine, &bytes).expect("load module error");
    // } else if wasmparser::Parser::is_component(&bytes) {
    //     println!("is component");
    //     Component::from_binary(&engine, &bytes).expect("load component error")
    // } else {
    //     Err("not support")
    // }

    // let wasi_ctx = WasiCtxBuilder::new()
    //     // .envs()
    //     .inherit_stdin()
    //     .inherit_stdout()
    //     .inherit_stderr()
    //     .preopened_dir(
    //         Dir::open_ambient_dir(std::env::current_dir(), ambient_authority()).unwrap(),
    //         DirPerms::all(),
    //         FilePerms::all(),
    //         ".",
    //     ).build();
    // let mut table = preview2::Table::default();
    //
    // let mut component_linker = component::Linker::new(&engine);
    // preview2::command::add_to_linker(&mut component_linker).unwrap();


}
