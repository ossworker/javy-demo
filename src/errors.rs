pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    CannotReadModule,
    InvalidExtension { extension: Option<String> },
    InvalidWrapper,
    IOError(std::io::Error),
    MissingRuntime { extension: String },
    WasiContextError { error: String },
    WasiError(Option<wasmtime_wasi::Error>),
}