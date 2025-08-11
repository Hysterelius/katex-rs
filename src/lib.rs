//! Rust bindings to the high‑performance math typesetting library
//! [KaTeX](https://katex.org). This crate lets you render LaTeX (and a
//! subset of TeX) expressions into HTML (and/or MathML) entirely in memory
//! without spawning external processes.
//!
//! Rendering is performed by executing the KaTeX (or, when requested,
//! [Temml](https://temml.org)) JavaScript bundle inside an embedded JS
//! engine. Several engines are supported via Cargo features (see below).
//!
//! ## Quick start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! katex = "0.4"
//! ```
//!
//! Then render some math:
//!
//! ```
//! let html_fragment = katex::render(r"E = mc^2").unwrap();
//! assert!(html_fragment.contains("katex"));
//! ```
//!
//! Or configure options with the builder:
//!
//! ```
//! let opts = katex::Opts::builder()
//!     .display_mode(true)
//!     .output_type(katex::OutputType::HtmlAndMathml)
//!     .build()
//!     .unwrap();
//! let html = katex::render_with_opts(r"\\frac{a}{b}", &opts).unwrap();
//! assert!(html.contains("katex-display"));
//! ```
//!
//! ## Feature flags / backends
//!
//! Exactly one JS execution backend must be enabled. The default backend is
//! `quick-js`.
//!
//! * `quick-js` *(default)* – Uses [rquickjs](https://crates.io/crates/rquickjs)
//!   (QuickJS) for fast, embeddable execution.
//! * `duktape` – Uses [ducc](https://crates.io/crates/ducc) (Duktape). Disable
//!   default features first: `default-features = false, features = ["duktape"]`.
//! * `wasm-js` – Uses a browser / wasm environment via
//!   [wasm-bindgen](https://crates.io/crates/wasm-bindgen) +
//!   [js-sys](https://crates.io/crates/js-sys). Only valid for `wasm32-unknown-unknown`.
//! * `temml` – When combined with `OutputType::Mathml`, use the
//!   [Temml](https://temml.org) library (KaTeX compatible) to produce concise
//!   MathML output. Falls back to KaTeX for other output types.
//!
//! ## Threading & caching
//!
//! A JavaScript engine instance is created lazily per thread and then reused
//! for subsequent renders on that thread (using a thread‑local). This keeps
//! rendering cheap after the first call. Each thread therefore maintains its
//! own isolated JS context – there is no cross‑thread mutation.
//!
//! ## Error handling
//!
//! All fallible APIs return [`Result<T, Error>`]. Distinct error variants
//! differentiate between: engine initialisation, JavaScript execution, and
//! value conversion issues. Parse errors from KaTeX itself surface as the
//! `JsExecError` variant with a message produced by KaTeX.
//!
//! ## Performance notes
//!
//! * The first render on a thread pays the cost of bootstrapping and loading
//!   the (minified) JS bundle.
//! * Subsequent renders only invoke pure JS functions and are typically fast.
//! * If you render in many short‑lived threads you will incur repeated init
//!   overhead; prefer reusing threads (e.g. a thread pool) for batch work.
//!
//! ## HTML & CSS integration
//!
//! The returned string is an HTML fragment; you are responsible for including
//! the appropriate KaTeX (or Temml) CSS in your page if you want visual layout
//! besides plain MathML. For server‑side rendering pipelines you can inline or
//! bundle the KaTeX stylesheet separately.
//!
//! ## Choosing an output type
//!
//! Set via [`Opts::output_type`]:
//! * `Html` – visual HTML only (smallest payload, least accessible).
//! * `Mathml` – MathML only (pair well with `temml` for semantic output).
//! * `HtmlAndMathml` – default KaTeX hybrid (good balance of accessibility & compatibility).
//!
//! ## Security
//!
//! If you accept untrusted LaTeX input consider:
//! * Set `throw_on_error(false)` to avoid throwing on invalid input.
//! * Leave `trust(false)` (default) so potentially unsafe constructs (e.g. `\url{}`)
//!   are sanitized.
//! * Filter / sandbox usage depending on your deployment context.
//!
//! ## Minimum Supported Rust Version (MSRV)
//! This crate follows *Cargo.toml* specification; consult `Cargo.toml` for the
//! currently tested MSRV. Semver‑minor bumps may raise MSRV with justification.
//!
//! ## License
//! Dual‑licensed under either MIT or Apache‑2.0 at your option.
//!
//! ---
//! Happy typesetting! 🧮

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use core::iter;

pub mod error;
pub use error::{Error, Result};

pub mod opts;
pub use opts::{Opts, OptsBuilder, OutputType};

mod js_engine;
use js_engine::{Engine, JsEngine};

/// KaTeX version.
pub const KATEX_VERSION: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/KATEX-VERSION"));

/// JS source code.
#[cfg(not(feature = "temml"))]
const JS_SRC: &str = concat!(
    // HACK to load KaTeX code in Node.js
    // By setting `module` and `exports` as undefined, we prevent KaTeX to
    // be loaded like normal Node.js module.
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/node-hack.js")),
    // KaTeX JS source code
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/katex.min.js")),
    // mhchem JS source code
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendor/contrib/mhchem.min.js"
    )),
    // restore HACK done in node-hack.js
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/post-node-hack.js")),
    // entry function
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/entry.js")),
);

#[cfg(feature = "temml")]
const JS_SRC: &str = concat!(
    // HACK to load KaTeX code in Node.js
    // By setting `module` and `exports` as undefined, we prevent KaTeX to
    // be loaded like normal Node.js module.
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/node-hack.js")),
    // KaTeX JS source code
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/katex.min.js")),
    // mhchem JS source code
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendor/contrib/mhchem.min.js"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendor/temml/dist/temml.min.js"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendor/temml/contrib/mhchem/mhchem.min.js"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendor/temml/contrib/physics/physics.js"
    )),
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendor/temml/contrib/texvc/texvc.js"
    )),
    // restore HACK done in node-hack.js
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/post-node-hack.js")),
    // entry function
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/entry.js")),
);

thread_local! {
    /// Per thread JS Engine used to render KaTeX.
    static KATEX: Result<Engine> = init_katex();
}

/// Initialize KaTeX js environment.
fn init_katex<E>() -> Result<E>
where
    E: JsEngine,
{
    let engine = E::new()?;
    engine.eval(JS_SRC)?;
    Ok(engine)
}

/// Render LaTeX equation to HTML using specified [engine](`JsEngine`) and [options](`Opts`).
#[inline]
fn render_inner<E>(engine: &E, input: &str, opts: impl AsRef<Opts>) -> Result<String>
where
    E: JsEngine,
{
    let opts = opts.as_ref();
    let input = engine.create_string_value(input.to_owned())?;
    let opts_js = opts.to_js_value(engine)?;
    let args = iter::once(input).chain(iter::once(opts_js));
    let result = (if cfg!(feature = "temml") && opts.is_mathml_only() {
        engine.call_function("temmlRenderToString", args)
    } else {
        engine.call_function("katexRenderToString", args)
    })?;
    engine.value_to_string(result)
}

/// Render LaTeX equation to HTML with additional [options](`Opts`).
pub fn render_with_opts(input: &str, opts: impl AsRef<Opts>) -> Result<String> {
    KATEX.with(|engine| {
        engine
            .as_ref()
            .map_err(|e| e.clone())
            .and_then(|engine| render_inner(engine, input, opts))
    })
}

/// Render LaTeX equation to HTML.
#[inline]
pub fn render(input: &str) -> Result<String> {
    render_with_opts(input, Opts::default())
}

#[cfg(test)]
mod tests;
