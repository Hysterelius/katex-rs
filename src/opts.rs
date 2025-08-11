//! Configuration options controlling how KaTeX / Temml renders math.
//!
//! The central type is [`Opts`], constructed either directly or (more commonly)
//! via the ergonomic [`Opts::builder`]. Unspecified options fall back to the
//! defaults provided by KaTeX / Temml themselves. Only options explicitly set
//! are forwarded to the underlying JS engine so the surface stays minimal.
//!
//! See the upstream KaTeX documentation for the semantics of most fields:
//! <https://katex.org/docs/options.html>. Temml‑specific options are documented
//! at <https://temml.org/docs/en/administration#options>.
//!
//! # Example
//!
//! Basic usage with the builder pattern:
//! ```
//! let opts = katex::Opts::builder()
//!     .display_mode(true)
//!     .output_type(katex::OutputType::HtmlAndMathml)
//!     .error_color("#cc0000")
//!     .macros(std::collections::HashMap::from([
//!         (r"\\RR".into(), r"\\mathbb{R}".into())
//!     ]))
//!     .build()
//!     .unwrap();
//! let html = katex::render_with_opts(r"\\RR", &opts).unwrap();
//! assert!(html.contains("mathbb"));
//! ```

use crate::{error::Result, js_engine::JsEngine};
use derive_builder::Builder;
use itertools::process_results;
use std::{collections::HashMap, fmt};

/// Options to be passed to KaTeX.
///
/// Read <https://katex.org/docs/options.html> for more information.
#[non_exhaustive]
#[derive(Clone, Builder, Debug, Default)]
#[builder(default)]
#[builder(setter(into, strip_option))]
pub struct Opts {
    /// Whether to render math in KaTeX *display* mode (`true`) or *inline* (`false`).
    ///
    /// Display mode centers the expression on its own line and uses larger
    /// vertical spacing. Corresponds to KaTeX `displayMode`.
    display_mode: Option<bool>,
    /// Which output format KaTeX should produce.
    ///
    /// Defaults to KaTeX's hybrid HTML + MathML when unset.
    output_type: Option<OutputType>,
    /// Whether to typeset equation tags / numbers (`\tag{}` / `\label{}`)
    /// on the left instead of the right (LaTeX's `leqno`).
    leqno: Option<bool>,
    /// Whether display mode equations are left‑aligned instead of centered (`fleqn`).
    fleqn: Option<bool>,
    /// If `true`, parsing invalid LaTeX will raise an error (returned as
    /// [`Error::JsExecError`]); if `false` KaTeX inserts error nodes styled by
    /// [`error_color`].
    throw_on_error: Option<bool>,
    /// CSS color (hex / rgb / named) applied to invalid LaTeX segments when
    /// `throw_on_error` is `false`.
    error_color: Option<String>,
    /// Collection of custom macros.
    /// Read <https://katex.org/docs/options.html> for more information.
    macros: HashMap<String, String>,
    /// Specifies a minimum thickness, in ems.
    /// Read <https://katex.org/docs/options.html> for more information.
    min_rule_thickness: Option<f64>,
    /// Max size for user-specified sizes.
    /// If set to `None`, users can make elements and spaces arbitrarily large.
    /// Read <https://katex.org/docs/options.html> for more information.
    #[allow(clippy::option_option)]
    max_size: Option<Option<f64>>,
    /// Limit the number of macro expansions to the specified number.
    /// If set to `None`, the macro expander will try to fully expand as in LaTeX.
    /// Read <https://katex.org/docs/options.html> for more information.
    #[allow(clippy::option_option)]
    max_expand: Option<Option<i32>>,
    /// Whether to trust users' input.
    /// Read <https://katex.org/docs/options.html> for more information.
    trust: Option<bool>,

    /// Temml-sepcific:
    /// whether to annotate MathML with input LaTeX string.
    /// Read <https://temml.org/docs/en/administration#options> for more information.
    #[cfg(feature = "temml")]
    annotate: Option<bool>,
    /// Temml-sepcific:
    /// where to insert soft line breaks.
    /// Read <https://temml.org/docs/en/administration#options> for more information.
    #[cfg(feature = "temml")]
    wrap: Option<WrapMode>,
    /// Temml-sepcific:
    /// whether to include an XML namespace inside MathML elements.
    /// Read <https://temml.org/docs/en/administration#options> for more information.
    #[cfg(feature = "temml")]
    xml: Option<bool>,
}

impl Opts {
    /// Return [`OptsBuilder`].
    pub fn builder() -> OptsBuilder {
        OptsBuilder::default()
    }

    /// Set whether to render the math in display mode.
    pub fn set_display_mode(&mut self, flag: bool) {
        self.display_mode = Some(flag);
    }

    /// Whether the output type is MathML only (allowing usage of Temml).
    pub(crate) fn is_mathml_only(&self) -> bool {
        self.output_type == Some(OutputType::Mathml)
    }

    /// Set which format(s) to emit.
    pub fn set_output_type(&mut self, output_type: OutputType) {
        self.output_type = Some(output_type);
    }

    /// Set whether to place equation tags on the left.
    pub fn set_leqno(&mut self, flag: bool) {
        self.leqno = Some(flag);
    }

    /// Set whether display math should be left‑aligned.
    pub fn set_fleqn(&mut self, flag: bool) {
        self.fleqn = Some(flag);
    }

    /// Set whether invalid LaTeX triggers a hard error.
    pub fn set_throw_on_error(&mut self, flag: bool) {
        self.throw_on_error = Some(flag);
    }

    /// Set the color used for decorating invalid LaTeX segments.
    pub fn set_error_color(&mut self, color: String) {
        self.error_color = Some(color);
    }

    /// Add a single custom macro mapping. Convenience for inserting into
    /// [`Opts::macros`]. See KaTeX docs for macro expansion semantics.
    pub fn add_macro(&mut self, entry_name: String, entry_data: String) {
        self.macros.insert(entry_name, entry_data);
    }

    /// Set the minimum thickness (in `em`) for fraction lines, `\rule`, etc.
    pub fn set_min_rule_thickness(&mut self, value: f64) {
        self.min_rule_thickness = Some(value);
    }

    /// Set the max size (in `em`) for user‑specified sizes (e.g. via `\rule`).
    ///
    /// `None` removes the limit (allowing arbitrarily large elements). The
    /// outer `Option` indicates whether to send this override at all.
    pub fn set_max_size(&mut self, value: Option<f64>) {
        self.max_size = Some(value);
    }

    /// Set the limit for macro expansion depth. Prevents runaway recursion.
    ///
    /// * `Some(Some(n))` – Explicit finite limit.
    /// * `Some(None)` – Remove limit (use with care!).
    /// * `None` – Do not override KaTeX default.
    pub fn set_max_expand(&mut self, value: Option<i32>) {
        self.max_expand = Some(value);
    }

    /// Set whether to trust user input for potentially unsafe commands.
    ///
    /// Controls sanitization of constructs like `\url{}` and raw HTML. Keep
    /// `false` for untrusted input sources.
    pub fn set_trust(&mut self, flag: bool) {
        self.trust = Some(flag);
    }

    /// Temml-specific: add an annotation with the source LaTeX inside the
    /// generated MathML (facilitates copy/paste fidelity and debugging).
    #[cfg(feature = "temml")]
    pub fn set_annotate(&mut self, flag: bool) {
        self.annotate = Some(flag);
    }

    /// Temml-specific: choose where soft line breaks may be inserted.
    #[cfg(feature = "temml")]
    pub fn set_wrap(&mut self, mode: WrapMode) {
        self.wrap = Some(mode);
    }

    /// Temml-specific: include the XML namespace on `<math>` elements.
    #[cfg(feature = "temml")]
    pub fn set_xml(&mut self, flag: bool) {
        self.xml = Some(flag);
    }

    pub(crate) fn to_js_value<'a, E>(&self, engine: &'a E) -> Result<E::JsValue<'a>>
    where
        E: JsEngine,
    {
        let mut opt: HashMap<String, E::JsValue<'a>> = HashMap::new();
        if let Some(display_mode) = self.display_mode {
            opt.insert(
                "displayMode".to_owned(),
                engine.create_bool_value(display_mode)?,
            );
        }
        if let Some(output_type) = self.output_type {
            opt.insert(
                "output".to_owned(),
                engine.create_string_value(output_type.to_string())?,
            );
        }
        if let Some(leqno) = self.leqno {
            opt.insert("leqno".to_owned(), engine.create_bool_value(leqno)?);
        }
        if let Some(fleqn) = self.fleqn {
            opt.insert("fleqn".to_owned(), engine.create_bool_value(fleqn)?);
        }
        if let Some(throw_on_error) = self.throw_on_error {
            opt.insert(
                "throwOnError".to_owned(),
                engine.create_bool_value(throw_on_error)?,
            );
        }
        if let Some(error_color) = &self.error_color {
            opt.insert(
                "errorColor".to_owned(),
                engine.create_string_value(error_color.clone())?,
            );
        }
        if !self.macros.is_empty() {
            let macros = process_results(
                self.macros
                    .iter()
                    .map(|(k, v)| -> Result<(String, E::JsValue<'a>)> {
                        Ok((k.clone(), engine.create_string_value(v.clone())?))
                    }),
                |iter| -> Result<E::JsValue<'a>> { engine.create_object_value(iter) },
            )??;

            opt.insert("macros".to_owned(), macros);
        }
        if let Some(min_rule_thickness) = self.min_rule_thickness {
            opt.insert(
                "minRuleThickness".to_owned(),
                engine.create_float_value(min_rule_thickness)?,
            );
        }
        if let Some(Some(max_size)) = self.max_size {
            opt.insert("maxSize".to_owned(), engine.create_float_value(max_size)?);
        }
        if let Some(max_expand) = self.max_expand {
            match max_expand {
                Some(max_expand) => {
                    opt.insert("maxExpand".to_owned(), engine.create_int_value(max_expand)?);
                }
                None => {
                    opt.insert("maxExpand".to_owned(), engine.create_int_value(i32::MAX)?);
                }
            }
        }
        if let Some(trust) = self.trust {
            opt.insert("trust".to_owned(), engine.create_bool_value(trust)?);
        }

        #[cfg(feature = "temml")]
        if let Some(annotate) = self.annotate {
            opt.insert("xml".to_owned(), engine.create_bool_value(annotate)?);
        }

        #[cfg(feature = "temml")]
        if let Some(wrap) = self.wrap {
            opt.insert(
                "wrap".to_owned(),
                engine.create_string_value(wrap.to_string())?,
            );
        }

        #[cfg(feature = "temml")]
        if let Some(xml) = self.xml {
            opt.insert("xml".to_owned(), engine.create_bool_value(xml)?);
        }

        engine.create_object_value(opt.into_iter())
    }
}

impl AsRef<Opts> for Opts {
    fn as_ref(&self) -> &Opts {
        self
    }
}

impl OptsBuilder {
    /// Add (chain) a macro mapping into the accumulated macro table.
    ///
    /// Shorthand for manipulating the `macros` map directly. Duplicate keys
    /// are overwritten by later calls.
    ///
    /// # Examples
    ///
    /// ```
    /// let opts = katex::Opts::builder()
    ///     .add_macro(r#"\RR"#.to_owned(), r#"\mathbb{R}"#.to_owned())
    ///     .build()
    ///     .unwrap();
    /// let html = katex::render_with_opts(r#"\RR"#, &opts).unwrap();
    /// ```
    pub fn add_macro(mut self, entry_name: String, entry_data: String) -> Self {
        match self.macros.as_mut() {
            Some(macros) => {
                macros.insert(entry_name, entry_data);
            }
            None => {
                let mut macros = HashMap::new();
                macros.insert(entry_name, entry_data);
                self.macros = Some(macros);
            }
        }
        self
    }
}

/// Output type from KaTeX.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutputType {
    /// Outputs KaTeX in HTML only.
    Html,
    /// Outputs KaTeX in MathML only.
    Mathml,
    /// Outputs HTML for visual rendering and includes MathML for accessibility.
    HtmlAndMathml,
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            OutputType::Html => "html",
            OutputType::Mathml => "mathml",
            OutputType::HtmlAndMathml => "htmlAndMathml",
        })
    }
}

/// Wrap mode for Temml.
#[non_exhaustive]
#[cfg(feature = "temml")]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WrapMode {
    /// Soft line break after every top-level relation and binary operator.
    Tex,
    /// Soft line break after every top-level `=` except for the first.
    Equals,
    /// No soft line breaks.
    None,
}

#[cfg(feature = "temml")]
impl fmt::Display for WrapMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            WrapMode::Tex => "tex",
            WrapMode::Equals => "=",
            WrapMode::None => "none",
        })
    }
}
