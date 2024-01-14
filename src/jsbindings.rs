use javy::quickjs::{JSContextRef, JSValue, JSValueRef};

#[derive(Debug)]
pub enum RuntimeError {
    InvalidBinding { invalid_export: String },
}

pub fn load_bindings_into_global(
    context: &JSContextRef,
    global: JSValueRef,
) -> Result<(), RuntimeError> {
    global
        .set_property(
            "__console_log",
            context
                .wrap_callback(|_ctx, _this_arg, args| {
                    for arg in args {
                        eprintln!("{:#?}", arg.to_string());
                    }
                    Ok(JSValue::Null)
                })
                .map_err(|_| RuntimeError::InvalidBinding {
                    invalid_export: "console.log".to_string(),
                })?,
        )
        .map_err(|_| RuntimeError::InvalidBinding {
            invalid_export: "console.log".to_string(),
        })?;

    Ok(())
}
