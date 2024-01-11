use std::io::Bytes;
use wasmtime_wasi::preview2;
use wasmtime_wasi::preview2::WasiCtxBuilder;

pub struct Stdio {
    pub stdin: Vec<u8>,
    pub stdout: preview2::pipe::MemoryOutputPipe,
    pub stderr: preview2::pipe::MemoryOutputPipe,
}

impl Stdio {
    pub fn new(stdio: Vec<u8>) -> Self {
        let stdout = preview2::pipe::MemoryOutputPipe::new(1024);
        let stderr = preview2::pipe::MemoryOutputPipe::new(1024);
        Stdio { stdin, stdout, stderr }
    }

    pub fn configure_wasi_ctx(&self, ref mut wasi_ctx_builder: &mut WasiCtxBuilder) -> &mut WasiCtxBuilder {
        wasi_ctx_builder
            .stdin(
                preview2::pipe::MemoryInputPipe::new(self.stdin.clone().into()),
            )
            .stdout(
                self.stdout.clone(),
            )
            .stderr(
                self.stderr.clone(),
            )
    }
}