use std::collections::HashMap;
use std::env;
use std::io::{Read, stderr, stdin, stdout, Write};
use std::sync::OnceLock;

use crate::jsbindings::{load_bindings_into_global, RuntimeError};
use javy::{json, Runtime};
use regex::Regex;
use serde::Deserialize;

mod jsbindings;

mod handler;

const EXPOSED_PREFIX: &'static str = "ZEN_EXPOSED_";
// JS polyfill
static POLYFILL: &str = include_str!("../shims/index.js");

static mut RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let runtime = precompile();
    unsafe { RUNTIME.set(runtime).unwrap() };
}

fn precompile() -> Runtime {
    let runtime = Runtime::default();
    // Precompile the Polyfill to bytecode
    let context = runtime.context();

    let dayjs_src = context
        .compile_global("dayjs.js", include_str!("script/dayjs.js"))
        .unwrap();
    context.eval_binary(&dayjs_src).expect("load dayjs error");

    let dayjs_src = context
        .compile_global("big.js", include_str!("script/big.js"))
        .unwrap();
    context.eval_binary(&dayjs_src).expect("load dayjs error");

    let bytecode = context.compile_global("polyfill.js", POLYFILL).unwrap();
    // Preload it
    let _ = context.eval_binary(&bytecode);
    runtime
}

enum JSWorkerType {
    Global,
    DefaultExport,
}

fn identify_type(src: &str) -> JSWorkerType {
    // Detect default exported functions and objects
    let default_regex = Regex::new(r"(?-u)export\s+default\s+\w+;?").unwrap();
    // Detect default exported object
    let default_block_regex = Regex::new(r"export\s+default\s*\{([\s\n\r]*.*)+\};?").unwrap();
    // Detect exported functions with the "as" syntax like "export { app as default }";
    let default_as_regex =
        Regex::new(r"(?-u)export\s*\{[\s\n\r]*\w+\s+(as default){1}[\s\n\r]*\};?").unwrap();
    if default_regex.is_match(src)
        || default_block_regex.is_match(src)
        || default_as_regex.is_match(src)
    {
        JSWorkerType::DefaultExport
    } else {
        JSWorkerType::Global
    }
}

#[derive(Debug,Deserialize)]
struct WasmInput{
    js_content: String,
    input: String,
}

fn main() {
    let runtime = unsafe { RUNTIME.get_or_init(precompile) };
    let context = runtime.context();

    let mut request = String::new();

    let mut contents = String::new();

    let env_src_string = {
        let mut env_vars = HashMap::new();
        env::vars().for_each(|(key, value)| {
            if let Some(mod_key) = key.strip_prefix(EXPOSED_PREFIX) {
                env_vars.insert(mod_key.to_string(), value);
            }
        });
        env_vars.insert(String::from("key"), String::from("value"));
        format!(
            "{};const __GLOBAL__ENV = {};",
            "__setNowDate(Date.now())",
            serde_json::to_string(&env_vars).unwrap()
        )
    };

    stdin().read_to_string(&mut request).unwrap();

    // contents.push_str(&env_src_string);
    context
        .eval_global("__GLOBAL__ENV", &env_src_string)
        .unwrap();



    // let handler_str = "console.log('11111'); result='{\"code\":\"1111\"}'";

    // handler.js
    //     let handler_str = "const handler = (input, { dayjs, Big, moment, env }) => {
    //     console.log('input',input);
    //     const momentValid = typeof moment === 'function' && Object.keys(moment).includes('isDayjs');
    //     const dayjsValid = typeof dayjs === 'function' && Object.keys(moment).includes('isDayjs');
    //     const bigjsValid = typeof Big === 'function';
    //
    //     return {
    //         momentValid,
    //         dayjsValid,
    //         bigjsValid,
    //         bigjsTests: [
    //             Big(0.1).add(0.2).eq(0.3),
    //             Big(123.12).mul(0.1).round(2).eq(12.31),
    //         ],
    //         env
    //     };
    // };";

    let input: WasmInput = serde_json::from_str(&request).unwrap();

    println!("--{:#?}--",&input);

    contents.push_str(&input.js_content);

    let global = context.global_object().unwrap();

    match load_bindings_into_global(context, global) {
        Ok(_) => {}
        Err(e) => match e {
            RuntimeError::InvalidBinding { invalid_export } => {
                eprintln!("There was an error adding the '{invalid_export}' binding");
            }
        },
    }

    match identify_type(&contents) {
        JSWorkerType::DefaultExport => {
            let _ = context.eval_module("handler.mjs", &contents).unwrap();
            let _ = context
                .eval_module(
                    "runtime.mjs",
                    "import {default as handler} from 'handler.mjs';__addHandler(handler.handler);",
                )
                .unwrap();
        }
        _ => {
            context
                .eval_global(
                    "handler.js",
                    &format!("{};__addHandler(handler);", contents),
                )
                .unwrap();
        }
    }

    let global = context.global_object().unwrap();
    let entrypoint = global.get_property("entrypoint").unwrap();

    // request.push_str("{\"id\":\"abc\",\"name\":\"张三\"}");

    let input_bytes = input.input.as_bytes();
    let input_value = json::transcode_input(context, input_bytes).unwrap();

    match entrypoint.call(&global, &[input_value]) {
        Ok(_) => {}
        Err(err) => eprintln!("Error calling the main entrypoint: {err}"),
    }

    if context.is_pending() {
        if let Err(err) = context.execute_pending() {
            eprintln!("Error running async methods: {err}");
        }
    }

    let global = context.global_object().unwrap();
    let error_value = global.get_property("error").unwrap();
    let output_value = global.get_property("result").unwrap();

    if !error_value.is_null_or_undefined() {
        let error = error_value.as_str_lossy();
        eprintln!("jsError:{}", &error);
        stderr().write_all(&error.as_bytes()).expect("js error");
    }


    let output = json::transcode_output(output_value).unwrap();
    stdout()
        .write_all(&output)
        .expect("Error when returning the response");
}
