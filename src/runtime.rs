pub enum CtxBuilder {
    Preview2(wasmtime_wasi::WasiCtxBuilder),
}

// pub trait Runtime {
//     ///init
//
//     fn init(&self) -> Result<()> {
//         Ok(())
//     }
//     /// load component
//     fn load_wasm_component(&self) -> Component;
//     /// prepare
//
//     /// execute
// }
