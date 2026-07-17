use dprint_core::configuration::ParseConfigurationError;
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
}

generate_str_to_from![LineEnding, [LineFeed, "lf"], [CarriageReturnLineFeed, "cr-lf"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuoteStyle {
  Single,
  Double,
}

generate_str_to_from![QuoteStyle, [Single, "single"], [Double, "double"]];

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
  pub indent_style: Option<IndentStyle>,
  pub line_length: Option<u16>,
  pub indent_width: Option<u8>,
  pub line_ending: Option<LineEnding>,
  pub quote_style: Option<QuoteStyle>,
  pub skip_magic_trailing_comma: Option<bool>,
  pub preview: Option<bool>,
}
