//! JS Engine implemented by [Duktape](https://crates.io/crates/ducc).

use crate::{
    error::{Error, Result},
    js_engine::JsEngine,
};
use ducc::{FromValue, ToValue};

/// Duktape engine wrapper implementing [`JsEngine`].
pub struct Engine(ducc::Ducc);

impl JsEngine for Engine {
    type JsValue<'a> = ducc::Value<'a>;

    fn new() -> Result<Self> {
        Ok(Self(ducc::Ducc::new()))
    }

    fn eval<'a>(&'a self, code: &str) -> Result<Self::JsValue<'a>> {
        let result = self
            .0
            .exec(code, Some("katex"), ducc::ExecSettings::default())?;
        Ok(result)
    }

    fn call_function<'a>(
        &'a self,
        func_name: &str,
        args: impl Iterator<Item = Self::JsValue<'a>>,
    ) -> Result<Self::JsValue<'a>> {
        let function = self
            .0
            .globals()
            .get::<String, ducc::Function>(func_name.to_owned())?;
        let args: ducc::Values = args.collect();
        let result = function.call(args)?;
        Ok(result)
    }

    fn create_bool_value(&self, input: bool) -> Result<Self::JsValue<'_>> {
        Ok(input.to_value(&self.0)?)
    }

    fn create_int_value(&self, input: i32) -> Result<Self::JsValue<'_>> {
        Ok(input.to_value(&self.0)?)
    }

    fn create_float_value(&self, input: f64) -> Result<Self::JsValue<'_>> {
        Ok(input.to_value(&self.0)?)
    }

    fn create_string_value(&self, input: String) -> Result<Self::JsValue<'_>> {
        Ok(input.to_value(&self.0)?)
    }

    fn create_object_value<'a>(
        &'a self,
        input: impl Iterator<Item = (String, Self::JsValue<'a>)>,
    ) -> Result<Self::JsValue<'a>> {
        let obj = self.0.create_object();
        for (k, v) in input {
            obj.set(k, v)?;
        }
        Ok(ducc::Value::Object(obj))
    }

    fn value_to_string(&self, value: Self::JsValue<'_>) -> Result<String> {
        Ok(String::from_value(value, &self.0)?)
    }
}

impl From<ducc::Error> for Error {
    fn from(e: ducc::Error) -> Self {
        use ducc::ErrorKind;

        match e.kind {
            ErrorKind::ToJsConversionError { .. } | ErrorKind::FromJsConversionError { .. } => {
                Self::JsValueError(format!("{e}"))
            }
            _ => Self::JsExecError(format!("{e}")),
        }
    }
}
