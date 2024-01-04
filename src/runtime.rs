use wasmtime::component::Component;
use crate::errors::Result;

pub trait Runtime {
    ///init

    fn init(&self) -> Result<()> {
        Ok(())
    }
    /// load component
    fn load_wasm_component(&self) -> Component;
    /// prepare
    
    /// execute
}