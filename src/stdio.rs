use wasmtime_wasi::preview2;

use crate::runtime::CtxBuilder;

pub struct Stdio {
    pub stdin: Vec<u8>,
    pub stdout: preview2::pipe::MemoryOutputPipe,
    pub stderr: preview2::pipe::MemoryOutputPipe,
}

impl Stdio {
    pub fn new(stdin: Vec<u8>) -> Self {
        let stdout = preview2::pipe::MemoryOutputPipe::new(1024);
        let stderr = preview2::pipe::MemoryOutputPipe::new(1024);
        Stdio { stdin, stdout, stderr }
    }

    pub fn configure_wasi_ctx(&self, mut builder: CtxBuilder) -> CtxBuilder {
        match  builder{
            CtxBuilder::Preview2(ref mut builder) => {
                builder
                    .stdin(
                        preview2::pipe::MemoryInputPipe::new(self.stdin.clone().into()),
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