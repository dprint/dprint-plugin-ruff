use dprint_core::configuration::ParseConfigurationError;
use dprint_core::generate_str_to_from;
use serde::Deserialize;
use serde::Serialize;

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
