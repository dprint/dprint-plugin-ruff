use std::sync::Arc;
use std::sync::OnceLock;

use dprint_core::configuration::ParseConfigurationError;
use ruff_linter::settings::LinterSettings;
use ruff_python_ast::PythonVersion;
use serde::Deserialize;
use serde::Serialize;

macro_rules! generate_str_to_from {
  ($enum_name:ident, $([$member_name:ident, $string_value:expr]),* ) => {
    impl std::str::FromStr for $enum_name {
      type Err = ParseConfigurationError;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          $($string_value => Ok(Self::$member_name)),*,
          _ => Err(ParseConfigurationError(String::from(s))),
        }
      }
    }

    impl std::fmt::Display for $enum_name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          $(Self::$member_name => f.write_str($string_value)),*,
        }
      }
    }
  };
}

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IndentStyle {
  Tab,
  Space,
}

generate_str_to_from![IndentStyle, [Tab, "tab"], [Space, "space"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineEnding {
  LineFeed,
  CarriageReturnLineFeed,
  CarriageReturn,
}

generate_str_to_from![
  LineEnding,
  [LineFeed, "lf"],
  [CarriageReturnLineFeed, "cr-lf"],
  [CarriageReturn, "cr"]
];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuoteStyle {
  Single,
  Double,
  Preserve,
}

generate_str_to_from![
  QuoteStyle,
  [Single, "single"],
  [Double, "double"],
  [Preserve, "preserve"]
];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NestedStringQuoteStyle {
  Alternating,
  Preferred,
}

generate_str_to_from![
  NestedStringQuoteStyle,
  [Alternating, "alternating"],
  [Preferred, "preferred"]
];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DocstringCodeLineWidth {
  Fixed(std::num::NonZeroU16),
  Dynamic(DynamicLineWidth),
}

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DynamicLineWidth {
  Dynamic,
}

impl std::str::FromStr for DocstringCodeLineWidth {
  type Err = ParseConfigurationError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    if value == "dynamic" {
      Ok(Self::Dynamic(DynamicLineWidth::Dynamic))
    } else {
      value
        .parse()
        .map(Self::Fixed)
        .map_err(|_| ParseConfigurationError(value.to_string()))
    }
  }
}

impl std::fmt::Display for DocstringCodeLineWidth {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Fixed(width) => width.fmt(f),
      Self::Dynamic(_) => f.write_str("dynamic"),
    }
  }
}

#[derive(Clone, PartialEq, Copy, Debug)]
pub struct TargetVersion(pub PythonVersion);

impl TargetVersion {
  /// All accepted config values: every Python version the bundled Ruff supports.
  pub fn all_names() -> Vec<String> {
    PythonVersion::iter().map(|version| Self(version).to_string()).collect()
  }
}

impl std::str::FromStr for TargetVersion {
  type Err = ParseConfigurationError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    PythonVersion::iter()
      .map(Self)
      .find(|target_version| value == target_version.to_string())
      .ok_or_else(|| ParseConfigurationError(value.to_string()))
  }
}

impl std::fmt::Display for TargetVersion {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "py{}{}", self.0.major, self.0.minor)
  }
}

impl Serialize for TargetVersion {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}

impl<'de> Deserialize<'de> for TargetVersion {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    use serde::de::Error;
    String::deserialize(deserializer)?.parse().map_err(D::Error::custom)
  }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixLintErrors {
  pub unused_import: Option<bool>,
  pub unsorted_imports: Option<bool>,
  pub missing_required_import: Option<bool>,
  pub required_imports: Option<Vec<String>>,
  pub known_first_party: Option<Vec<String>>,
  pub known_third_party: Option<Vec<String>>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
  pub indent_style: Option<IndentStyle>,
  pub line_length: Option<u16>,
  pub indent_width: Option<u8>,
  pub line_ending: Option<LineEnding>,
  pub quote_style: Option<QuoteStyle>,
  pub skip_magic_trailing_comma: Option<bool>,
  pub docstring_code_format: Option<bool>,
  pub docstring_code_line_width: Option<DocstringCodeLineWidth>,
  pub nested_string_quote_style: Option<NestedStringQuoteStyle>,
  pub preview: Option<bool>,
  pub target_version: Option<TargetVersion>,
  pub fix_lint_errors: Option<FixLintErrors>,
  #[doc(hidden)]
  #[serde(skip)]
  pub linter_settings: LinterSettingsCache,
}

/// Ruff linter settings resolved from a `Configuration` the first time it is used to
/// format a file, so they are not rebuilt for every file.
///
/// Changing a `Configuration` after it has formatted a file does not update these settings.
#[derive(Default, Clone)]
pub struct LinterSettingsCache(OnceLock<Result<Option<Arc<LinterSettings>>, String>>);

impl LinterSettingsCache {
  /// Gets the settings, or `None` when no lint fixes are enabled.
  pub(crate) fn get_or_resolve(
    &self,
    resolve: impl FnOnce() -> anyhow::Result<Option<LinterSettings>>,
  ) -> anyhow::Result<Option<&LinterSettings>> {
    let resolved = self.0.get_or_init(|| {
      resolve()
        .map(|settings| settings.map(Arc::new))
        .map_err(|error| error.to_string())
    });
    match resolved {
      Ok(settings) => Ok(settings.as_deref()),
      Err(message) => Err(anyhow::anyhow!("{message}")),
    }
  }
}
