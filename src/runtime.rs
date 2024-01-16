use wasmtime_wasi::preview2;

pub enum CtxBuilder {
    Preview2(preview2::WasiCtxBuilder),
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