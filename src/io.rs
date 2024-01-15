use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WasmInput<'a> {
    js_content: &'a str,
    body: &'a str,
}

impl<'a> WasmInput<'a> {
    pub fn new(
        js_content: &'a str,
        body: &'a str,
    ) -> Self {
        Self {
            js_content,
            body,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct WasmOutput {
    pub(crate) success: bool,
    pub data: Vec<u8>,
}

impl WasmOutput {
    pub fn new(success: bool, body: Vec<u8>) -> Self {
        Self {
            success,
            data: body,
        }
    }
}