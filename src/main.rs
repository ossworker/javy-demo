use std::collections::HashMap;
use std::env;
use std::io::{stderr, stdin, stdout, Read, Write};
use std::sync::{LazyLock, OnceLock};

use javy::{json, quickjs, Config, Runtime};
use regex::Regex;
use serde::Deserialize;

mod handler;

const EXPOSED_PREFIX: &'static str = "ZEN_EXPOSED_";
// JS polyfill
static POLYFILL: &str = include_str!("../shims/index.js");

// static mut RUNTIME: OnceLock<Runtime> = OnceLock::new();

static mut RUNTIME: LazyLock<Runtime> = LazyLock::new(|| precompile());

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let _ = unsafe { &*RUNTIME };
}

fn precompile() -> Runtime {
    let runtime = Runtime::new(Default::default()).unwrap();
    // Precompile the Polyfill to bytecode
    let context = runtime.context();

    let _dayjs_src = runtime
        .compile_to_bytecode("dayjs.js", include_str!("script/dayjs.js"))
        .unwrap();

    let _dayjs_src = runtime
        .compile_to_bytecode("big.js", include_str!("script/big.js"))
        .unwrap();

    let bytecode = runtime
        .compile_to_bytecode("polyfill.js", POLYFILL)
        .unwrap();
    // Preload it
    // let _ = context.eval_binary(&bytecode).expect("load polyfill error");
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

#[derive(Debug, Deserialize)]
struct WasmInput {
    js_content: String,
    body: String,
}

fn main() {
    let runtime = unsafe { &*RUNTIME };
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
            "globalThis.now = Date.now();",
            serde_json::to_string(&env_vars).unwrap()
        )
    };

    stdin().read_to_string(&mut request).unwrap();

    // context.with(|ctx| {
    //    ctx.eval_with_options(env_src_string, Default::default()).unwrap();
    // });

    let input: WasmInput = serde_json::from_str(&request).unwrap();

    contents.push_str(&input.js_content);

    match identify_type(&contents) {
        JSWorkerType::DefaultExport => {
            context.with(|ctx| {
                // let globals = ctx.globals();
                // ctx.eval_with_options(&*contents, Default::default()).unwrap();
                quickjs::Module::evaluate(
                    ctx.clone(),
                    "runtime.mjs",
                    "import {default as handler} from 'handler.mjs';__addHandler(handler.handler);",
                )
                .unwrap();
            });
        }
        _ => {
            // context.with(|ctx| {
            //     ctx.eval_with_options(&format!("{};__addHandler(handler);", contents), Default::default()).unwrap();
            // });
        }
    }

    // let global = context.global_object().unwrap();
    // let entrypoint = global.get_property("entrypoint").unwrap();
    //
    // let input_bytes = input.body.as_bytes();
    // let input_value = json::parse(context, input_bytes).unwrap();
    //
    // match entrypoint.call(&global, &[input_value]) {
    //     Ok(_) => {}
    //     Err(err) => eprintln!("Error calling the main entrypoint: {err}"),
    // }
    //
    //
    // if runtime.has_pending_jobs() {
    //     if let Err(err) = runtime.resolve_pending_jobs() {
    //         eprintln!("Error running async methods: {err}");
    //     }
    // }
    //
    //
    //
    // let global = context.global_object().unwrap();
    // let error_value = global.get_property("error").unwrap();
    // let output_value = global.get_property("result").unwrap();
    //
    // if !error_value.is_null_or_undefined() {
    //     let error = json::stringify(error_value).unwrap();
    //     stderr().write_all(&error.as_slice()).expect("js error");
    //     return;
    // }
    // let output = json::stringify(output_value).unwrap();
    // stdout()
    //     .write_all(&output.as_slice())
    //     .expect("Error when returning the response");
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Error};
    use javy::quickjs::context::EvalOptions;
    use javy::quickjs::function::{IntoArgs, MutFn, Rest};
    use javy::quickjs::{Ctx, Function, String as JString, String, Value};
    use javy::{
        from_js_error, hold, hold_and_release, json, to_js_error, to_string_lossy, val_to_string,
        Config, Runtime,
    };
    use std::env::vars;

    #[test]
    fn test_javy() {
        let runtime = Runtime::default();
        let context = runtime.context();


        context.with(|cx| {
            let globals = cx.globals();
            globals.set(
                "print_hello",
                Function::new(
                    cx.clone(),
                    MutFn::new(move |cx: Ctx, args: Rest<Value>| {
                        println!("hello")
                    }),
                ).expect("1111111"),
            ).expect("22222");
        });

        context.with(|this| {
            let mut eval_opts = EvalOptions::default();
            let f: Function = this
                .eval("() => { console.log(JSON.stringify({id:111})); return 42;}")
                .unwrap();

            let res: i32 = ().apply(&f).unwrap();
            assert_eq!(res, 42);

            let res: i32 = f.call(()).unwrap();
            assert_eq!(res, 42);
            eval_opts.strict = false;

            let json_fun: Function = this
                .eval(
                    r#"() => {
                console.log("----");
                //console.log("="+JSON.stringify({id:111}));
                const jsonStr = JSON.stringify({id:111});
                console.log("=="+jsonStr);
                return "111";
            }
            "#,
                )
                .unwrap();

            // let result: String = json_fun.call(()).unwrap();
            let result = json_fun.call::<(), String>(()).unwrap();
            println!("{:#?}", result);
        });
    }

    #[test]
    fn test_random() -> anyhow::Result<()> {
        let mut config = Config::default();
        config.operator_overloading(true);

        let runtime = Runtime::new(config).expect("runtime to be created");
        runtime.context().with(|this| {
            let mut eval_opts = EvalOptions::default();
            eval_opts.strict = false;
            this.eval_with_options("result = Math.random()", eval_opts)?;
            let result: f64 = this
                .globals()
                .get::<&str, Value<'_>>("result")?
                .as_float()
                .unwrap();

            println!("{:#?}", result);

            assert!(result >= 0.0);
            assert!(result < 1.0);

            let quickjs_result: javy::quickjs::Result<String> = this
                .eval_with_options("result = JSON.stringify({id:111});", EvalOptions::default());

            let binding: Value = this.globals().get::<&str, Value<'_>>("result").unwrap();

            let err_msg = val_to_string(&this, this.catch()).unwrap();

            // let str = String::from_utf8(json::stringify(binding)?)?;
            // // let str = JString::from_str(this.clone(), &str)?;
            println!("global var: {:#?}", val_to_string(&this, binding).unwrap());

            println!("{:#?} {:#?}", quickjs_result.is_ok(), err_msg);

            let json_fun: Function = this
                .clone()
                .eval(
                    r#"() => {
                const st = JSON.stringify({id:111});
                console.log(st+"--");
                return st;
             }
         "#,
                )
                .unwrap();
            let result: String = json_fun.call::<(), String>(()).unwrap();
            println!("fun:{:#?}", result);

            Ok::<_, Error>(())
        })?;

        Ok(())
    }
}
