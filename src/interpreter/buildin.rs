use super::value::Value;
use crate::error::{Error, Result};

pub type BuiltinFn = fn(&[Value]) -> Result<Value>;

pub const BUILTIN_FUNCTIONS: &[(&str, BuiltinFn)] = &[
    ("len", builtin_len as BuiltinFn),
    ("type", builtin_type as BuiltinFn),
];

/// Built-in function: len(value) -> number
/// Returns the length of arrays and strings
fn builtin_len(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::runtime(format!(
            "len() takes exactly 1 argument ({} given)",
            args.len()
        )));
    }

    match &args[0] {
        Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        _ => Err(Error::runtime(format!(
            "object of type '{}' has no len()",
            args[0].type_name()
        ))),
    }
}

/// Built-in function: type(value) -> string
/// Returns the type name of the value
fn builtin_type(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::runtime(format!(
            "type() takes exactly 1 argument ({} given)",
            args.len()
        )));
    }

    Ok(Value::String(args[0].type_name().to_string()))
}
