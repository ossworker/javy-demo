use wasmtime_wasi::preview2;

pub struct Stdio {
    pub stdio: Vec<u8>,
    pub stdout: preview2::pipe::MemoryOutputPipe,
}

impl Stdio {
    pub fn new(stdio: Vec<u8>) -> Self {
        let stdout = preview2::pipe::MemoryOutputPipe::new(1024);
        Stdio { stdio, stdout }
    }
}