
use crate::runtime::CtxBuilder;

pub struct Stdio {
    pub stdin: Vec<u8>,
    pub stdout: wasmtime_wasi::pipe::MemoryOutputPipe,
    pub stderr: wasmtime_wasi::pipe::MemoryOutputPipe,
}

impl Stdio {
    pub fn new(stdin: Vec<u8>) -> Self {
        let stdout = wasmtime_wasi::pipe::MemoryOutputPipe::new(1024);
        let stderr = wasmtime_wasi::pipe::MemoryOutputPipe::new(1024);
        Stdio { stdin, stdout, stderr }
    }

    pub fn configure_wasi_ctx(&self, mut builder: CtxBuilder) -> CtxBuilder {
        match  builder{
            CtxBuilder::Preview2(ref mut builder) => {
                builder
                    .stdin(
                        wasmtime_wasi::pipe::MemoryInputPipe::new(self.stdin.clone().into()),
                    )
                    .stdout(
                        self.stdout.clone(),
                    )
                    .stderr(
                        self.stderr.clone(),
                    );
            }
        }
        builder
    }
}