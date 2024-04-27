pub struct Stdio {
    pub stdin: Vec<u8>,
    pub stdout: wasmtime_wasi::pipe::MemoryOutputPipe,
    pub stderr: wasmtime_wasi::pipe::MemoryOutputPipe,
}

impl Stdio {
    pub fn new(stdin: Vec<u8>) -> Self {
        let stdout = wasmtime_wasi::pipe::MemoryOutputPipe::new(1024);
        let stderr = wasmtime_wasi::pipe::MemoryOutputPipe::new(1024);
        Stdio {
            stdin,
            stdout,
            stderr,
        }
    }
}
