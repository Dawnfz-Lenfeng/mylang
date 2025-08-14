use super::value::Value;
use crate::error::{Error, Result};

pub type BuiltinFn = fn(&[Value]) -> Result<Value>;

pub const BUILTIN_FUNCTIONS: &[(&str, BuiltinFn)] = &[
    ("len", builtin_len as BuiltinFn),
    ("type", builtin_type as BuiltinFn),
    ("clock", builtin_clock as BuiltinFn),
    ("assert", builtin_assert as BuiltinFn),
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
        Value::Array(arr) => Ok(Value::Number(arr.borrow().len() as f64)),
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

/// Built-in function: clock() -> number
/// Returns the current time in seconds since the UNIX epoch
pub fn builtin_clock(args: &[Value]) -> Result<Value> {
    if args.len() != 0 {
        return Err(Error::runtime(format!(
            "clock() takes no arguments ({} given)",
            args.len()
        )));
    }
    Ok(Value::Number(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
    ))
}

/// Built-in function: assert(condition, message?) -> nil
/// Throws a runtime error if condition is falsy
fn builtin_assert(args: &[Value]) -> Result<Value> {
    match args.iter().as_slice() {
        [condition, message] => {
            if !condition.is_truthy() {
                return Err(Error::assertion_with_message(message.to_string()));
            }
        }
        [condition] => {
            if !condition.is_truthy() {
                return Err(Error::assertion());
            }
        }
        _ => {
            return Err(Error::runtime(format!(
                "assert() takes 1 or 2 arguments ({} given)",
                args.len()
            )));
        }
    }

    Ok(Value::Nil)
}
