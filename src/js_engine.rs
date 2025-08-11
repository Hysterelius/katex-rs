//! Internal abstraction over different JavaScript execution backends.
//!
//! The public API of the crate hides which JS engine is used. This module
//! defines a small trait [`JsEngine`] that each backend implements so the rest
//! of the code can render through a uniform interface. Only a subset of JS
//! features (value creation, evaluation, calling functions, and obtaining
//! strings) is required.
//!
//! Backends are selected by Cargo features:
//! * `quick-js` (default)
//! * `duktape`
//! * `wasm-js` (wasm targets only)
//!
//! This module is `pub(crate)` because the stability surface does not include
//! custom user supplied engines. If you need alternative execution semantics,
//! open an issue to discuss extending the abstraction.

use crate::error::Result;
use cfg_if::cfg_if;

/// Minimal interface a JS backend must implement.
///
/// The trait deliberately avoids exposing lifetimes originating from backend
/// internals except via the associated `JsValue` wrapper type to keep usage in
/// the rest of the crate straightforward.
pub(crate) trait JsEngine: Sized {
    /// The type of the JS value.
    type JsValue<'a>
    where
        Self: 'a;

    /// Construct a new engine instance ready to evaluate KaTeX bundles.
    fn new() -> Result<Self>;

    /// Evaluate arbitrary code in the engine (used once for bootstrapping).
    fn eval<'a>(&'a self, code: &str) -> Result<Self::JsValue<'a>>;

    /// Call a top‑level JavaScript function by name with the provided
    /// arguments. Arguments must already be JS values created by this engine.
    fn call_function<'a>(
        &'a self,
        func_name: &str,
        args: impl Iterator<Item = Self::JsValue<'a>>,
    ) -> Result<Self::JsValue<'a>>;

    /// Create a new JS boolean value.
    fn create_bool_value(&self, input: bool) -> Result<Self::JsValue<'_>>;

    /// Create a new JS integer value.
    fn create_int_value(&self, input: i32) -> Result<Self::JsValue<'_>>;

    /// Create a new JS floating point value.
    fn create_float_value(&self, input: f64) -> Result<Self::JsValue<'_>>;

    /// Create a new JS string value.
    fn create_string_value(&self, input: String) -> Result<Self::JsValue<'_>>;

    /// Create a plain JS object populated from `(key, value)` pairs.
    fn create_object_value<'a>(
        &'a self,
        input: impl Iterator<Item = (String, Self::JsValue<'a>)>,
    ) -> Result<Self::JsValue<'a>>;

    /// Convert a JS value to a UTF‑8 Rust `String`.
    fn value_to_string(&self, value: Self::JsValue<'_>) -> Result<String>;
}

cfg_if! {
    if #[cfg(feature = "quick-js")] {
        mod quick_js;

        pub(crate) type Engine = self::quick_js::Engine;
    } else if #[cfg(feature = "duktape")] {
        cfg_if! {
            if #[cfg(any(unix, windows))] {
                mod duktape;

                pub(crate) type Engine = self::duktape::Engine;
            } else {
                compile_error!("duktape backend is not support in the current build target.");
            }
        }
    } else if #[cfg(feature = "wasm-js")] {
        cfg_if! {
            if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
                mod wasm_js;

                pub(crate) type Engine = self::wasm_js::Engine;
            } else {
                compile_error!("wasm-js backend is not support in the current build target.");
            }
        }
    } else {
        compile_error!("Must enable one of the JS engines.");
    }
}
