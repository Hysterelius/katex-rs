//! Error handling for the `katex` crate.
//!
//! The crate exposes a single [`Error`] enum which groups the different
//! categories of failures that can occur while initialising or interacting
//! with the underlying JavaScript engine as well as converting values to / from
//! JS. All public, fallible APIs return a [`Result<T, Error>`].
//!
//! Error variants are intentionally coarse‑grained so that downstream users
//! can either pattern‑match to distinguish between *initialisation* problems
//! (often unrecoverable), *execution* problems (e.g. KaTeX parse errors), and
//! *value* conversion issues (usually a bug or unsupported type), or simply
//! bubble them up with `?`.

/// Error type for this crate.
#[non_exhaustive]
#[derive(thiserror::Error, Clone, Debug)]
pub enum Error {
    /// Failure while creating / initialising the selected JavaScript engine.
    ///
    /// Examples include: inability to allocate a runtime, backend‑specific
    /// setup errors, or platform limitations. Retrying is unlikely to succeed
    /// unless the underlying resource constraints change.
    #[error("failed to initialize js environment (detail: {0})")]
    JsInitError(String),
    /// Failure reported while evaluating KaTeX / Temml code or executing a
    /// render call.
    ///
    /// This encompasses both *logical* errors (such as KaTeX parse errors for
    /// invalid LaTeX when `throw_on_error` is true) and *runtime* JS failures.
    /// The string payload contains the (minified) message returned by the
    /// underlying engine.
    #[error("failed to execute js (detail: {0})")]
    JsExecError(String),
    /// Failure converting between host (Rust) values and JS values.
    ///
    /// Generally indicates a bug, unsupported type coercion, or encoding
    /// problem (e.g. invalid UTF‑8). These are not typically caused by user
    /// LaTeX input.
    #[error("failed to convert js value (detail: {0})")]
    JsValueError(String),
}

/// Convenient alias used throughout the crate.
///
/// This corresponds to `core::result::Result<T, katex::Error>`.
pub type Result<T, E = Error> = core::result::Result<T, E>;
