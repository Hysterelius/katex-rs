//! JS Engine implemented by [QuickJs](https://crates.io/crates/rquickjs).

use rquickjs::IteratorJs;

use crate::{
    error::{Error, Result},
    js_engine::JsEngine,
};

#[derive(Debug)]
pub(crate) struct Value(rquickjs::Persistent<rquickjs::Value<'static>>);

pub type Engine = rquickjs::Context;

impl JsEngine for Engine {
    type JsValue<'a> = Value;

    fn new() -> Result<Self> {
        let runtime = rquickjs::Runtime::new()?;
        Ok(rquickjs::Context::full(&runtime)?)
    }

    fn eval<'a>(&'a self, code: &str) -> Result<Self::JsValue<'a>> {
        self.with(|ctx| {
            Ok(Value(rquickjs::Persistent::<rquickjs::Value>::save(
                &ctx,
                ctx.eval(code)?,
            )))
        })
    }

    fn call_function<'a>(
        &'a self,
        func_name: &str,
        args: impl Iterator<Item = Self::JsValue<'a>>,
    ) -> Result<Self::JsValue<'a>> {
        let args: Vec<_> = args.collect(); // needed to avoid re-entrant borrow of `ctx`
        self.with(|ctx| {
            let func: rquickjs::Function<'_> = ctx.globals().get(func_name)?;
            let mut qjs_args = rquickjs::function::Args::new_unsized(ctx.clone());
            qjs_args.push_args(args.into_iter().map(|arg: Value| arg.0.restore(&ctx)))?;

            let result = func.call_arg(qjs_args)?;
            Ok(Value(rquickjs::Persistent::<rquickjs::Value>::save(
                &ctx, result,
            )))
        })
    }

    fn create_bool_value(&self, input: bool) -> Result<Self::JsValue<'_>> {
        self.with(|ctx| {
            let value = rquickjs::Value::new_bool(ctx.clone(), input);
            Ok(Value(rquickjs::Persistent::<rquickjs::Value>::save(
                &ctx, value,
            )))
        })
    }

    fn create_int_value(&self, input: i32) -> Result<Self::JsValue<'_>> {
        self.with(|ctx| {
            let value = rquickjs::Value::new_int(ctx.clone(), input);
            Ok(Value(rquickjs::Persistent::<rquickjs::Value>::save(
                &ctx, value,
            )))
        })
    }

    fn create_float_value(&self, input: f64) -> Result<Self::JsValue<'_>> {
        self.with(|ctx| {
            let value = rquickjs::Value::new_float(ctx.clone(), input);
            Ok(Value(rquickjs::Persistent::<rquickjs::Value>::save(
                &ctx, value,
            )))
        })
    }

    fn create_string_value(&self, input: String) -> Result<Self::JsValue<'_>> {
        self.with(|ctx| {
            let value = rquickjs::String::from_str(ctx.clone(), &input)?.into();
            Ok(Value(rquickjs::Persistent::<rquickjs::Value>::save(
                &ctx, value,
            )))
        })
    }

    fn create_object_value<'a>(
        &'a self,
        input: impl Iterator<Item = (String, Self::JsValue<'a>)>,
    ) -> Result<Self::JsValue<'a>> {
        let input: Vec<_> = input.collect(); // needed to avoid re-entrant borrow of `ctx`
        self.with(|ctx| {
            let obj: rquickjs::Object = input
                .into_iter()
                .map(|(s, val)| (s, val.0.restore(&ctx)))
                .collect_js(&ctx)?;
            Ok(Value(rquickjs::Persistent::<rquickjs::Value>::save(
                &ctx,
                obj.into(),
            )))
        })
    }

    fn value_to_string(&self, value: Self::JsValue<'_>) -> Result<String> {
        self.with(|ctx| {
            let v: rquickjs::Value = value.0.restore(&ctx)?;
            Ok(v.into_string()
                .ok_or_else(|| Error::JsValueError("failed to convert value to string".to_owned()))?
                .to_string()?)
        })
    }
}

impl From<rquickjs::Error> for Error {
    fn from(e: rquickjs::Error) -> Self {
        (&e).into()
    }
}

impl From<&'_ rquickjs::Error> for Error {
    fn from(e: &'_ rquickjs::Error) -> Self {
        match e {
            rquickjs::Error::Allocation => Error::JsInitError(e.to_string()),
            rquickjs::Error::InvalidString(_)
            | rquickjs::Error::InvalidCStr(_)
            | rquickjs::Error::Utf8(_)
            | rquickjs::Error::FromJs { .. }
            | rquickjs::Error::IntoJs { .. }
            | rquickjs::Error::AsSlice(_) => Error::JsValueError(e.to_string()),
            _ => Error::JsExecError(e.to_string()),
        }
    }
}
