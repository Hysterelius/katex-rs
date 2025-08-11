//! Custom KaTeX behaviors.

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
    /// Whether to render the math in the display mode.
    display_mode: Option<bool>,
    /// KaTeX output type.
    output_type: Option<OutputType>,
    /// Whether to have `\tags` rendered on the left instead of the right.
    leqno: Option<bool>,
    /// Whether to make display math flush left.
    fleqn: Option<bool>,
    /// Whether to let KaTeX throw a ParseError for invalid LaTeX.
    throw_on_error: Option<bool>,
    /// Color used for invalid LaTeX.
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

    /// Set whether to render the math in the display mode.
    pub fn set_display_mode(&mut self, flag: bool) {
        self.display_mode = Some(flag);
    }

    /// Whether the output type is MathML only (allowing usage of Temml).
    pub(crate) fn is_mathml_only(&self) -> bool {
        self.output_type == Some(OutputType::Mathml)
    }

    /// Set KaTeX output type.
    pub fn set_output_type(&mut self, output_type: OutputType) {
        self.output_type = Some(output_type);
    }

    /// Set whether to have `\tags` rendered on the left instead of the right.
    pub fn set_leqno(&mut self, flag: bool) {
        self.leqno = Some(flag);
    }

    /// Set whether to make display math flush left.
    pub fn set_fleqn(&mut self, flag: bool) {
        self.fleqn = Some(flag);
    }

    /// Set whether to let KaTeX throw a ParseError for invalid LaTeX.
    pub fn set_throw_on_error(&mut self, flag: bool) {
        self.throw_on_error = Some(flag);
    }

    /// Set the color used for invalid LaTeX.
    pub fn set_error_color(&mut self, color: String) {
        self.error_color = Some(color);
    }

    /// Add a custom macro.
    /// Read <https://katex.org/docs/options.html> for more information.
    pub fn add_macro(&mut self, entry_name: String, entry_data: String) {
        self.macros.insert(entry_name, entry_data);
    }

    /// Set the minimum thickness, in ems.
    /// Read <https://katex.org/docs/options.html> for more information.
    pub fn set_min_rule_thickness(&mut self, value: f64) {
        self.min_rule_thickness = Some(value);
    }

    /// Set the max size for user-specified sizes.
    /// If set to `None`, users can make elements and spaces arbitrarily large.
    /// Read <https://katex.org/docs/options.html> for more information.
    pub fn set_max_size(&mut self, value: Option<f64>) {
        self.max_size = Some(value);
    }

    /// Set the limit for the number of macro expansions.
    /// If set to `None`, the macro expander will try to fully expand as in LaTeX.
    /// Read <https://katex.org/docs/options.html> for more information.
    pub fn set_max_expand(&mut self, value: Option<i32>) {
        self.max_expand = Some(value);
    }

    /// Set whether to trust users' input.
    /// Read <https://katex.org/docs/options.html> for more information.
    pub fn set_trust(&mut self, flag: bool) {
        self.trust = Some(flag);
    }

    /// Temml-specific:
    /// whether to annotate MathML with input LaTeX string.
    /// Read <https://temml.org/docs/en/administration#options> for more information.
    #[cfg(feature = "temml")]
    pub fn set_annotate(&mut self, flag: bool) {
        self.annotate = Some(flag);
    }

    /// Temml-specific:
    /// where to insert soft line breaks.
    /// Read <https://temml.org/docs/en/administration#options> for more information.
    #[cfg(feature = "temml")]
    pub fn set_wrap(&mut self, mode: WrapMode) {
        self.wrap = Some(mode);
    }

    /// Temml-specific:
    /// set whether to specify XML namespace on `<math>` elements.
    /// Read <https://temml.org/docs/en/administration#options> for more information.
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
    /// Add an entry to [`macros`](OptsBuilder::macros).
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
